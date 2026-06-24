package handler

import (
	"context"
	"net/http"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"

	"minibili/internal/data"
	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

type adminLoginReq struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type adminTokenPairResp struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token"`
}

// AdminLogin POST /api/v1/admin/auth/login
func (a *API) AdminLogin(c *gin.Context) {
	var req adminLoginReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var adm model.Admin
	if err := a.DB.Where("username = ?", strings.TrimSpace(req.Username)).First(&adm).Error; err != nil {
		a.recordAdminLoginLog(0, strings.TrimSpace(req.Username), c.ClientIP(), false, "用户不存在")
		resp.Err(c, http.StatusUnauthorized, errcode.CodeInvalidLogin)
		return
	}
	if adm.Status != "active" {
		resp.Err(c, http.StatusForbidden, errcode.CodeAdminDisabled)
		return
	}
	if bcrypt.CompareHashAndPassword([]byte(adm.PasswordHash), []byte(req.Password)) != nil {
		a.recordAdminLoginLog(adm.ID, adm.Username, c.ClientIP(), false, "密码错误")
		resp.Err(c, http.StatusUnauthorized, errcode.CodeInvalidLogin)
		return
	}
	a.recordAdminLoginLog(adm.ID, adm.Username, c.ClientIP(), true, "")
	access, refresh, _, err := a.JWT.IssueAdminPair(adm.ID)
	if err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	now := time.Now()
	_ = a.DB.Model(&adm).Update("last_login_at", now).Error
	a.Log.Info("admin login", zap.String("username", adm.Username), zap.Uint64("admin_id", adm.ID))
	resp.OK(c, adminTokenPairResp{AccessToken: access, RefreshToken: refresh})
}

// AdminRefresh POST /api/v1/admin/auth/refresh
func (a *API) AdminRefresh(c *gin.Context) {
	var req refreshReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	aid, tokenID, err := a.JWT.ParseAdminRefresh(strings.TrimSpace(req.RefreshToken))
	if err != nil {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var adm model.Admin
	if err := a.DB.First(&adm, aid).Error; err != nil || adm.Status != "active" {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	ctx := context.Background()
	if a.Redis.Exists(ctx, data.AdminRefreshInvalidKey(tokenID)).Val() == 1 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	_ = a.Redis.Set(ctx, data.AdminRefreshInvalidKey(tokenID), "1", data.RefreshInvalidTTL).Err()
	access, refresh, _, err := a.JWT.IssueAdminPair(adm.ID)
	if err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, adminTokenPairResp{AccessToken: access, RefreshToken: refresh})
}

// AdminMe GET /api/v1/admin/me
func (a *API) AdminMe(c *gin.Context) {
	aid, ok := adminIDFromCtx(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var adm model.Admin
	if err := a.DB.First(&adm, aid).Error; err != nil {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	resp.OK(c, gin.H{
		"id":           adm.ID,
		"username":     adm.Username,
		"display_name": adm.DisplayName,
	})
}

func adminIDFromCtx(c *gin.Context) (uint64, bool) {
	return middleware.AdminID(c)
}

func (a *API) recordAdminLoginLog(adminID uint64, username, ip string, success bool, reason string) {
	lg := model.AdminLoginLog{
		AdminID: adminID, Username: username, IP: ip,
		Success: success, FailReason: reason,
	}
	if err := a.DB.Create(&lg).Error; err != nil {
		a.Log.Error("record admin login log failed", zap.Error(err))
	}
}

// AdminListLoginLogs GET /api/v1/admin/login-logs
func (a *API) AdminListLoginLogs(c *gin.Context) {
	var logs []model.AdminLoginLog
	if err := a.DB.Order("created_at DESC").Limit(200).Find(&logs).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "查询失败"})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": logs})
}

// AdminListRiskHitLogs GET /api/v1/admin/risk/hits
func (a *API) AdminListRiskHitLogs(c *gin.Context) {
	var hits []model.RiskHitLog
	if err := a.DB.Order("created_at DESC").Limit(200).Find(&hits).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "查询失败"})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": hits})
}
