package handler

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// PostReport POST /api/v1/reports (authenticated)
func (a *API) PostReport(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	var req struct {
		TargetType string `json:"target_type"`
		TargetID   uint64 `json:"target_id"`
		Reason     string `json:"reason"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.TargetType = strings.TrimSpace(req.TargetType)
	req.Reason = strings.TrimSpace(req.Reason)

	if req.TargetType == "" || req.TargetID == 0 || req.Reason == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	if !isValidReportTarget(req.TargetType) {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	r := model.Report{
		ReporterID: userID,
		TargetType: req.TargetType,
		TargetID:   req.TargetID,
		Reason:     req.Reason,
		Status:     "pending",
	}
	if err := a.DB.Create(&r).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{"id": r.ID})
}

func isValidReportTarget(t string) bool {
	switch t {
	case "video", "article", "dynamic", "comment", "user":
		return true
	}
	return false
}

// ---------- Admin ----------

// AdminListReports GET /api/v1/admin/reports
func (a *API) AdminListReports(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	st := strings.TrimSpace(c.Query("status")) // pending / resolved / dismissed / empty=all

	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tx := a.DB.Model(&model.Report{})
	if st != "" {
		tx = tx.Where("status = ?", st)
	}
	var total int64
	tx.Count(&total)

	var reports []model.Report
	tx.Order("created_at DESC").Offset((page - 1) * pageSize).Limit(pageSize).Find(&reports)

	type item struct {
		ID           uint64     `json:"id"`
		ReporterID   uint64     `json:"reporter_id"`
		Reporter     gin.H      `json:"reporter"`
		TargetType   string     `json:"target_type"`
		TargetID     uint64     `json:"target_id"`
		Reason       string     `json:"reason"`
		Status       string     `json:"status"`
		HandlerNote  string     `json:"handler_note"`
		CreatedAt    time.Time  `json:"created_at"`
		HandledAt    *time.Time `json:"handled_at"`
	}
	items := make([]item, 0, len(reports))
	for _, r := range reports {
		it := item{
			ID:         r.ID,
			ReporterID: r.ReporterID,
			TargetType: r.TargetType,
			TargetID:   r.TargetID,
			Reason:     r.Reason,
			Status:     r.Status,
			HandlerNote: r.HandlerNote,
			CreatedAt:  r.CreatedAt,
			HandledAt:  r.HandledAt,
		}
		// Load reporter brief
		var u model.User
		if a.DB.Select("id, username, nickname, avatar_url").First(&u, r.ReporterID).Error == nil {
			it.Reporter = gin.H{
				"id":         u.ID,
				"username":   u.Username,
				"nickname":   u.Nickname,
				"avatar_url": u.AvatarURL,
			}
		}
		items = append(items, it)
	}

	resp.OK(c, gin.H{
		"items":     items,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	})
}

// AdminHandleReport POST /api/v1/admin/reports/:id/handle
func (a *API) AdminHandleReport(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req struct {
		Action      string `json:"action"`       // "resolve" / "dismiss"
		HandlerNote string `json:"handler_note"` // optional
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.Action = strings.TrimSpace(req.Action)
	if req.Action != "resolve" && req.Action != "dismiss" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var r model.Report
	if err := a.DB.First(&r, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if r.Status != "pending" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	newStatus := "resolved"
	if req.Action == "dismiss" {
		newStatus = "dismissed"
	}

	updates := map[string]interface{}{
		"status":       newStatus,
		"handler_note": strings.TrimSpace(req.HandlerNote),
		"handled_by":   adminID,
		"handled_at":   now,
	}
	if err := a.DB.Model(&r).Updates(updates).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{"status": newStatus})
}
