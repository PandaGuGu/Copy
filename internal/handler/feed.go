package handler

import (
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"

	e "minibili/internal/errcode"
	"minibili/internal/model"
)

// ─── Feed & Recommendation Enhancement (Module 7) ───
//
// Handlers delegate to FeedService which performs:
//
//	multi-path recall → MMR diversity re-ranking → Redis caching.
//
// Authenticated users get personalised λ based on profile;
// anonymous users get default λ=0.7 with pre-computed cache.

// GetSubscriptionFeed returns videos from users the current user follows.
// Keeps the original simple SQL implementation — subscription feed is
// inherently personal and time-sorted; diversity re-ranking is unnecessary.
func (a *API) GetSubscriptionFeed(c *gin.Context) {
	uid := c.MustGet("user_id").(uint64)
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit > 50 {
		limit = 50
	}
	cursorID, _ := strconv.ParseUint(c.DefaultQuery("cursor", "0"), 10, 64)

	var followeeIDs []uint64
	if err := a.DB.Model(&model.UserFollow{}).
		Where("follower_id = ?", uid).
		Pluck("followee_id", &followeeIDs).Error; err != nil || len(followeeIDs) == 0 {
		c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": gin.H{"items": []model.Video{}, "next_cursor": 0}})
		return
	}

	query := a.DB.Where("status = ? AND user_id IN ?", "published", followeeIDs)
	if cursorID > 0 {
		query = query.Where("id < ?", cursorID)
	}
	query = query.Order("id DESC").Limit(limit)

	var videos []model.Video
	if err := query.Find(&videos).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}

	nextCursor := uint64(0)
	if len(videos) == limit {
		nextCursor = videos[limit-1].ID
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": gin.H{"items": videos, "next_cursor": nextCursor}})
}

// GetRecommendationFeed returns MMR diversity-ranked recommendations.
// Authenticated users receive personalised λ; anonymous users get cached.
func (a *API) GetRecommendationFeed(c *gin.Context) {
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit > 50 {
		limit = 50
	}

	uid := a.getOptionalUserID(c)
	result := a.Feed.GetRecommendation(c.Request.Context(), uid, limit)

	c.JSON(http.StatusOK, gin.H{
		"code": e.CodeSuccess,
		"msg":  e.GetMsg(e.CodeSuccess),
		"data": gin.H{
			"items":       videoOrEmpty(result.Items),
			"next_cursor": result.NextCursor,
		},
	})
}

// GetLeaderboard returns a leaderboard (top videos by play count or coin count).
// Leaderboard is ranking by design — no diversity re-ranking applied.
func (a *API) GetLeaderboard(c *gin.Context) {
	by := c.DefaultQuery("by", "play")
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "50"))
	if limit > 100 {
		limit = 100
	}
	period := c.DefaultQuery("period", "all")

	query := a.DB.Where("status = ?", "published").Limit(limit)

	switch period {
	case "week":
		query = query.Where("created_at >= ?", time.Now().AddDate(0, 0, -7))
	case "month":
		query = query.Where("created_at >= ?", time.Now().AddDate(0, -1, 0))
	}

	switch by {
	case "coin":
		query = query.Order("coin_count DESC, created_at DESC")
	case "like":
		query = query.Order("like_count DESC, created_at DESC")
	case "fav":
		query = query.Order("fav_count DESC, created_at DESC")
	default:
		query = query.Order("play_count DESC, created_at DESC")
	}

	var videos []model.Video
	if err := query.Find(&videos).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": videos})
}

// GetZoneRecommendation returns MMR diversity-ranked videos within a zone.
func (a *API) GetZoneRecommendation(c *gin.Context) {
	zone := c.Param("zone")
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit > 50 {
		limit = 50
	}

	uid := a.getOptionalUserID(c)
	result := a.Feed.GetZoneRecommendation(c.Request.Context(), uid, zone, limit)

	c.JSON(http.StatusOK, gin.H{
		"code": e.CodeSuccess,
		"msg":  e.GetMsg(e.CodeSuccess),
		"data": gin.H{
			"items":       videoOrEmpty(result.Items),
			"next_cursor": result.NextCursor,
		},
	})
}

// ─── Helpers ──────────────────────────────────────────────────

// getOptionalUserID returns the authenticated user ID or 0 for anonymous.
func (a *API) getOptionalUserID(c *gin.Context) uint64 {
	raw, exists := c.Get("user_id")
	if !exists {
		return 0
	}
	uid, ok := raw.(uint64)
	if !ok {
		return 0
	}
	return uid
}

// videoOrEmpty ensures the response is never null.
func videoOrEmpty(videos []*model.Video) []*model.Video {
	if videos == nil {
		return []*model.Video{}
	}
	return videos
}
