package handler

import (
	"math"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
	"minibili/internal/pkg/userlevel"
)

func adminUserStatusFilter(s string) []string {
	switch strings.TrimSpace(s) {
	case "", "all":
		return nil
	case "active", "banned", "disabled":
		return []string{s}
	default:
		return nil
	}
}

// adminLoadUserCounts batch-loads video/article/dynamic/follower counts for user IDs.
func adminLoadUserCounts(db *gorm.DB, uids []uint64) (vidMap, artMap, dynMap, folMap map[uint64]int) {
	vidMap = map[uint64]int{}
	artMap = map[uint64]int{}
	dynMap = map[uint64]int{}
	folMap = map[uint64]int{}
	if len(uids) == 0 {
		return
	}

	type cnt struct {
		UserID uint64
		N      int
	}
	// video count
	{
		var rows []cnt
		_ = db.Model(&model.Video{}).
			Select("user_id, COUNT(*) as n").
			Where("user_id IN ? AND status = ?", uids, "published").
			Group("user_id").Find(&rows).Error
		for i := range rows {
			vidMap[rows[i].UserID] = rows[i].N
		}
	}
	// article count
	{
		var rows []cnt
		_ = db.Model(&model.Article{}).
			Select("user_id, COUNT(*) as n").
			Where("user_id IN ? AND status = ?", uids, articleStatusPublished).
			Group("user_id").Find(&rows).Error
		for i := range rows {
			artMap[rows[i].UserID] = rows[i].N
		}
	}
	// dynamic count
	{
		var rows []cnt
		_ = db.Model(&model.UserDynamic{}).
			Select("user_id, COUNT(*) as n").
			Where("user_id IN ?", uids).
			Group("user_id").Find(&rows).Error
		for i := range rows {
			dynMap[rows[i].UserID] = rows[i].N
		}
	}
	// follower count
	{
		var rows []cnt
		_ = db.Model(&model.UserFollow{}).
			Select("followee_id as user_id, COUNT(*) as n").
			Where("followee_id IN ? AND status = ?", uids, "following").
			Group("followee_id").Find(&rows).Error
		for i := range rows {
			folMap[rows[i].UserID] = rows[i].N
		}
	}
	return
}

// AdminListUsers GET /api/v1/admin/users
func (a *API) AdminListUsers(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 50 {
		pageSize = 20
	}
	statusQ := strings.TrimSpace(c.DefaultQuery("status", ""))
	searchQ := strings.TrimSpace(c.Query("q"))
	sortQ := strings.TrimSpace(c.DefaultQuery("sort", "created_at"))

	q := a.DB.Model(&model.User{})
	if sts := adminUserStatusFilter(statusQ); sts != nil {
		q = q.Where("status IN ?", sts)
	}
	if searchQ != "" {
		like := "%" + searchQ + "%"
		q = q.Where("username LIKE ? OR cake_id LIKE ? OR nickname LIKE ?", like, like, like)
	}

	var total int64
	if err := q.Count(&total).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	totalPages := int(math.Ceil(float64(total) / float64(pageSize)))
	offset := (page - 1) * pageSize

	orderCol := "created_at DESC, id DESC"
	if sortQ == "video_count" {
		// handled after query
	}

	var rows []model.User
	if err := q.Order(orderCol).Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	uids := make([]uint64, len(rows))
	for i := range rows {
		uids[i] = rows[i].ID
	}

	vidMap, artMap, dynMap, folMap := adminLoadUserCounts(a.DB, uids)
	lvMap := userlevel.BatchCurrentLevels(a.DB, uids)

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		uid := rows[i].ID
		lv := lvMap[uid]
		if lv < 1 {
			lv = 1
		}
		items = append(items, gin.H{
			"id":             uid,
			"username":       rows[i].Username,
			"cake_id":        rows[i].CakeID,
			"nickname":       rows[i].Nickname,
			"avatar_url":     rows[i].AvatarURL,
			"status":         rows[i].Status,
			"banned_reason":  rows[i].BannedReason,
			"video_count":    vidMap[uid],
			"article_count":  artMap[uid],
			"dynamic_count":  dynMap[uid],
			"follower_count": folMap[uid],
			"coin_balance":   float64(rows[i].CoinBalanceTenths) / 10.0,
			"experience":     rows[i].Experience,
			"level":          lv,
			"created_at":     rows[i].CreatedAt,
		})
	}

	// video_count sort — sort in memory if requested
	if sortQ == "video_count" {
		// already queried all; just sort the result
		for i := 0; i < len(items); i++ {
			for j := i + 1; j < len(items); j++ {
				vi, _ := items[i]["video_count"].(int)
				vj, _ := items[j]["video_count"].(int)
				if vj > vi {
					items[i], items[j] = items[j], items[i]
				}
			}
		}
		// apply pagination after sort
		if offset > len(items) {
			items = items[:0]
		} else {
			end := offset + pageSize
			if end > len(items) {
				end = len(items)
			}
			items = items[offset:end]
		}
	}

	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}

