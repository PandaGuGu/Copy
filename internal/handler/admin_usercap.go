package handler

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
	"minibili/internal/pkg/usercap"
)

// AdminListUserCapabilities GET /admin/users/:id/capabilities
func (a *API) AdminListUserCapabilities(c *gin.Context) {
	uid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var rows []model.UserCapabilityRestriction
	if err := a.DB.Where("user_id = ?", uid).Order("status ASC, id DESC").Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	type item struct {
		model.UserCapabilityRestriction
		Label string `json:"label"`
	}
	items := make([]item, 0, len(rows))
	for _, r := range rows {
		items = append(items, item{UserCapabilityRestriction: r, Label: usercap.Label(r.Capability)})
	}

	// Active set for this user.
	var active []string
	for _, r := range rows {
		if r.Status == "active" {
			active = append(active, r.Capability)
		}
	}

	resp.OK(c, gin.H{"items": items, "active": active, "capabilities": usercap.All, "reason_templates": a.listReasonTemplates()})
}

func (a *API) listReasonTemplates() []string {
	var rows []model.CapabilityReasonTemplate
	if err := a.DB.Order("id ASC").Find(&rows).Error; err != nil {
		return nil
	}
	out := make([]string, 0, len(rows))
	for _, r := range rows {
		out = append(out, r.Content)
	}
	return out
}

// AdminListCapReasonTemplates GET /admin/usercap/templates
func (a *API) AdminListCapReasonTemplates(c *gin.Context) {
	var rows []model.CapabilityReasonTemplate
	if err := a.DB.Order("id ASC").Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"templates": rows})
}

// AdminCreateCapReasonTemplate POST /admin/usercap/templates
func (a *API) AdminCreateCapReasonTemplate(c *gin.Context) {
	var req struct {
		Content string `json:"content"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Content = strings.TrimSpace(req.Content)
	if req.Content == "" || len([]rune(req.Content)) > 200 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	row := model.CapabilityReasonTemplate{Content: req.Content}
	if err := a.DB.Create(&row).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, row)
}

// AdminUpdateCapReasonTemplate PUT /admin/usercap/templates/:id
func (a *API) AdminUpdateCapReasonTemplate(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var req struct {
		Content string `json:"content"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Content = strings.TrimSpace(req.Content)
	if req.Content == "" || len([]rune(req.Content)) > 200 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var row model.CapabilityReasonTemplate
	if err := a.DB.First(&row, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if err := a.DB.Model(&row).Updates(map[string]interface{}{
		"content": req.Content, "updated_at": time.Now(),
	}).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.DB.First(&row, id)
	resp.OK(c, row)
}

// AdminDeleteCapReasonTemplate DELETE /admin/usercap/templates/:id
func (a *API) AdminDeleteCapReasonTemplate(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if err := a.DB.Delete(&model.CapabilityReasonTemplate{}, id).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"deleted": id})
}

// AdminRestrictUserCapability POST /admin/users/:id/capabilities
func (a *API) AdminRestrictUserCapability(c *gin.Context) {
	uid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	adminID, _ := middleware.AdminID(c)

	var req struct {
		Capability string     `json:"capability"`
		Reason     string     `json:"reason"`
		ExpiresAt  *time.Time `json:"expires_at"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Capability = strings.TrimSpace(req.Capability)
	if !usercap.Valid(req.Capability) {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Idempotent upsert against an existing active restriction.
	var existing model.UserCapabilityRestriction
	hasActive := a.DB.Where("user_id = ? AND capability = ? AND status = 'active'", uid, req.Capability).
		First(&existing).Error == nil
	if hasActive {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	row := model.UserCapabilityRestriction{
		UserID:     uid,
		Capability: req.Capability,
		Reason:     strings.TrimSpace(req.Reason),
		ExpiresAt:  req.ExpiresAt,
		Status:     "active",
		CreatedBy:  adminID,
		CreatedAt:  time.Now(),
	}
	if err := a.DB.Create(&row).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	middleware.InvalidateUserCapCache(a.Redis, uid)
	a.notifyCapChange(uid, req.Capability, "restricted", strings.TrimSpace(req.Reason))
	a.Log.Info("admin restricted user capability",
		zap.Uint64("user_id", uid), zap.String("capability", req.Capability), zap.Uint64("admin_id", adminID))

	resp.OK(c, row)
}

// AdminRestoreUserCapability DELETE /admin/users/:id/capabilities/:capability
func (a *API) AdminRestoreUserCapability(c *gin.Context) {
	uid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	adminID, _ := middleware.AdminID(c)
	capability := strings.TrimSpace(c.Param("capability"))
	if !usercap.Valid(capability) {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	note := a.removeCapRestriction(uid, capability)
	if strings.HasPrefix(note, "未找到") || strings.HasPrefix(note, "不存在") {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	middleware.InvalidateUserCapCache(a.Redis, uid)
	a.notifyCapChange(uid, capability, "restored", "")
	a.Log.Info("admin restored user capability",
		zap.Uint64("user_id", uid), zap.String("capability", capability), zap.Uint64("admin_id", adminID))

	resp.OK(c, gin.H{"user_id": uid, "capability": capability, "note": note})
}

// removeCapRestriction lifts an active capability restriction. Returns a note.
func (a *API) removeCapRestriction(uid uint64, capability string) string {
	var row model.UserCapabilityRestriction
	if err := a.DB.Where("user_id = ? AND capability = ? AND status = 'active'", uid, capability).First(&row).Error; err != nil {
		if err.Error() == "record not found" || strings.Contains(err.Error(), "record not found") {
			return "未找到该能力限制"
		}
		return "查询失败"
	}
	now := time.Now()
	if err := a.DB.Model(&row).Updates(map[string]interface{}{
		"status": "restored", "restored_at": &now,
	}).Error; err != nil {
		return "恢复失败"
	}
	return "已恢复能力:" + usercap.Label(capability)
}

func (a *API) notifyCapChange(uid uint64, capability, action, reason string) {
	msg := "您的「" + usercap.Label(capability) + "」功能已被限制"
	if action == "restored" {
		msg = "您的「" + usercap.Label(capability) + "」功能已恢复"
	}
	if reason != "" {
		msg += "（原因：" + reason + "）"
	}
	_ = a.DB.Create(&model.NotificationRecord{
		RecipientID: uid, RecipientType: "user",
		Channel: "in_app", Title: "功能状态通知",
		Content:     msg,
		RelatedType: "user_capability", RelatedID: 0, Status: "pending",
	})
}
