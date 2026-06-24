package handler

import (
	"encoding/json"
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
)

// ──────────────────────────────────────────────
// Copyright complaint admin handlers
// ──────────────────────────────────────────────

type copyrightComplaintItem struct {
	ID             uint64     `json:"id"`
	ComplainantID  uint64     `json:"complainant_id"`
	Complainant    gin.H      `json:"complainant,omitempty"`
	RelatedID      uint64     `json:"related_id"`
	RelatedType    string     `json:"related_type"`
	Description    string     `json:"description"`
	EvidenceURLs   []string   `json:"evidence_urls"`
	Status         string     `json:"status"`
	HandlerID      *uint64    `json:"handler_id"`
	HandlerComment string     `json:"handler_comment"`
	TakedownAt     *time.Time `json:"takedown_at"`
	RestoredAt     *time.Time `json:"restored_at"`
	CreatedAt      time.Time  `json:"created_at"`
	UpdatedAt      time.Time  `json:"updated_at"`
}

func copyrightComplaintToItem(cp *model.CopyrightComplaint) copyrightComplaintItem {
	item := copyrightComplaintItem{
		ID:             cp.ID,
		ComplainantID:  cp.ComplainantID,
		RelatedID:      cp.RelatedID,
		RelatedType:    cp.RelatedType,
		Description:    cp.Description,
		Status:         cp.Status,
		HandlerID:      cp.HandlerID,
		HandlerComment: cp.HandlerComment,
		TakedownAt:     cp.TakedownAt,
		RestoredAt:     cp.RestoredAt,
		CreatedAt:      cp.CreatedAt,
		UpdatedAt:      cp.UpdatedAt,
	}
	if cp.EvidenceURLs != "" {
		_ = json.Unmarshal([]byte(cp.EvidenceURLs), &item.EvidenceURLs)
	}
	if item.EvidenceURLs == nil {
		item.EvidenceURLs = []string{}
	}
	return item
}

func (a *API) loadComplainantBrief(uid uint64) gin.H {
	var u model.User
	if err := a.DB.Select("id, username, nickname, avatar_url").First(&u, uid).Error; err != nil {
		return gin.H{"id": uid}
	}
	return gin.H{
		"id":         u.ID,
		"username":   u.Username,
		"nickname":   u.Nickname,
		"avatar_url": u.AvatarURL,
	}
}

