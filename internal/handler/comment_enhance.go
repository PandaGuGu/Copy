package handler

import (
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
	"minibili/internal/storage"
)

// CommentImage represents an image attached to a comment.
type CommentImage struct {
	ID        uint64 `gorm:"primaryKey"`
	CommentID uint64 `gorm:"index;not null"`
	URL       string `gorm:"size:1024;not null"`
	CreatedAt time.Time
}

// CommentReport represents a comment report submitted by a user.
type CommentReport struct {
	ID         uint64 `gorm:"primaryKey"`
	CommentID  uint64 `gorm:"index;not null"`
	CommentType string `gorm:"size:32;not null;default:video"` // video, article, dynamic
	ReporterID uint64 `gorm:"index;not null"`
	Reason     string `gorm:"size:500;not null"`
	Category   string `gorm:"size:64;not null"` // spam, harassment, inappropriate, other
	Status     string `gorm:"size:32;not null;default:pending"` // pending, handled, dismissed
	AdminID    uint64 `gorm:"index"`
	AdminNote  string `gorm:"size:1000"`
	CreatedAt  time.Time
	UpdatedAt  time.Time
}

// PostCommentWithImageRequest is the request for posting a comment with image.
type PostCommentWithImageRequest struct {
	Content string `form:"content" binding:"required"`
}

