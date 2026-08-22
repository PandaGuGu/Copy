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
	"minibili/internal/pkg/statemachine"
	"minibili/internal/pkg/usercap"
)

// ---------- User side: 申诉 ----------

// PostAppeal POST /api/v1/appeals (authenticated)
// Submits an appeal against a governance action (ban / takedown / warn).
func (a *API) PostAppeal(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	var req struct {
		TargetType     string `json:"target_type"` // user/video/article/dynamic/comment/capability
		TargetID       uint64 `json:"target_id"`
		ReasonType     string `json:"reason_type"` // ban/takedown/warn
		Content        string `json:"content"`
		EvidenceURLs   string `json:"evidence_urls"`
		SourceReportID uint64 `json:"source_report_id"`
		Capability     string `json:"capability"` // required when target_type == "capability"
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.TargetType = strings.TrimSpace(req.TargetType)
	req.ReasonType = strings.TrimSpace(req.ReasonType)
	req.Content = strings.TrimSpace(req.Content)
	req.EvidenceURLs = strings.TrimSpace(req.EvidenceURLs)
	req.Capability = strings.TrimSpace(req.Capability)

	if !containsString(model.AppealTargetTypes, req.TargetType) ||
		!containsString(model.AppealReasonTypes, req.ReasonType) ||
		req.TargetID == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.TargetType == "capability" && !usercap.Valid(req.Capability) {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.Content == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if len([]rune(req.Content)) > 2000 {
		req.Content = string([]rune(req.Content)[:2000])
	}
	if len([]rune(req.EvidenceURLs)) > 2000 {
		req.EvidenceURLs = string([]rune(req.EvidenceURLs)[:2000])
	}

	// Ownership check: a user may only appeal their own account or content.
	if !a.userOwnsTarget(userID, req.TargetType, req.TargetID) {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	// Dedupe: no open pending appeal for the same subject.
	var dup int64
	a.DB.Model(&model.Appeal{}).
		Where("user_id = ? AND target_type = ? AND target_id = ? AND status = 'pending'",
			userID, req.TargetType, req.TargetID).
		Count(&dup)
	if dup > 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	ap := model.Appeal{
		UserID:         userID,
		TargetType:     req.TargetType,
		TargetID:       req.TargetID,
		ReasonType:     req.ReasonType,
		SourceReportID: req.SourceReportID,
		Content:        req.Content,
		EvidenceURLs:   req.EvidenceURLs,
		Capability:     req.Capability,
		Status:         "pending",
		CreatedAt:      time.Now(),
	}
	if err := a.DB.Create(&ap).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Notify admins of the new appeal.
	a.notifyAppealAdmin(&ap)

	resp.OK(c, gin.H{"id": ap.ID, "status": ap.Status})
}

// ListMyAppeals GET /api/v1/appeals/me
func (a *API) ListMyAppeals(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	status := strings.TrimSpace(c.Query("status"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tx := a.DB.Model(&model.Appeal{}).Where("user_id = ?", userID)
	if status != "" {
		tx = tx.Where("status = ?", status)
	}

	var total int64
	tx.Count(&total)

	var appeals []model.Appeal
	tx.Order("created_at DESC").Offset((page - 1) * pageSize).Limit(pageSize).Find(&appeals)

	items := make([]gin.H, 0, len(appeals))
	for _, ap := range appeals {
		items = append(items, appealView(ap))
	}

	resp.OK(c, gin.H{"items": items, "total": total, "page": page, "page_size": pageSize})
}

// GetAppeal GET /api/v1/appeals/:id (owner only)
func (a *API) GetAppeal(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var ap model.Appeal
	if err := a.DB.First(&ap, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if ap.UserID != userID {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	resp.OK(c, appealView(ap))
}

// ---------- Admin side ----------

// AdminListAppeals GET /api/v1/admin/appeals
func (a *API) AdminListAppeals(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	status := strings.TrimSpace(c.Query("status"))      // pending/approved/rejected/empty=all
	target := strings.TrimSpace(c.Query("target"))      // user/video/article/dynamic/comment
	reason := strings.TrimSpace(c.Query("reason_type")) // ban/takedown/warn

	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tx := a.DB.Model(&model.Appeal{})
	if status != "" {
		tx = tx.Where("status = ?", status)
	}
	if target != "" {
		tx = tx.Where("target_type = ?", target)
	}
	if reason != "" {
		tx = tx.Where("reason_type = ?", reason)
	}

	var total int64
	tx.Count(&total)

	var appeals []model.Appeal
	tx.Order("created_at DESC").Offset((page - 1) * pageSize).Limit(pageSize).Find(&appeals)

	uidSet := make(map[uint64]bool)
	for _, ap := range appeals {
		uidSet[ap.UserID] = true
	}
	userBriefs := loadUserBriefs(a.DB, uidSet)

	type item struct {
		model.Appeal
		Applicant gin.H `json:"applicant"`
	}
	items := make([]item, 0, len(appeals))
	for _, ap := range appeals {
		it := item{Appeal: ap, Applicant: userBriefs[ap.UserID]}
		items = append(items, it)
	}

	// Stats
	var pendingC, approvedC, rejectedC int64
	a.DB.Model(&model.Appeal{}).Where("status = 'pending'").Count(&pendingC)
	a.DB.Model(&model.Appeal{}).Where("status = 'approved'").Count(&approvedC)
	a.DB.Model(&model.Appeal{}).Where("status = 'rejected'").Count(&rejectedC)

	resp.OK(c, gin.H{
		"items":          items,
		"total":          total,
		"page":           page,
		"page_size":      pageSize,
		"pending_count":  pendingC,
		"approved_count": approvedC,
		"rejected_count": rejectedC,
	})
}

// AdminHandleAppeal POST /api/v1/admin/appeals/:id/handle
// action: "approve" (restore target) / "reject" (keep decision).
func (a *API) AdminHandleAppeal(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req struct {
		Action    string `json:"action"` // approve / reject
		AdminNote string `json:"admin_note"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Action = strings.TrimSpace(req.Action)
	if req.Action != "approve" && req.Action != "reject" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.AdminNote = strings.TrimSpace(req.AdminNote)
	if req.Action == "reject" && req.AdminNote == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if len([]rune(req.AdminNote)) > 500 {
		req.AdminNote = string([]rune(req.AdminNote)[:500])
	}

	adminID, _ := middleware.AdminID(c)

	var ap model.Appeal
	if err := a.DB.First(&ap, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if ap.Status != "pending" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	newStatus := "approved"
	if req.Action == "reject" {
		newStatus = "rejected"
	}
	if err := statemachine.Appeal.Transition(ap.Status, newStatus, "admin"); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Approve → attempt to reverse the governance action (restore target).
	restoreNote := ""
	if req.Action == "approve" {
		restoreNote = a.restoreAppealTarget(&ap)
	}
	if restoreNote != "" && req.AdminNote != "" {
		req.AdminNote += " | " + restoreNote
	} else if restoreNote != "" {
		req.AdminNote = restoreNote
	}

	now := time.Now()
	if err := a.DB.Model(&ap).Updates(map[string]interface{}{
		"status":     newStatus,
		"admin_note": req.AdminNote,
		"handled_by": adminID,
		"handled_at": now,
	}).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Notify the appealing user of the decision.
	a.notifyAppealResult(&ap, newStatus)

	resp.OK(c, gin.H{"id": ap.ID, "status": newStatus, "restore_note": restoreNote})
}

// ---------- helpers ----------

func appealView(ap model.Appeal) gin.H {
	return gin.H{
		"id":               ap.ID,
		"target_type":      ap.TargetType,
		"target_id":        ap.TargetID,
		"reason_type":      ap.ReasonType,
		"content":          ap.Content,
		"evidence_urls":    ap.EvidenceURLs,
		"source_report_id": ap.SourceReportID,
		"capability":       ap.Capability,
		"capability_label": usercap.Label(ap.Capability),
		"status":           ap.Status,
		"status_label":     model.AppealStatusLabel(ap.Status),
		"admin_note":       ap.AdminNote,
		"created_at":       ap.CreatedAt,
		"handled_at":       ap.HandledAt,
	}
}

// userOwnsTarget verifies the appealing user owns the account/content in question.
func (a *API) userOwnsTarget(userID uint64, targetType string, targetID uint64) bool {
	switch targetType {
	case "user", "capability":
		return userID == targetID
	case "video":
		var v model.Video
		return a.DB.Select("user_id").First(&v, targetID).Error == nil && v.UserID == userID
	case "article":
		var ar model.Article
		return a.DB.Select("user_id").First(&ar, targetID).Error == nil && ar.UserID == userID
	case "dynamic":
		var d model.UserDynamic
		return a.DB.Select("user_id").First(&d, targetID).Error == nil && d.UserID == userID
	case "comment":
		return a.ownsComment(userID, targetID)
	}
	return false
}

// ownsComment checks ownership across the three comment tables.
func (a *API) ownsComment(userID, id uint64) bool {
	for _, tbl := range []string{"comments", "article_comments", "dynamic_comments"} {
		var uid uint64
		if a.DB.Table(tbl).Select("user_id").Where("id = ?", id).Scan(&uid).Error == nil && uid == userID {
			return true
		}
	}
	return false
}

// restoreAppealTarget reverses a governance action on approve. Returns a note.
func (a *API) restoreAppealTarget(ap *model.Appeal) string {
	if ap == nil {
		return ""
	}
	switch ap.TargetType {
	case "user":
		return a.restoreUser(ap.TargetID)
	case "capability":
		note := a.removeCapRestriction(ap.TargetID, ap.Capability)
		if strings.HasPrefix(note, "未找到") || strings.HasPrefix(note, "查询失败") {
			return "该能力限制不存在或已恢复"
		}
		middleware.InvalidateUserCapCache(a.Redis, ap.TargetID)
		return note
	case "video":
		return a.restoreVideo(ap.TargetID)
	case "article":
		return a.restoreArticle(ap.TargetID)
	case "dynamic":
		return a.restoreDynamic(ap.TargetID)
	case "comment":
		return "评论已被物理删除，无法恢复（建议人工复核原举报）"
	}
	return ""
}

func (a *API) restoreUser(id uint64) string {
	var u model.User
	if a.DB.Select("id, status").First(&u, id).Error != nil {
		return "目标用户不存在"
	}
	if u.Status != "banned" {
		return "该账号未处于封禁状态"
	}
	if !statemachine.User.Can("banned", "active") {
		return "账号状态不允许解封"
	}
	if err := a.DB.Model(&u).Updates(map[string]interface{}{
		"status": "active", "banned_reason": "", "banned_at": nil, "ban_expires_at": nil,
	}).Error; err != nil {
		return "解封失败"
	}
	return "已解封用户账号"
}

func (a *API) restoreVideo(id uint64) string {
	var v model.Video
	if a.DB.Select("id, status").First(&v, id).Error != nil {
		return "视频不存在"
	}
	switch v.Status {
	case "deleted", "takedown", "pending_review":
		if err := a.DB.Model(&v).Updates(map[string]interface{}{
			"status": "published", "fail_reason": "",
		}).Error; err != nil {
			return "恢复视频失败"
		}
		return "已恢复视频可见"
	}
	return "视频当前状态无需恢复"
}

func (a *API) restoreArticle(id uint64) string {
	var ar model.Article
	if a.DB.Select("id, status").First(&ar, id).Error != nil {
		return "文章不存在"
	}
	switch ar.Status {
	case "deleted", "takedown", "pending_review":
		if err := a.DB.Model(&ar).Updates(map[string]interface{}{
			"status": "published", "fail_reason": "",
		}).Error; err != nil {
			return "恢复文章失败"
		}
		return "已恢复文章可见"
	}
	return "文章当前状态无需恢复"
}

func (a *API) restoreDynamic(id uint64) string {
	var d model.UserDynamic
	if a.DB.Select("id").First(&d, id).Error != nil {
		return "动态已被物理删除，无法恢复"
	}
	return "动态可正常访问"
}

// notifyAppealAdmin pushes an admin alert when a new appeal arrives.
func (a *API) notifyAppealAdmin(ap *model.Appeal) {
	_ = a.DB.Create(&model.NotificationRecord{
		RecipientID: 0, RecipientType: "admin",
		Channel: "in_app", Title: "新增申诉：" + model.AppealStatusLabel(ap.ReasonType) + "申诉",
		Content: "用户 #" + strconv.FormatUint(ap.UserID, 10) + " 针对 " + ap.TargetType +
			"#" + strconv.FormatUint(ap.TargetID, 10) + " 提交申诉：" + ap.Content,
		RelatedType: "appeal", RelatedID: ap.ID, Status: "pending",
	})
	if a.ChatHub != nil {
		a.ChatHub.PushJSON(0, gin.H{
			"type": "admin_alert",
			"data": gin.H{"kind": "appeal", "appeal_id": ap.ID, "target_type": ap.TargetType, "target_id": ap.TargetID},
		})
	}
}

// notifyAppealResult notifies the user of an appeal decision.
func (a *API) notifyAppealResult(ap *model.Appeal, status string) {
	msg := "您的申诉已被采纳，相关处罚已解除。"
	if status == "rejected" {
		msg = "您的申诉未通过。"
	}
	if ap.AdminNote != "" {
		msg += " 管理员备注：" + ap.AdminNote
	}
	_ = a.DB.Create(&model.NotificationRecord{
		RecipientID: ap.UserID, RecipientType: "user",
		Channel: "in_app", Title: "申诉处理结果",
		Content:     msg,
		RelatedType: "appeal", RelatedID: ap.ID, Status: "pending",
	})
}

func containsString(set []string, v string) bool {
	for _, s := range set {
		if s == v {
			return true
		}
	}
	return false
}