// AdminListCopyrightComplaints GET /admin/copyright/complaints
func (a *API) AdminListCopyrightComplaints(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 50 {
		pageSize = 20
	}
	statusQ := strings.TrimSpace(c.Query("status"))
	relatedTypeQ := strings.TrimSpace(c.Query("related_type"))

	q := a.DB.Model(&model.CopyrightComplaint{})
	if statusQ != "" {
		q = q.Where("status = ?", statusQ)
	}
	if relatedTypeQ != "" {
		q = q.Where("related_type = ?", relatedTypeQ)
	}

	var total int64
	if err := q.Count(&total).Error; err != nil {
		a.Log.Error("count copyright complaints failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	offset := (page - 1) * pageSize
	var rows []model.CopyrightComplaint
	if err := q.Order("created_at DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		a.Log.Error("list copyright complaints failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		item := copyrightComplaintToItem(&rows[i])
		h := gin.H{
			"id":              item.ID,
			"complainant_id":  item.ComplainantID,
			"related_id":      item.RelatedID,
			"related_type":    item.RelatedType,
			"description":     item.Description,
			"evidence_urls":   item.EvidenceURLs,
			"status":          item.Status,
			"handler_id":      item.HandlerID,
			"handler_comment": item.HandlerComment,
			"takedown_at":     item.TakedownAt,
			"restored_at":     item.RestoredAt,
			"created_at":      item.CreatedAt,
			"updated_at":      item.UpdatedAt,
			"complainant":     a.loadComplainantBrief(item.ComplainantID),
		}
		items = append(items, h)
	}

	resp.OK(c, gin.H{
		"items":      items,
		"total":      total,
		"page":       page,
		"page_size":  pageSize,
	})
}

// AdminGetCopyrightComplaint GET /admin/copyright/complaints/:id
func (a *API) AdminGetCopyrightComplaint(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var cp model.CopyrightComplaint
	if err := a.DB.First(&cp, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	item := copyrightComplaintToItem(&cp)
	resp.OK(c, gin.H{
		"id":              item.ID,
		"complainant_id":  item.ComplainantID,
		"complainant":     a.loadComplainantBrief(item.ComplainantID),
		"related_id":      item.RelatedID,
		"related_type":    item.RelatedType,
		"description":     item.Description,
		"evidence_urls":   item.EvidenceURLs,
		"status":          item.Status,
		"handler_id":      item.HandlerID,
		"handler_comment": item.HandlerComment,
		"takedown_at":     item.TakedownAt,
		"restored_at":     item.RestoredAt,
		"created_at":      item.CreatedAt,
		"updated_at":      item.UpdatedAt,
	})
}

// AdminAcceptCopyrightComplaint POST /admin/copyright/complaints/:id/accept
func (a *API) AdminAcceptCopyrightComplaint(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	adminID, _ := middleware.AdminID(c)

	var cp model.CopyrightComplaint
	if err := a.DB.First(&cp, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if cp.Status != "pending" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	updates := map[string]interface{}{
		"status":     "accepted",
		"handler_id": adminID,
		"updated_at": now,
	}
	if err := a.DB.Model(&cp).Updates(updates).Error; err != nil {
		a.Log.Error("accept copyright complaint failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Trigger takedown: update video/article status to "takedown"
	_ = a.takedownRelatedContent(cp.RelatedID, cp.RelatedType)

	a.Log.Info("copyright complaint accepted",
		zap.Uint64("complaint_id", id),
		zap.Uint64("admin_id", adminID))
	resp.OK(c, gin.H{"status": "accepted"})
}

// AdminRejectCopyrightComplaint POST /admin/copyright/complaints/:id/reject
func (a *API) AdminRejectCopyrightComplaint(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	adminID, _ := middleware.AdminID(c)

	var body struct {
		Comment string `json:"comment"`
	}
	_ = c.ShouldBindJSON(&body)

	var cp model.CopyrightComplaint
	if err := a.DB.First(&cp, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if cp.Status != "pending" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	updates := map[string]interface{}{
		"status":          "rejected",
		"handler_id":      adminID,
		"handler_comment": strings.TrimSpace(body.Comment),
		"updated_at":      now,
	}
	if err := a.DB.Model(&cp).Updates(updates).Error; err != nil {
		a.Log.Error("reject copyright complaint failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("copyright complaint rejected",
		zap.Uint64("complaint_id", id),
		zap.Uint64("admin_id", adminID))
	resp.OK(c, gin.H{"status": "rejected"})
}

// AdminTakedownContent POST /admin/copyright/complaints/:id/takedown
func (a *API) AdminTakedownContent(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var cp model.CopyrightComplaint
	if err := a.DB.First(&cp, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	now := time.Now()
	if err := a.takedownRelatedContent(cp.RelatedID, cp.RelatedType); err != nil {
		a.Log.Error("takedown content failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.DB.Model(&cp).Updates(map[string]interface{}{
		"status":      "takedown",
		"takedown_at": now,
		"updated_at":  now,
	})

	a.Log.Info("content takedown executed",
		zap.Uint64("complaint_id", id),
		zap.Uint64("related_id", cp.RelatedID),
		zap.String("related_type", cp.RelatedType))
	resp.OK(c, gin.H{"status": "takedown"})
}

// AdminRestoreContent POST /admin/copyright/complaints/:id/restore
func (a *API) AdminRestoreContent(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var cp model.CopyrightComplaint
	if err := a.DB.First(&cp, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	now := time.Now()
	if err := a.restoreRelatedContent(cp.RelatedID, cp.RelatedType); err != nil {
		a.Log.Error("restore content failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.DB.Model(&cp).Updates(map[string]interface{}{
		"status":      "restored",
		"restored_at": now,
		"updated_at":  now,
	})

	a.Log.Info("content restored",
		zap.Uint64("complaint_id", id),
		zap.Uint64("related_id", cp.RelatedID),
		zap.String("related_type", cp.RelatedType))
	resp.OK(c, gin.H{"status": "restored"})
}

// ──────────────────────────────────────────────
// User-facing copyright complaint handlers
// ──────────────────────────────────────────────

type postCopyrightComplaintReq struct {
	RelatedID    uint64   `json:"related_id"    binding:"required"`
	RelatedType  string   `json:"related_type"  binding:"required"`
	Description  string   `json:"description"   binding:"required"`
	EvidenceURLs []string `json:"evidence_urls"`
}

// PostCopyrightComplaint POST /copyright/complaints
func (a *API) PostCopyrightComplaint(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	var req postCopyrightComplaintReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.RelatedType != "video" && req.RelatedType != "article" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	evidenceJSON := "[]"
	if len(req.EvidenceURLs) > 0 {
		b, _ := json.Marshal(req.EvidenceURLs)
		evidenceJSON = string(b)
	}

	cp := model.CopyrightComplaint{
		ComplainantID: userID,
		RelatedID:     req.RelatedID,
		RelatedType:   req.RelatedType,
		Description:   strings.TrimSpace(req.Description),
		EvidenceURLs:  evidenceJSON,
		Status:        "pending",
		CreatedAt:     time.Now(),
		UpdatedAt:     time.Now(),
	}
	if err := a.DB.Create(&cp).Error; err != nil {
		a.Log.Error("create copyright complaint failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("copyright complaint filed",
		zap.Uint64("complaint_id", cp.ID),
		zap.Uint64("user_id", userID))
	resp.OK(c, gin.H{"id": cp.ID, "status": "pending"})
}

// ListMyCopyrightComplaints GET /users/me/copyright/complaints
func (a *API) ListMyCopyrightComplaints(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 50 {
		pageSize = 20
	}

	q := a.DB.Model(&model.CopyrightComplaint{}).Where("complainant_id = ?", userID)

	var total int64
	if err := q.Count(&total).Error; err != nil {
		a.Log.Error("count user copyright complaints failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	offset := (page - 1) * pageSize
	var rows []model.CopyrightComplaint
	if err := q.Order("created_at DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		a.Log.Error("list user copyright complaints failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		item := copyrightComplaintToItem(&rows[i])
		items = append(items, gin.H{
			"id":              item.ID,
			"related_id":      item.RelatedID,
			"related_type":    item.RelatedType,
			"description":     item.Description,
			"evidence_urls":   item.EvidenceURLs,
			"status":          item.Status,
			"handler_comment": item.HandlerComment,
			"created_at":      item.CreatedAt,
			"updated_at":      item.UpdatedAt,
		})
	}

	resp.OK(c, gin.H{
		"items":     items,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	})
}

// ──────────────────────────────────────────────
// Internal helpers
// ──────────────────────────────────────────────

// takedownRelatedContent sets the content status to "takedown".
func (a *API) takedownRelatedContent(relatedID uint64, relatedType string) error {
	switch relatedType {
	case "video":
		return a.DB.Model(&model.Video{}).Where("id = ?", relatedID).
			Update("status", "takedown").Error
	case "article":
		return a.DB.Model(&model.Article{}).Where("id = ?", relatedID).
			Update("status", "takedown").Error
	default:
		return nil
	}
}

// restoreRelatedContent sets the content status back to "published".
func (a *API) restoreRelatedContent(relatedID uint64, relatedType string) error {
	switch relatedType {
	case "video":
		return a.DB.Model(&model.Video{}).Where("id = ?", relatedID).
			Update("status", "published").Error
	case "article":
		return a.DB.Model(&model.Article{}).Where("id = ?", relatedID).
			Update("status", "published").Error
	default:
		return nil
	}
}
