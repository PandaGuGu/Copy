package handler

import (
	"net/http"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	"minibili/internal/errcode"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

type adminCommentItem struct {
	ID        uint64    `json:"id"`
	Type      string    `json:"type"`
	Content   string    `json:"content"`
	AuthorID  uint64    `json:"-"`
	Author    gin.H     `json:"author"`
	TargetID  uint64    `json:"-"`
	Target    gin.H     `json:"target"`
	LikeCount uint64    `json:"like_count"`
	Status    string    `json:"status"`
	CreatedAt time.Time `json:"created_at"`
}

func commentStatus(approved, curatedIgnored bool) string {
	if curatedIgnored {
		return "ignored"
	}
	if !approved {
		return "pending"
	}
	return "approved"
}

func truncateStr(s string, n int) string {
	runes := []rune(s)
	if len(runes) > n {
		return string(runes[:n]) + "..."
	}
	return s
}

func (a *API) loadAuthorBrief(userID uint64) gin.H {
	if userID == 0 {
		return nil
	}
	var u model.User
	if err := a.DB.Select("id, username, nickname, avatar_url").First(&u, userID).Error; err != nil {
		return gin.H{"id": userID}
	}
	return gin.H{
		"id":         u.ID,
		"username":   u.Username,
		"nickname":   u.Nickname,
		"avatar_url": u.AvatarURL,
	}
}

// AdminListComments GET /api/v1/admin/comments
func (a *API) AdminListComments(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	q := strings.TrimSpace(c.Query("q"))
	typ := strings.TrimSpace(c.Query("type")) // video / article / dynamic / empty=all
	status := strings.TrimSpace(c.Query("status")) // pending / approved / empty=all

	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	var all []adminCommentItem

	// Query each type
	doVideo := typ == "" || typ == "video"
	doArticle := typ == "" || typ == "article"
	doDynamic := typ == "" || typ == "dynamic"

	// Video comments
	if doVideo {
		var comments []model.Comment
		tx := a.DB.Model(&model.Comment{})
		if status == "pending" {
			tx = tx.Where("approved = ?", false)
		} else if status == "approved" {
			tx = tx.Where("approved = ?", true)
		}
		if q != "" {
			like := "%" + q + "%"
			tx = tx.Where("content LIKE ?", like)
		}
		tx = tx.Order("created_at DESC").Limit(500)
		if err := tx.Find(&comments).Error; err == nil {
			for _, cm := range comments {
				all = append(all, adminCommentItem{
					ID:        cm.ID,
					Type:      "video",
					Content:   truncateStr(cm.Content, 120),
					AuthorID:  cm.UserID,
					TargetID:  cm.VideoID,
					LikeCount: cm.LikeCount,
					Status:    commentStatus(cm.Approved, cm.CuratedIgnored),
					CreatedAt: cm.CreatedAt,
				})
			}
		}
	}

	// Article comments
	if doArticle {
		var comments []model.ArticleComment
		tx := a.DB.Model(&model.ArticleComment{})
		if status == "pending" {
			tx = tx.Where("approved = ?", false)
		} else if status == "approved" {
			tx = tx.Where("approved = ?", true)
		}
		if q != "" {
			like := "%" + q + "%"
			tx = tx.Where("content LIKE ?", like)
		}
		tx = tx.Order("created_at DESC").Limit(500)
		if err := tx.Find(&comments).Error; err == nil {
			for _, cm := range comments {
				all = append(all, adminCommentItem{
					ID:        cm.ID,
					Type:      "article",
					Content:   truncateStr(cm.Content, 120),
					AuthorID:  cm.UserID,
					TargetID:  cm.ArticleID,
					LikeCount: cm.LikeCount,
					Status:    commentStatus(cm.Approved, cm.CuratedIgnored),
					CreatedAt: cm.CreatedAt,
				})
			}
		}
	}

	// Dynamic comments
	if doDynamic {
		var comments []model.DynamicComment
		tx := a.DB.Model(&model.DynamicComment{})
		if status == "pending" {
			tx = tx.Where("approved = ?", false)
		} else if status == "approved" {
			tx = tx.Where("approved = ?", true)
		}
		if q != "" {
			like := "%" + q + "%"
			tx = tx.Where("content LIKE ?", like)
		}
		tx = tx.Order("created_at DESC").Limit(500)
		if err := tx.Find(&comments).Error; err == nil {
			for _, cm := range comments {
				all = append(all, adminCommentItem{
					ID:        cm.ID,
					Type:      "dynamic",
					Content:   truncateStr(cm.Content, 120),
					AuthorID:  cm.UserID,
					TargetID:  cm.DynamicID,
					LikeCount: cm.LikeCount,
					Status:    commentStatus(cm.Approved, cm.CuratedIgnored),
					CreatedAt: cm.CreatedAt,
				})
			}
		}
	}

	// Sort by time desc
	sort.Slice(all, func(i, j int) bool {
		return all[i].CreatedAt.After(all[j].CreatedAt)
	})

	// Paginate
	total := len(all)
	start := (page - 1) * pageSize
	end := start + pageSize
	if start > total {
		start = total
	}
	if end > total {
		end = total
	}
	pageItems := all[start:end]

	// Batch load author and target info
	authorCache := make(map[uint64]gin.H)
	for i := range pageItems {
		uid := pageItems[i].AuthorID
		if _, ok := authorCache[uid]; !ok {
			authorCache[uid] = a.loadAuthorBrief(uid)
		}
		pageItems[i].Author = authorCache[uid]
		pageItems[i].Target = a.loadCommentTarget(pageItems[i].Type, pageItems[i].TargetID)
	}

	resp.OK(c, gin.H{
		"items":     pageItems,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	})
}

func (a *API) loadCommentTarget(typ string, targetID uint64) gin.H {
	if targetID == 0 {
		return nil
	}
	switch typ {
	case "video":
		var v model.Video
		if err := a.DB.Select("id, title").First(&v, targetID).Error; err == nil {
			return gin.H{"id": v.ID, "title": v.Title}
		}
	case "article":
		var ar model.Article
		if err := a.DB.Select("id, title").First(&ar, targetID).Error; err == nil {
			return gin.H{"id": ar.ID, "title": ar.Title}
		}
	case "dynamic":
		return gin.H{"id": targetID}
	}
	return gin.H{"id": targetID}
}

// AdminGetComment GET /api/v1/admin/comments/:id?type=video|article|dynamic
func (a *API) AdminGetComment(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	typ := strings.TrimSpace(c.Query("type"))

	var content string
	var authorID uint64
	var likeCount uint64
	var status string
	var createdAt time.Time

	switch typ {
	case "video":
		var cm model.Comment
		if err := a.DB.First(&cm, id).Error; err != nil {
			resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
			return
		}
		content = cm.Content
		authorID = cm.UserID
		likeCount = cm.LikeCount
		status = commentStatus(cm.Approved, cm.CuratedIgnored)
		createdAt = cm.CreatedAt
	case "article":
		var cm model.ArticleComment
		if err := a.DB.First(&cm, id).Error; err != nil {
			resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
			return
		}
		content = cm.Content
		authorID = cm.UserID
		likeCount = cm.LikeCount
		status = commentStatus(cm.Approved, cm.CuratedIgnored)
		createdAt = cm.CreatedAt
	case "dynamic":
		var cm model.DynamicComment
		if err := a.DB.First(&cm, id).Error; err != nil {
			resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
			return
		}
		content = cm.Content
		authorID = cm.UserID
		likeCount = cm.LikeCount
		status = commentStatus(cm.Approved, cm.CuratedIgnored)
		createdAt = cm.CreatedAt
	default:
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Load full author info
	var author model.User
	a.DB.Select("id, username, nickname, avatar_url, cake_id, status").First(&author, authorID)

	resp.OK(c, gin.H{
		"id":         id,
		"type":       typ,
		"content":    content,
		"like_count": likeCount,
		"status":     status,
		"created_at": createdAt,
		"author": gin.H{
			"id":         author.ID,
			"username":   author.Username,
			"nickname":   author.Nickname,
			"avatar_url": author.AvatarURL,
			"cake_id":    author.CakeID,
			"status":     author.Status,
		},
	})
}

// AdminDeleteComment POST /api/v1/admin/comments/:id/delete?type=video|article|dynamic
func (a *API) AdminDeleteComment(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	typ := strings.TrimSpace(c.Query("type"))

	var result error
	switch typ {
	case "video":
		result = a.DB.Delete(&model.Comment{}, id).Error
	case "article":
		result = a.DB.Delete(&model.ArticleComment{}, id).Error
	case "dynamic":
		result = a.DB.Delete(&model.DynamicComment{}, id).Error
	default:
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	if result != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin delete comment",
		zap.String("type", typ),
		zap.Uint64("comment_id", id),
	)
	resp.OK(c, nil)
}