// AdminGetUser GET /api/v1/admin/users/:id
func (a *API) AdminGetUser(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var u model.User
	if err := a.DB.First(&u, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	uid := u.ID
	uids := []uint64{uid}
	vidMap, artMap, dynMap, folMap := adminLoadUserCounts(a.DB, uids)
	lvMap := userlevel.BatchCurrentLevels(a.DB, uids)
	lv := lvMap[uid]
	if lv < 1 {
		lv = 1
	}

	resp.OK(c, gin.H{
		"id":             uid,
		"username":       u.Username,
		"cake_id":        u.CakeID,
		"nickname":       u.Nickname,
		"avatar_url":     u.AvatarURL,
		"sign":           u.Sign,
		"gender":         u.Gender,
		"birthday":       u.Birthday,
		"status":         u.Status,
		"banned_reason":  u.BannedReason,
		"video_count":    vidMap[uid],
		"article_count":  artMap[uid],
		"dynamic_count":  dynMap[uid],
		"follower_count": folMap[uid],
		"coin_balance":   float64(u.CoinBalanceTenths) / 10.0,
		"experience":     u.Experience,
		"level":          lv,
		"created_at":     u.CreatedAt,
		"updated_at":     u.UpdatedAt,
	})
}

// AdminBanUser POST /api/v1/admin/users/:id/ban
func (a *API) AdminBanUser(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	type banJSON struct {
		Reason string `json:"reason"`
	}
	var body banJSON
	_ = c.ShouldBindJSON(&body)
	reason := strings.TrimSpace(body.Reason)
	if reason == "" {
		reason = "违规行为"
	}

	var u model.User
	if err := a.DB.Select("id", "status", "banned_reason", "banned_at").First(&u, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if u.Status == "banned" {
		resp.OK(c, gin.H{"status": "already_banned", "message": "该账号已被封禁"})
		return
	}

	now := time.Now()
	if err := a.DB.Model(&u).Updates(map[string]interface{}{
		"status":        "banned",
		"banned_reason": reason,
		"banned_at":     now,
	}).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{
		"status": "banned",
		"reason": reason,
	})
}

// AdminUnbanUser POST /api/v1/admin/users/:id/unban
func (a *API) AdminUnbanUser(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var u model.User
	if err := a.DB.Select("id", "status").First(&u, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if u.Status != "banned" {
		resp.OK(c, gin.H{"status": "not_banned", "message": "该账号未被封禁"})
		return
	}

	if err := a.DB.Model(&u).Updates(map[string]interface{}{
		"status":        "active",
		"banned_reason": "",
		"banned_at":     nil,
	}).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{"status": "active"})
}

// AdminDeleteUser POST /api/v1/admin/users/:id/delete
func (a *API) AdminDeleteUser(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var u model.User
	if err := a.DB.Select("id", "status").First(&u, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	now := time.Now()
	if err := a.DB.Model(&u).Updates(map[string]interface{}{
		"status":               "disabled",
		"deletion_requested_at": now,
		"deletion_effective_at": now.Add(time.Hour), // 1 hour grace period for admin force-delete
	}).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{"status": "disabled", "message": "账号已强制注销"})
}
