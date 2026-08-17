package handler

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ListMyFollowing lists the authenticated user's followings (no privacy gate —
// it's the caller's own follow list, used by mobile 关注页).
// GET /api/v1/users/me/followings
func (a *API) ListMyFollowing(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	limit := 200
	if raw := c.Query("limit"); raw != "" {
		if n, err := strconv.Atoi(raw); err == nil && n > 0 && n <= 500 {
			limit = n
		}
	}
	var rows []model.UserFollow
	if err := a.DB.Where("follower_id = ?", uid).Order("created_at DESC").Limit(limit).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items, err := followUserRows(a.DB, uid, rows, true)
	if err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	following, _ := userFollowCounts(a.DB, uid)
	resp.OK(c, gin.H{"items": items, "total": following})
}

// ListFollowingDynamics lists recent dynamics authored by users the caller follows
// (关注页动态流). Ordered by created_at DESC.
// GET /api/v1/dynamics/following?limit=&cursor=
func (a *API) ListFollowingDynamics(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	limit := 50
	if raw := c.Query("limit"); raw != "" {
		if n, err := strconv.Atoi(raw); err == nil && n > 0 && n <= 100 {
			limit = n
		}
	}
	// 我关注的用户 id 集合
	var follows []model.UserFollow
	if err := a.DB.Where("follower_id = ?", uid).Find(&follows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	authorIDs := make([]uint64, 0, len(follows))
	for _, f := range follows {
		authorIDs = append(authorIDs, f.FolloweeID)
	}
	if len(authorIDs) == 0 {
		resp.OK(c, gin.H{"items": []gin.H{}, "next_cursor": ""})
		return
	}

	var dyns []model.UserDynamic
	q := a.DB.Where("user_id IN ?", authorIDs)
	// 游标：?cursor=<id> 只取比它更早的记录
	if raw := c.Query("cursor"); raw != "" {
		if id, err := strconv.ParseUint(raw, 10, 64); err == nil && id > 0 {
			q = q.Where("id < ?", id)
		}
	}
	if err := q.Order("id DESC").Limit(limit).Find(&dyns).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// 作者信息 + 点赞态批量
	ids := make([]uint64, 0, len(dyns))
	authorMap := map[uint64]*model.User{}
	var authors []model.User
	if err := a.DB.Where("id IN ?", authorIDs).Find(&authors).Error; err == nil {
		for i := range authors {
			authorMap[authors[i].ID] = &authors[i]
		}
	}
	for _, d := range dyns {
		ids = append(ids, d.ID)
	}
	likedMap := dynamicLikesByViewer(a.DB, uid, ids)

	items := make([]gin.H, 0, len(dyns))
	for _, d := range dyns {
		item := userDynamicPayload(&d, likedMap[d.ID])
		item["id"] = d.ID
		author := authorMap[d.UserID]
		item["author_id"] = d.UserID
		item["author_name"] = userDynamicAuthorName(author)
		item["author_avatar"] = uploaderAvatarForAPI(author)
		items = append(items, item)
	}

	next := ""
	if len(dyns) > 0 {
		next = strconv.FormatUint(dyns[len(dyns)-1].ID, 10)
	}
	resp.OK(c, gin.H{"items": items, "next_cursor": next})
}
