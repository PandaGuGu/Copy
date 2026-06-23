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
		TargetType   string `json:"target_type"`
		TargetID     uint64 `json:"target_id"`
		ReasonType   string `json:"reason_type"`
		ReasonDetail string `json:"reason_detail"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.TargetType = strings.TrimSpace(req.TargetType)
	req.ReasonType = strings.TrimSpace(req.ReasonType)
	req.ReasonDetail = strings.TrimSpace(req.ReasonDetail)

	if req.TargetType == "" || req.TargetID == 0 || req.ReasonType == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	if !isValidReportTarget(req.TargetType) {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	if !isValidReasonType(req.ReasonType) {
		req.ReasonType = "other"
	}

	r := model.Report{
		ReporterID:   userID,
		TargetType:   req.TargetType,
		TargetID:     req.TargetID,
		ReasonType:   req.ReasonType,
		ReasonDetail: req.ReasonDetail,
		Status:       "pending",
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

func isValidReasonType(t string) bool {
	for _, r := range model.ReportReasonTypes {
		if r.Type == t {
			return true
		}
	}
	return false
}

// ---------- Admin ----------

// AdminListReports GET /api/v1/admin/reports
func (a *API) AdminListReports(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	st := strings.TrimSpace(c.Query("status"))      // pending / resolved / dismissed / empty=all
	target := strings.TrimSpace(c.Query("target"))   // video/article/dynamic/comment/user
	rtype := strings.TrimSpace(c.Query("reason_type")) // reason type filter

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
	if target != "" {
		tx = tx.Where("target_type = ?", target)
	}
	if rtype != "" {
		tx = tx.Where("reason_type = ?", rtype)
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
		ReasonType   string     `json:"reason_type"`
		ReasonLabel  string     `json:"reason_label"`
		ReasonDetail string     `json:"reason_detail"`
		Status       string     `json:"status"`
		HandlerNote  string     `json:"handler_note"`
		CreatedAt    time.Time  `json:"created_at"`
		HandledAt    *time.Time `json:"handled_at"`
	}
	items := make([]item, 0, len(reports))
	uidSet := make(map[uint64]bool)
	for _, r := range reports {
		uidSet[r.ReporterID] = true
		items = append(items, item{
			ID:           r.ID,
			ReporterID:   r.ReporterID,
			TargetType:   r.TargetType,
			TargetID:     r.TargetID,
			ReasonType:   r.ReasonType,
			ReasonLabel:  model.ReportReasonLabel(r.ReasonType),
			ReasonDetail: r.ReasonDetail,
			Status:       r.Status,
			HandlerNote:  r.HandlerNote,
			CreatedAt:    r.CreatedAt,
			HandledAt:    r.HandledAt,
		})
	}

	// Batch load reporters
	type uBrief struct {
		ID         uint64
		Username   string
		Nickname   string
		AvatarURL  string
	}
	uids := make([]uint64, 0, len(uidSet))
	for uid := range uidSet {
		uids = append(uids, uid)
	}
	if len(uids) > 0 {
		var users []uBrief
		a.DB.Model(&model.User{}).Select("id, username, nickname, avatar_url").Where("id IN ?", uids).Find(&users)
		um := make(map[uint64]uBrief, len(users))
		for _, u := range users {
			um[u.ID] = u
		}
		for i := range items {
			u, ok := um[items[i].ReporterID]
			if ok {
				items[i].Reporter = gin.H{
					"id":         u.ID,
					"username":   u.Username,
					"nickname":   u.Nickname,
					"avatar_url": u.AvatarURL,
				}
			}
		}
	}

	// Stats
	var pendingCount, resolvedCount, dismissedCount int64
	a.DB.Model(&model.Report{}).Where("status = 'pending'").Count(&pendingCount)
	a.DB.Model(&model.Report{}).Where("status = 'resolved'").Count(&resolvedCount)
	a.DB.Model(&model.Report{}).Where("status = 'dismissed'").Count(&dismissedCount)

	// Reason distribution (pending only, for overview)
	type reasonStat struct {
		Type  string `json:"type"`
		Label string `json:"label"`
		Count int64  `json:"count"`
	}
	var reasonStats []reasonStat
	for _, rt := range model.ReportReasonTypes {
		var c int64
		a.DB.Model(&model.Report{}).Where("reason_type = ? AND status = 'pending'", rt.Type).Count(&c)
		if c > 0 {
			reasonStats = append(reasonStats, reasonStat{Type: rt.Type, Label: rt.Label, Count: c})
		}
	}

	resp.OK(c, gin.H{
		"items":           items,
		"total":           total,
		"page":            page,
		"page_size":       pageSize,
		"pending_count":   pendingCount,
		"resolved_count":  resolvedCount,
		"dismissed_count": dismissedCount,
		"reason_stats":    reasonStats,
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

// AdminBatchHandleReports POST /api/v1/admin/reports/batch
func (a *API) AdminBatchHandleReports(c *gin.Context) {
	var req struct {
		IDs         []uint64 `json:"ids"`
		Action      string   `json:"action"`
		HandlerNote string   `json:"handler_note"`
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
	if len(req.IDs) == 0 || len(req.IDs) > 100 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)
	now := time.Now()
	newStatus := "resolved"
	if req.Action == "dismiss" {
		newStatus = "dismissed"
	}

	count := a.DB.Model(&model.Report{}).
		Where("id IN ? AND status = 'pending'", req.IDs).
		Updates(map[string]interface{}{
			"status":       newStatus,
			"handler_note": strings.TrimSpace(req.HandlerNote),
			"handled_by":   adminID,
			"handled_at":   now,
		})

	resp.OK(c, gin.H{"handled": count.RowsAffected})
}
