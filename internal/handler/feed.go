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

// GetSubscriptionFeed returns videos from users the current user follows.
func (a *API) GetSubscriptionFeed(c *gin.Context) {
	uid := c.MustGet("user_id").(uint64)
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit > 50 {
		limit = 50
	}
	cursorID, _ := strconv.ParseUint(c.DefaultQuery("cursor", "0"), 10, 64)

	// Get followed user IDs
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

// GetRecommendationFeed returns recommended videos based on simple weighted scoring.
func (a *API) GetRecommendationFeed(c *gin.Context) {
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit > 50 {
		limit = 50
	}
	cursorID, _ := strconv.ParseUint(c.DefaultQuery("cursor", "0"), 10, 64)

	// Simple recommendation: weighted score = play_count * 1 + like_count * 10 + coin_count * 20
	// In production this would use a proper recommendation engine
	query := a.DB.Where("status = ?", "published")
	if cursorID > 0 {
		query = query.Where("id < ?", cursorID)
	}

	var videos []model.Video
	// Fallback: order by play_count DESC, created_at DESC (same as home page for now)
	if err := query.Order("play_count DESC, created_at DESC, danmaku_count DESC").
		Limit(limit).Find(&videos).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}

	nextCursor := uint64(0)
	if len(videos) == limit {
		nextCursor = videos[limit-1].ID
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": gin.H{"items": videos, "next_cursor": nextCursor}})
}

// GetLeaderboard returns a leaderboard (top videos by play count or coin count).
func (a *API) GetLeaderboard(c *gin.Context) {
	by := c.DefaultQuery("by", "play") // play | coin | like | fav
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "50"))
	if limit > 100 {
		limit = 100
	}
	period := c.DefaultQuery("period", "all") // all | week | month

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

// GetZoneRecommendation returns top videos in a specific zone/partition.
func (a *API) GetZoneRecommendation(c *gin.Context) {
	zone := c.Param("zone")
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit > 50 {
		limit = 50
	}

	var videos []model.Video
	if err := a.DB.Where("status = ? AND zone LIKE ?", "published", zone+"%").
		Order("play_count DESC, created_at DESC").Limit(limit).Find(&videos).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": videos})
}
