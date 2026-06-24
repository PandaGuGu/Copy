package handler

import (
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ScheduledPublish represents a scheduled publish record for a video.
type ScheduledPublish struct {
	ID        uint64    `gorm:"primaryKey"`
	VideoID   uint64    `gorm:"uniqueIndex;not null"`
	PublishAt time.Time `gorm:"not null"`
	Published bool      `gorm:"not null;default:false"`
	CreatedAt time.Time
	UpdatedAt time.Time
}

// SchedulePublishRequest is the request body for scheduling a video publish.
type SchedulePublishRequest struct {
	PublishAt string `json:"publish_at"` // ISO datetime string
}

// SchedulePublish schedules a video to be published at a future time (Module 5).
// POST /api/v1/videos/:id/schedule
func (a *API) SchedulePublish(c *gin.Context) {
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

	var req SchedulePublishRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	publishAt, err := time.Parse(time.RFC3339, req.PublishAt)
	if err != nil {
		// Try parsing without timezone as ISO datetime
		publishAt, err = time.Parse("2006-01-02T15:04:05", req.PublishAt)
		if err != nil {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
	}

	// Validate publish_at is in the future
	if publishAt.Before(time.Now()) {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Validate video exists and belongs to user
	var v model.Video
	if err := a.DB.First(&v, vid).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if v.UserID != uid {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	// Validate video is in draft or published state
	if v.Status != "draft" && v.Status != "published" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Create or update ScheduledPublish record
	var sp ScheduledPublish
	err = a.DB.Where("video_id = ?", vid).First(&sp).Error
	if err != nil && err == gorm.ErrRecordNotFound {
		// Create new schedule
		sp = ScheduledPublish{
			VideoID:   vid,
			PublishAt: publishAt,
			Published: false,
		}
		if err := a.DB.Create(&sp).Error; err != nil {
			a.Log.Error("create scheduled publish failed", zap.Error(err), zap.Uint64("video_id", vid))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	} else if err != nil {
		a.Log.Error("query scheduled publish failed", zap.Error(err), zap.Uint64("video_id", vid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	} else {
		// Update existing schedule
		if err := a.DB.Model(&sp).Updates(map[string]interface{}{
			"publish_at": publishAt,
			"published":  false,
		}).Error; err != nil {
			a.Log.Error("update scheduled publish failed", zap.Error(err), zap.Uint64("video_id", vid))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	}

	a.Log.Info("video publish scheduled",
		zap.Uint64("user_id", uid),
		zap.Uint64("video_id", vid),
		zap.Time("publish_at", publishAt),
	)

	resp.OK(c, gin.H{
		"video_id":   vid,
		"publish_at": publishAt.Format(time.RFC3339),
		"scheduled":  true,
	})
}

// CancelSchedule cancels a scheduled publish for a video (Module 5).
// DELETE /api/v1/videos/:id/schedule
func (a *API) CancelSchedule(c *gin.Context) {
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

	// Validate video exists and belongs to user
	var v model.Video
	if err := a.DB.First(&v, vid).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if v.UserID != uid {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	// Delete ScheduledPublish record
	result := a.DB.Where("video_id = ?", vid).Delete(&ScheduledPublish{})
	if result.Error != nil {
		a.Log.Error("delete scheduled publish failed", zap.Error(result.Error), zap.Uint64("video_id", vid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	if result.RowsAffected == 0 {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	a.Log.Info("video publish schedule cancelled",
		zap.Uint64("user_id", uid),
		zap.Uint64("video_id", vid),
	)

	resp.OK(c, gin.H{
		"video_id":    vid,
		"scheduled":   false,
		"cancelled_at": time.Now().Format(time.RFC3339),
	})
}

// CreatorStatsResponse is the response for creator dashboard stats.
type CreatorStatsResponse struct {
	TotalVideos int64 `json:"total_videos"`
	TotalPlays  int64 `json:"total_plays"`
	TotalCoins  int64 `json:"total_coins"`
	TotalFans   int64 `json:"total_fans"`
}

// CreatorStatsTrendItem represents daily stats for 7-day trend.
type CreatorStatsTrendItem struct {
	Date       string `json:"date"`
	PlayCount  int64  `json:"play_count"`
}

// GetCreatorStats returns creator dashboard stats (Module 5).
// GET /api/v1/users/me/creator/stats
func (a *API) GetCreatorStats(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	// Get total published videos count
	var totalVideos int64
	if err := a.DB.Model(&model.Video{}).
		Where("user_id = ? AND status = ?", uid, "published").
		Count(&totalVideos).Error; err != nil {
		a.Log.Error("count total videos failed", zap.Error(err), zap.Uint64("user_id", uid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Get total plays (sum of play_count from user's published videos)
	var totalPlays int64
	if err := a.DB.Model(&model.Video{}).
		Where("user_id = ? AND status = ?", uid, "published").
		Select("COALESCE(SUM(play_count), 0)").
		Scan(&totalPlays).Error; err != nil {
		a.Log.Error("sum total plays failed", zap.Error(err), zap.Uint64("user_id", uid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Get total coins (sum of coin_count from user's published videos)
	var totalCoins int64
	if err := a.DB.Model(&model.Video{}).
		Where("user_id = ? AND status = ?", uid, "published").
		Select("COALESCE(SUM(coin_count), 0)").
		Scan(&totalCoins).Error; err != nil {
		a.Log.Error("sum total coins failed", zap.Error(err), zap.Uint64("user_id", uid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Get total fans (count of UserFollow where followee_id = user_id)
	var totalFans int64
	if err := a.DB.Model(&model.UserFollow{}).
		Where("followee_id = ?", uid).
		Count(&totalFans).Error; err != nil {
		a.Log.Error("count total fans failed", zap.Error(err), zap.Uint64("user_id", uid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Get 7-day trend from real daily stats
	trend := make([]CreatorStatsTrendItem, 0, 7)
	now := time.Now()
	for i := 6; i >= 0; i-- {
		date := now.AddDate(0, 0, -i)
		dateStr := date.Format("2006-01-02")

		var dailyCount int64
		a.DB.Model(&model.VideoDailyStat{}).
			Where("video_id IN (?) AND date = ?",
				a.DB.Model(&model.Video{}).Select("id").Where("user_id = ? AND status = ?", uid, "published"),
				dateStr,
			).
			Select("COALESCE(SUM(play_count), 0)").
			Scan(&dailyCount)

		trend = append(trend, CreatorStatsTrendItem{
			Date:      dateStr,
			PlayCount: dailyCount,
		})
	}

	resp.OK(c, gin.H{
		"total_videos": totalVideos,
		"total_plays":  totalPlays,
		"total_coins":  totalCoins,
		"total_fans":   totalFans,
		"trend_7d":     trend,
	})
}

// CreatorVideoStatsItem represents per-video stats for creator's own videos.
type CreatorVideoStatsItem struct {
	VideoID      uint64 `json:"video_id"`
	Title        string `json:"title"`
	CoverURL     string `json:"cover_url"`
	PlayCount    uint64 `json:"play_count"`
	LikeCount    uint64 `json:"like_count"`
	CoinCount    uint64 `json:"coin_count"`
	CommentCount uint64 `json:"comment_count"`
	DanmakuCount uint64 `json:"danmaku_count"`
	FavCount     uint64 `json:"fav_count"`
}

// GetCreatorVideoStats returns per-video stats for creator's own videos (Module 5).
// GET /api/v1/users/me/creator/video-stats
func (a *API) GetCreatorVideoStats(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	// Query all published videos for the user
	var videos []model.Video
	if err := a.DB.Where("user_id = ? AND status = ?", uid, "published").
		Order("created_at DESC").
		Find(&videos).Error; err != nil {
		a.Log.Error("query creator video stats failed", zap.Error(err), zap.Uint64("user_id", uid))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]CreatorVideoStatsItem, 0, len(videos))
	for _, v := range videos {
		items = append(items, CreatorVideoStatsItem{
			VideoID:      v.ID,
			Title:        v.Title,
			CoverURL:     v.CoverURL,
			PlayCount:    v.PlayCount,
			LikeCount:    v.LikeCount,
			CoinCount:    v.CoinCount,
			CommentCount: v.CommentCount,
			DanmakuCount: v.DanmakuCount,
			FavCount:     v.FavCount,
		})
	}

	resp.OK(c, gin.H{
		"items": items,
		"total": len(items),
	})
}
