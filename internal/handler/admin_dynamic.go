package handler

import (
	"fmt"
	"net/http"
	"sort"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

func adminDynamicToJSON(d *model.UserDynamic, authorName string) gin.H {
	imgs := parseDynamicImagesJSON(d.ImagesJSON)
	if imgs == nil {
		imgs = []string{}
	}
	cover := ""
	if len(imgs) > 0 {
		cover = imgs[0]
	}
	dynType := d.Type
	if dynType == "" {
		dynType = "image"
	}
	return gin.H{
		"id":            d.ID,
		"title":         d.Title,
		"content":       d.Content,
		"images":        imgs,
		"cover_url":     cover,
		"user_id":       d.UserID,
		"uploader_name": authorName,
		"type":          dynType,
		"like_count":    d.LikeCount,
		"comment_count": d.CommentCount,
		"created_at":    d.CreatedAt,
	}
}

// AdminListDynamics GET /api/v1/admin/dynamics — 支持 user_id / type / q 过滤。
func (a *API) AdminListDynamics(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 50 {
		pageSize = 20
	}
	q := strings.TrimSpace(c.Query("q"))
	uidStr := strings.TrimSpace(c.Query("user_id"))
	dynType := strings.TrimSpace(c.Query("type"))

	dbq := a.DB.Model(&model.UserDynamic{})
	if q != "" {
		dbq = dbq.Where("title LIKE ? OR content LIKE ?", "%"+q+"%", "%"+q+"%")
	}
	if uidStr != "" {
		if uid, err := strconv.ParseUint(uidStr, 10, 64); err == nil && uid > 0 {
			dbq = dbq.Where("user_id = ?", uid)
		}
	}
	if dynType != "" && (dynType == "text" || dynType == "image") {
		dbq = dbq.Where("type = ?", dynType)
	}
	var total int64
	if err := dbq.Count(&total).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	totalPages := int((total + int64(pageSize) - 1) / int64(pageSize))
	if totalPages < 1 {
		totalPages = 1
	}
	if page > totalPages {
		page = totalPages
	}
	offset := (page - 1) * pageSize
	var rows []model.UserDynamic
	if err := dbq.Order("created_at DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	uids := make([]uint64, 0, len(rows))
	for i := range rows {
		uids = append(uids, rows[i].UserID)
	}
	names := map[uint64]string{}
	if len(uids) > 0 {
		var users []model.User
		_ = a.DB.Where("id IN ?", uids).Find(&users).Error
		for i := range users {
			names[users[i].ID] = model.DisplayUsername(&users[i])
			if users[i].Nickname != "" && !model.IsUserAnonymized(&users[i]) {
				names[users[i].ID] = strings.TrimSpace(users[i].Nickname)
			}
		}
	}
	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, adminDynamicToJSON(&rows[i], names[rows[i].UserID]))
	}
	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}

