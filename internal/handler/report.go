package handler

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"

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

	// Auto-flag: if target has >= 3 pending reports, mark content
	autoFlagContent(a.DB, r)
}

func autoFlagContent(db *gorm.DB, r model.Report) {
	var pendingCount int64
	db.Model(&model.Report{}).
		Where("target_type = ? AND target_id = ? AND status = 'pending'", r.TargetType, r.TargetID).
		Count(&pendingCount)

	const autoThreshold = 3
	if pendingCount < autoThreshold {
		return
	}

	switch r.TargetType {
	case "video":
		db.Model(&model.Video{}).Where("id = ? AND status IN ('published','pending_review')", r.TargetID).
			Update("fail_reason", "[系统] 该内容被多次举报，待管理员审核")
	case "article":
		db.Model(&model.Article{}).Where("id = ? AND status IN ('published','pending_review')", r.TargetID).
			Update("fail_reason", "[系统] 该内容被多次举报，待管理员审核")
	case "user":
		var u model.User
		if db.Select("id, status").First(&u, r.TargetID).Error == nil && u.Status == "active" {
			db.Model(&u).Updates(map[string]interface{}{
				"banned_reason": "[系统] 该用户被多次举报，待管理员审核",
			})
		}
	}
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
		Action        string `json:"action"`         // "resolve" / "dismiss"
		ContentAction string `json:"content_action"` // "none" / "takedown" / "warn" / "ban"
		HandlerNote   string `json:"handler_note"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.Action = strings.TrimSpace(req.Action)
	if req.Action != "resolve" && req.Action != "dismiss" && req.Action != "revert" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.ContentAction = strings.TrimSpace(req.ContentAction)

	adminID, _ := middleware.AdminID(c)

	var r model.Report
	if err := a.DB.First(&r, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if r.Status != "pending" && req.Action != "revert" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.Action == "revert" && r.Status == "pending" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Revert: set back to pending
	if req.Action == "revert" {
		a.DB.Model(&r).Updates(map[string]interface{}{
			"status":       "pending",
			"handler_note": "",
			"handled_by":   0,
			"handled_at":   nil,
		})
		resp.OK(c, gin.H{"status": "pending"})
		return
	}

	now := time.Now()
	newStatus := "resolved"
	if req.Action == "dismiss" {
		newStatus = "dismissed"
	}

	note := strings.TrimSpace(req.HandlerNote)

	// Execute content action (only when resolving, not dismissing)
	contentResult := ""
	if req.Action == "resolve" {
		contentResult = a.executeContentAction(r.TargetType, r.TargetID, req.ContentAction, note)
		if contentResult != "" {
			if note != "" {
				note += " | "
			}
			note += contentResult
		}
	}

	updates := map[string]interface{}{
		"status":       newStatus,
		"handler_note": note,
		"handled_by":   adminID,
		"handled_at":   now,
	}
	if err := a.DB.Model(&r).Updates(updates).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{"status": newStatus, "content_result": contentResult})
}

// executeContentAction performs moderation on reported content.
func (a *API) executeContentAction(targetType string, targetID uint64, action, note string) string {
	switch targetType {
	case "video":
		return a.moderateVideo(targetID, action)
	case "article":
		return a.moderateArticle(targetID, action)
	case "dynamic":
		return a.moderateDynamic(targetID, action)
	case "comment":
		return a.moderateComment(targetID, action)
	case "user":
		return a.moderateUser(targetID, action)
	}
	return ""
}

func (a *API) moderateVideo(id uint64, action string) string {
	var v model.Video
	if a.DB.Select("id, status, user_id").First(&v, id).Error != nil {
		return ""
	}
	switch action {
	case "takedown":
		if v.Status != "published" && v.Status != "pending_review" {
			return "视频不是可退回状态"
		}
		a.DB.Model(&v).Updates(map[string]interface{}{
			"status":      "deleted",
			"fail_reason": "管理员下架：因举报违规",
		})
		return "视频已下架"
	case "warn":
		return a.warnAuthor(v.UserID, "您的视频因违规被举报，请自查内容")
	case "ban":
		return a.banAuthor(v.UserID, "视频违规")
	}
	return ""
}

func (a *API) moderateArticle(id uint64, action string) string {
	var ar model.Article
	if a.DB.Select("id, status, user_id").First(&ar, id).Error != nil {
		return ""
	}
	switch action {
	case "takedown":
		if ar.Status != "published" && ar.Status != "pending_review" {
			return "文章不是可退回状态"
		}
		a.DB.Model(&ar).Updates(map[string]interface{}{
			"status":      "deleted",
			"fail_reason": "管理员下架：因举报违规",
		})
		return "文章已下架"
	case "warn":
		return a.warnAuthor(ar.UserID, "您的文章因违规被举报，请自查内容")
	case "ban":
		return a.banAuthor(ar.UserID, "文章违规")
	}
	return ""
}

func (a *API) moderateDynamic(id uint64, action string) string {
	var dyn model.UserDynamic
	if a.DB.Select("id, user_id").First(&dyn, id).Error != nil {
		return ""
	}
	switch action {
	case "takedown":
		a.DB.Delete(&dyn)
		return "动态已删除"
	case "warn":
		return a.warnAuthor(dyn.UserID, "您的动态因违规被举报，请自查内容")
	case "ban":
		return a.banAuthor(dyn.UserID, "动态违规")
	}
	return ""
}

func (a *API) moderateComment(targetID uint64, action string) string {
	// Comments are scattered across 3 tables. Try each.
	if _, err := a.modCommentAction("comments", targetID, action); err == nil {
		return a.commentActionResult(targetID, action)
	}
	if _, err := a.modCommentAction("article_comments", targetID, action); err == nil {
		return a.commentActionResult(targetID, action)
	}
	if _, err := a.modCommentAction("dynamic_comments", targetID, action); err == nil {
		return a.commentActionResult(targetID, action)
	}
	return "未找到该评论"
}

func (a *API) modCommentAction(table string, id uint64, action string) (uint64, error) {
	var userID uint64
	err := a.DB.Table(table).Select("user_id").Where("id = ?", id).Scan(&userID).Error
	if err != nil || userID == 0 {
		return 0, err
	}
	switch action {
	case "takedown":
		a.DB.Table(table).Where("id = ?", id).Delete(nil)
	case "warn":
		a.warnAuthor(userID, "您的评论因违规被举报")
	case "ban":
		a.banAuthor(userID, "评论违规")
	}
	return userID, nil
}

func (a *API) commentActionResult(id uint64, action string) string {
	switch action {
	case "takedown":
		return "评论已删除"
	case "warn":
		return "已警告评论作者"
	case "ban":
		return "已封禁评论作者"
	}
	return ""
}

func (a *API) moderateUser(id uint64, action string) string {
	switch action {
	case "takedown":
		return "对用户无法执行下架操作"
	case "warn":
		return a.warnAuthor(id, "您的账号因违规被举报，请注意行为规范")
	case "ban":
		return a.banAuthor(id, "账号违规")
	}
	return ""
}

func (a *API) warnAuthor(userID uint64, msg string) string {
	if userID == 0 {
		return ""
	}
	// Set a "warned" flag or create a notification. For now, we ensure status reflects.
	var u model.User
	if a.DB.Select("id, status, banned_reason").First(&u, userID).Error != nil {
		return ""
	}
	// Don't overwrite existing ban
	if u.Status == "banned" {
		return "作者已被封禁"
	}
	if u.Status == "active" {
		a.DB.Model(&u).Updates(map[string]interface{}{
			"banned_reason": "[警告] " + msg,
		})
	}
	return "已警告作者"
}

func (a *API) banAuthor(userID uint64, reason string) string {
	if userID == 0 {
		return ""
	}
	var u model.User
	if a.DB.Select("id, status").First(&u, userID).Error != nil {
		return ""
	}
	if u.Status == "banned" {
		return "作者已被封禁"
	}
	now := time.Now()
	a.DB.Model(&u).Updates(map[string]interface{}{
		"status":        "banned",
		"banned_reason": "[举报] " + reason,
		"banned_at":     now,
	})
	return "已封禁作者"
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