// PostCommentWithImage posts a comment with optional image (Module 4).
// POST /api/v1/videos/:id/comments-with-image
// multipart/form-data with fields: content (text), image (file)
func (a *API) PostCommentWithImage(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || vid == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Validate video exists and is published
	var v model.Video
	if err := a.DB.First(&v, vid).Error; err != nil || v.Status != "published" {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if v.CommentsClosed {
		resp.Err(c, http.StatusForbidden, errcode.CodeCommentsClosed)
		return
	}

	// Parse multipart form
	content := strings.TrimSpace(c.PostForm("content"))
	if content == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Create comment first
	approved := !v.CommentsCurated || uid == v.UserID
	cm := model.Comment{
		VideoID:    vid,
		UserID:     uid,
		ParentID:   0,
		Level:      1,
		Content:    content,
		LikeCount:  0,
		Approved:   approved,
		IpLocation: a.resolveCommentIPLocation(c),
	}

	if err := a.DB.Create(&cm).Error; err != nil {
		a.Log.Error("create comment failed", zap.Error(err), zap.Uint64("video_id", vid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Handle image upload if present
	file, err := c.FormFile("image")
	if err == nil && file != nil {
		// Generate OSS object key
		ext := "jpg"
		if idx := strings.LastIndex(file.Filename, "."); idx >= 0 {
			ext = strings.TrimPrefix(strings.ToLower(file.Filename[idx+1:]), ".")
		}
		timestamp := time.Now().UnixNano()
		objectKey := fmt.Sprintf("comment-images/%d_%d.%s", cm.ID, timestamp, ext)

		// Upload to OSS
		if a.OSS != nil {
			src, err := file.Open()
			if err != nil {
				a.Log.Warn("open comment image failed", zap.Error(err), zap.Uint64("comment_id", cm.ID))
			} else {
				defer src.Close()
				if err := a.OSS.UploadReader(objectKey, src); err != nil {
					a.Log.Warn("upload comment image failed", zap.Error(err), zap.String("key", objectKey))
				} else {
					// Create CommentImage record
					// Store object key, resolve to full URL when serving
					commentImage := CommentImage{
						CommentID: cm.ID,
						URL:       objectKey,
					}
					if err := a.DB.Create(&commentImage).Error; err != nil {
						a.Log.Warn("create comment image record failed", zap.Error(err), zap.Uint64("comment_id", cm.ID))
					}
				}
			}
		}
	}

	if approved {
		_ = a.DB.Model(&model.Video{}).Where("id = ?", vid).
			UpdateColumn("comment_count", gorm.Expr("comment_count + ?", 1)).Error
	}

	if uid != v.UserID {
		a.notifyUploaderOnVideoComment(&v, uid, &cm)
	}

	resp.JSON(c, http.StatusCreated, errcode.CodeSuccess, gin.H{
		"id":          cm.ID,
		"approved":    cm.Approved,
		"ip_location": cm.IpLocation,
	})
}

// UploadCommentImage adds an image to an existing comment (Module 4).
// POST /api/v1/comments/:id/images
// multipart/form-data with field: image (file)
func (a *API) UploadCommentImage(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	cid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || cid == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Validate comment exists and belongs to user
	var cm model.Comment
	if err := a.DB.First(&cm, cid).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if cm.UserID != uid {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	// Parse multipart form
	file, err := c.FormFile("image")
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Generate OSS object key
	ext := "jpg"
	if idx := strings.LastIndex(file.Filename, "."); idx >= 0 {
		ext = strings.TrimPrefix(strings.ToLower(file.Filename[idx+1:]), ".")
	}
	timestamp := time.Now().UnixNano()
	objectKey := fmt.Sprintf("comment-images/%d_%d.%s", cm.ID, timestamp, ext)

	// Upload to OSS
	if a.OSS == nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	src, err := file.Open()
	if err != nil {
		a.Log.Error("open comment image failed", zap.Error(err), zap.Uint64("comment_id", cm.ID))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	defer src.Close()

	if err := a.OSS.UploadReader(objectKey, src); err != nil {
		a.Log.Error("upload comment image failed", zap.Error(err), zap.String("key", objectKey))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Create CommentImage record
	commentImage := CommentImage{
		CommentID: cm.ID,
		URL:       objectKey,
	}
	if err := a.DB.Create(&commentImage).Error; err != nil {
		a.Log.Error("create comment image record failed", zap.Error(err), zap.Uint64("comment_id", cm.ID))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Build full URL (simplified - in practice use cfg to build correct URL)
	fullURL := objectKey // Simplified; in production use storage helper

	a.Log.Info("comment image uploaded",
		zap.Uint64("user_id", uid),
		zap.Uint64("comment_id", cm.ID),
		zap.String("image_url", fullURL),
	)

	resp.JSON(c, http.StatusCreated, errcode.CodeSuccess, gin.H{
		"id":        commentImage.ID,
		"comment_id": cm.ID,
		"url":       fullURL,
	})
}

// DeleteCommentImage removes an image from a comment (Module 4).
// DELETE /api/v1/comments/:id/images/:imageId
func (a *API) DeleteCommentImage(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	cid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || cid == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	imageID, err := strconv.ParseUint(c.Param("imageId"), 10, 64)
	if err != nil || imageID == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Validate comment exists and belongs to user
	var cm model.Comment
	if err := a.DB.First(&cm, cid).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if cm.UserID != uid {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	// Validate image exists and belongs to comment
	var img CommentImage
	if err := a.DB.First(&img, imageID).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if img.CommentID != cid {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Delete from OSS
	if a.OSS != nil {
		if err := a.OSS.DeleteObject(img.URL); err != nil {
			a.Log.Warn("delete comment image from OSS failed",
				zap.Error(err),
				zap.String("key", img.URL),
			)
		}
	}

	// Delete record
	if err := a.DB.Delete(&img).Error; err != nil {
		a.Log.Error("delete comment image record failed", zap.Error(err), zap.Uint64("image_id", imageID))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{"ok": true})
}

// ListCommentImages lists images for a comment (Module 4).
// GET /api/v1/comments/:id/images
func (a *API) ListCommentImages(c *gin.Context) {
	cid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || cid == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Validate comment exists
	var cm model.Comment
	if err := a.DB.First(&cm, cid).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	// Query images
	var images []CommentImage
	if err := a.DB.Where("comment_id = ?", cid).Order("id ASC").Find(&images).Error; err != nil {
		a.Log.Error("list comment images failed", zap.Error(err), zap.Uint64("comment_id", cid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(images))
	for _, img := range images {
		// Build full URL - in practice use storage helper
		fullURL := img.URL // Simplified
		items = append(items, gin.H{
			"id":        img.ID,
			"comment_id": img.CommentID,
			"url":       fullURL,
		})
	}

	resp.OK(c, gin.H{
		"items": items,
		"total": len(items),
	})
}

// CommentSortConfig represents available sort/filter options for comments.
type CommentSortConfig struct {
	SortOptions []string `json:"sort_options"`
	FilterOptions []string `json:"filter_options"`
}

// GetCommentSortOptions returns available sort/filter options for comments (Module 4).
// GET /api/v1/videos/:id/comments/config
func (a *API) GetCommentSortOptions(c *gin.Context) {
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || vid == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Validate video exists
	var v model.Video
	if err := a.DB.First(&v, vid).Error; err != nil || v.Status != "published" {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	config := gin.H{
		"sort_options": []string{"hot", "time", "latest"},
		"filter_options": []string{"all", "with_image"},
	}

	resp.OK(c, config)
}

// ReportCommentRequest is the request body for reporting a comment.
type ReportCommentRequest struct {
	Reason   string `json:"reason" binding:"required"`
	Category string `json:"category" binding:"required"` // spam, harassment, inappropriate, other
}

// ReportComment submits a report for a comment (Module 4).
// POST /api/v1/comments/:id/report
func (a *API) ReportComment(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	cid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || cid == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req ReportCommentRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Validate comment exists
	var cm model.Comment
	if err := a.DB.First(&cm, cid).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	// Check if user has already reported this comment
	var existingReport CommentReport
	if err := a.DB.Where("comment_id = ? AND reporter_id = ?", cid, uid).
		First(&existingReport).Error; err == nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Create report
	report := CommentReport{
		CommentID:  cid,
		CommentType: "video",
		ReporterID: uid,
		Reason:     req.Reason,
		Category:   req.Category,
		Status:     "pending",
	}

	if err := a.DB.Create(&report).Error; err != nil {
		a.Log.Error("create comment report failed", zap.Error(err), zap.Uint64("comment_id", cid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("comment reported",
		zap.Uint64("reporter_id", uid),
		zap.Uint64("comment_id", cid),
		zap.String("category", req.Category),
	)

	resp.OK(c, gin.H{
		"report_id": report.ID,
		"status":    "pending",
	})
}

// AdminListCommentReports lists comment reports for admin review (Module 4).
// GET /api/v1/admin/comment-reports
func (a *API) AdminListCommentReports(c *gin.Context) {
	// Check admin permission (simplified - in practice use admin middleware)
	adminID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	// TODO: Add admin role check here
	_ = adminID

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	status := strings.TrimSpace(c.Query("status")) // pending, handled, dismissed

	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tx := a.DB.Model(&CommentReport{})
	if status != "" {
		tx = tx.Where("status = ?", status)
	}

	var total int64
	if err := tx.Count(&total).Error; err != nil {
		a.Log.Error("count comment reports failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	var reports []CommentReport
	if err := tx.Order("created_at DESC").
		Offset((page - 1) * pageSize).
		Limit(pageSize).
		Find(&reports).Error; err != nil {
		a.Log.Error("list comment reports failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(reports))
	for _, r := range reports {
		items = append(items, gin.H{
			"id":          r.ID,
			"comment_id":  r.CommentID,
			"comment_type": r.CommentType,
			"reporter_id": r.ReporterID,
			"reason":      r.Reason,
			"category":    r.Category,
			"status":      r.Status,
			"admin_id":    r.AdminID,
			"admin_note":  r.AdminNote,
			"created_at":  r.CreatedAt.Format("2006-01-02 15:04:05"),
		})
	}

	resp.OK(c, gin.H{
		"items":      items,
		"total":      total,
		"page":       page,
		"page_size":  pageSize,
	})
}

// AdminHandleCommentReportRequest is the request body for handling a comment report.
type AdminHandleCommentReportRequest struct {
	Action   string `json:"action" binding:"required"` // approve, dismiss
	AdminNote string `json:"admin_note"`
}

// AdminHandleCommentReport handles a comment report (Module 4).
// POST /api/v1/admin/comment-reports/:id/handle
func (a *API) AdminHandleCommentReport(c *gin.Context) {
	// Check admin permission (simplified)
	adminID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	reportID, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || reportID == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req AdminHandleCommentReportRequest
	if err := c.ShouldBindJSON(&req).Error; err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Validate action
	if req.Action != "approve" && req.Action != "dismiss" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Get report
	var report CommentReport
	if err := a.DB.First(&report, reportID).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	// Update report status
	newStatus := "handled"
	if req.Action == "dismiss" {
		newStatus = "dismissed"
	}

	if err := a.DB.Model(&report).Updates(map[string]interface{}{
		"status":     newStatus,
		"admin_id":   adminID,
		"admin_note": req.AdminNote,
	}).Error; err != nil {
		a.Log.Error("update comment report failed", zap.Error(err), zap.Uint64("report_id", reportID))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// If approved, delete the comment
	if req.Action == "approve" {
		if err := a.DB.Delete(&model.Comment{}, report.CommentID).Error; err != nil {
			a.Log.Error("delete reported comment failed",
				zap.Error(err),
				zap.Uint64("comment_id", report.CommentID),
			)
		}
	}

	a.Log.Info("comment report handled",
		zap.Uint64("admin_id", adminID),
		zap.Uint64("report_id", reportID),
		zap.String("action", req.Action),
	)

	resp.OK(c, gin.H{
		"report_id": reportID,
		"status":    newStatus,
		"action":    req.Action,
	})
}

// Helper to resolve OSS URL (simplified version)
func (a *API) resolveOSSURL(objectKey string) string {
	if a.OSS == nil || objectKey == "" {
		return objectKey
	}
	// In practice, build full URL from cfg
	return objectKey
}

// Ensure storage.OSS is used for lint
var _ storage.OSS