// AdminGetDynamic GET /api/v1/admin/dynamics/:id
func (a *API) AdminGetDynamic(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var dyn model.UserDynamic
	if err := a.DB.First(&dyn, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	var u model.User
	_ = a.DB.First(&u, dyn.UserID).Error
	name := model.DisplayUsername(&u)
	if u.Nickname != "" && !model.IsUserAnonymized(&u) {
		name = strings.TrimSpace(u.Nickname)
	}
	resp.OK(c, adminDynamicToJSON(&dyn, name))
}

// AdminDeleteDynamic POST /api/v1/admin/dynamics/:id/delete 或 DELETE /api/v1/admin/dynamics/:id
func (a *API) AdminDeleteDynamic(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var dyn model.UserDynamic
	if err := a.DB.First(&dyn, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		return deleteUserDynamicCascade(tx, id)
	}); err != nil {
		a.Log.Error("admin delete dynamic", zap.Error(err), zap.Uint64("dynamic_id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	purgeDynamicOSSObjects(a.Cfg, a.OSS, a.Log, dyn)
	a.recordAudit(c, adminID, "delete", "dynamic", id,
		fmt.Sprintf(`{"title":"%s","user_id":%d,"type":"%s"}`, dyn.Title, dyn.UserID, dyn.Type))
	a.Log.Info("admin deleted dynamic",
		zap.Uint64("dynamic_id", id),
		zap.Uint64("admin_id", adminID),
		zap.Uint64("user_id", dyn.UserID),
	)
	resp.OK(c, gin.H{"ok": true})
}

// unifiedDynRow is the common shape for the unified dynamics feed.
type unifiedDynRow struct {
	ID           uint64
	Kind         string
	UserID       uint64
	Title        string
	Content      string
	CoverURL     string
	LikeCount    uint64
	CommentCount uint64
	CreatedAt    string
}

// AdminListUnifiedDynamics GET /api/v1/admin/dynamics/unified
// Merges videos (published), articles (published), and user_dynamics into one feed.
// Query: page, page_size, user_id, kind(video|article|image|text), q.
func (a *API) AdminListUnifiedDynamics(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 50 {
		pageSize = 20
	}
	q := strings.TrimSpace(c.Query("q"))
	uidStr := strings.TrimSpace(c.Query("user_id"))
	kind := strings.TrimSpace(c.Query("kind"))

	var filterUID uint64
	if uidStr != "" {
		if v, err := strconv.ParseUint(uidStr, 10, 64); err == nil && v > 0 {
			filterUID = v
		}
	}

	entries := make([]unifiedDynRow, 0)

	// ── 1. Videos (published only) ──
	if kind == "" || kind == "video" {
		vq := a.DB.Model(&model.Video{}).Where("status = ?", "published")
		if filterUID > 0 {
			vq = vq.Where("user_id = ?", filterUID)
		}
		if q != "" {
			vq = vq.Where("title LIKE ? OR description LIKE ?", "%"+q+"%", "%"+q+"%")
		}
		var videos []model.Video
		vq.Find(&videos)
		for i := range videos {
			v := &videos[i]
			content := v.Description
			if len(content) > 200 {
				content = content[:200]
			}
			entries = append(entries, unifiedDynRow{
				ID: v.ID, Kind: "video", UserID: v.UserID,
				Title: v.Title, Content: content, CoverURL: v.CoverURL,
				LikeCount: v.LikeCount, CommentCount: v.CommentCount,
				CreatedAt: v.CreatedAt.Format("2006-01-02 15:04:05"),
			})
		}
	}

	// ── 2. Articles (published only) ──
	if kind == "" || kind == "article" {
		aq := a.DB.Model(&model.Article{}).Where("status = ?", "published")
		if filterUID > 0 {
			aq = aq.Where("user_id = ?", filterUID)
		}
		if q != "" {
			aq = aq.Where("title LIKE ?", "%"+q+"%")
		}
		var articles []model.Article
		aq.Find(&articles)
		for i := range articles {
			a2 := &articles[i]
			content := ""
			if len(a2.BodyMD) > 200 {
				content = a2.BodyMD[:200]
			} else {
				content = a2.BodyMD
			}
			entries = append(entries, unifiedDynRow{
				ID: a2.ID, Kind: "article", UserID: a2.UserID,
				Title: a2.Title, Content: content, CoverURL: a2.CoverURL,
				LikeCount: 0, CommentCount: a2.CommentCount,
				CreatedAt: a2.CreatedAt.Format("2006-01-02 15:04:05"),
			})
		}
	}

	// ── 3. UserDynamics (image/text posts) ──
	if kind == "" || kind == "image" || kind == "text" {
		dq := a.DB.Model(&model.UserDynamic{})
		if filterUID > 0 {
			dq = dq.Where("user_id = ?", filterUID)
		}
		if kind == "image" {
			dq = dq.Where("type = ?", "image")
		} else if kind == "text" {
			dq = dq.Where("type = ?", "text")
		}
		if q != "" {
			dq = dq.Where("title LIKE ? OR content LIKE ?", "%"+q+"%", "%"+q+"%")
		}
		var dynamics []model.UserDynamic
		dq.Find(&dynamics)
		for i := range dynamics {
			d := &dynamics[i]
			dynType := d.Type
			if dynType == "" {
				dynType = "image"
			}
			cover := ""
			if imgs := parseDynamicImagesJSON(d.ImagesJSON); len(imgs) > 0 {
				cover = imgs[0]
			}
			entries = append(entries, unifiedDynRow{
				ID: d.ID, Kind: dynType, UserID: d.UserID,
				Title: d.Title, Content: d.Content, CoverURL: cover,
				LikeCount: d.LikeCount, CommentCount: d.CommentCount,
				CreatedAt: d.CreatedAt.Format("2006-01-02 15:04:05"),
			})
		}
	}

	// ── Sort: newest first ──
	sort.Slice(entries, func(i, j int) bool {
		return entries[i].CreatedAt > entries[j].CreatedAt
	})

	total := int64(len(entries))
	totalPages := int((total + int64(pageSize) - 1) / int64(pageSize))
	if totalPages < 1 {
		totalPages = 1
	}
	if page > totalPages {
		page = totalPages
	}
	offset := (page - 1) * pageSize
	end := offset + pageSize
	if end > len(entries) {
		end = len(entries)
	}
	paged := entries[offset:end]

	// ── Batch fetch author names ──
	uidSet := map[uint64]bool{}
	for i := range paged {
		uidSet[paged[i].UserID] = true
	}
	uids := make([]uint64, 0, len(uidSet))
	for id := range uidSet {
		uids = append(uids, id)
	}
	names := map[uint64]string{}
	if len(uids) > 0 {
		var users []model.User
		_ = a.DB.Where("id IN ?", uids).Find(&users).Error
		for i := range users {
			names[users[i].ID] = model.DisplayUsername(&users[i])
			if users[i].Nickname != "" && !model.IsUserAnonymized(&users[i]) {
				names[users[i].ID] = strings.TrimSpace(users[i].Nickname)
			}
		}
	}

	items := make([]gin.H, 0, len(paged))
	for i := range paged {
		items = append(items, gin.H{
			"id":            paged[i].ID,
			"kind":          paged[i].Kind,
			"user_id":       paged[i].UserID,
			"uploader_name": names[paged[i].UserID],
			"title":         paged[i].Title,
			"content":       paged[i].Content,
			"cover_url":     paged[i].CoverURL,
			"like_count":    paged[i].LikeCount,
			"comment_count": paged[i].CommentCount,
			"created_at":    paged[i].CreatedAt,
		})
	}

	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}
