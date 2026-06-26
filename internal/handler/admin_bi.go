package handler

import (
	"encoding/csv"
	"fmt"
	"math"
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
// BI / Statistics admin handlers
// ──────────────────────────────────────────────

// AdminGetZoneStats GET /admin/bi/zone-stats
func (a *API) AdminGetZoneStats(c *gin.Context) {
	type row struct {
		Zone      string `json:"zone"`
		Count     int64  `json:"video_count"`
		PlayCount int64  `json:"play_count"`
	}
	var rows []row
	if err := a.DB.Model(&model.Video{}).
		Select("zone, COUNT(*) as count, COALESCE(SUM(play_count), 0) as play_count").
		Where("zone != '' AND status != 'deleted'").
		Group("zone").
		Order("play_count DESC").
		Find(&rows).Error; err != nil {
		a.Log.Error("zone stats query failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		avg := float64(0)
		if rows[i].Count > 0 {
			avg = math.Round(float64(rows[i].PlayCount)/float64(rows[i].Count)*10) / 10
		}
		items = append(items, gin.H{
			"zone":               rows[i].Zone,
			"video_count":        rows[i].Count,
			"play_count":         rows[i].PlayCount,
			"avg_plays_per_video": avg,
		})
	}
	resp.OK(c, gin.H{"zones": items})
}

// AdminGetCreatorStats GET /admin/bi/creator-stats
func (a *API) AdminGetCreatorStats(c *gin.Context) {
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit < 1 || limit > 100 {
		limit = 20
	}

	// Aggregate video stats per user (plays, coins, video_count)
	type videoAgg struct {
		UserID     uint64
		PlayCount  int64
		CoinCount  int64
		VideoCount int64
	}
	var videoRows []videoAgg
	a.DB.Model(&model.Video{}).
		Select("user_id, COALESCE(SUM(play_count), 0) as play_count, COALESCE(SUM(coin_count), 0) as coin_count, COUNT(*) as video_count").
		Where("status != 'deleted'").
		Group("user_id").
		Order("play_count DESC").
		Limit(limit).
		Find(&videoRows)

	// Aggregate fan count per user
	type fanAgg struct {
		UserID   uint64
		FanCount int64
	}
	var fanRows []fanAgg
	a.DB.Model(&model.UserFollow{}).
		Select("followee_id as user_id, COUNT(*) as fan_count").
		Group("followee_id").
		Find(&fanRows)

	// Aggregate article count per user
	type artAgg struct {
		UserID      uint64
		ArticleCount int64
	}
	var artRows []artAgg
	a.DB.Model(&model.Article{}).
		Select("user_id, COUNT(*) as article_count").
		Where("status != 'deleted'").
		Group("user_id").
		Find(&artRows)

	// Build lookup maps
	fanMap := make(map[uint64]int64, len(fanRows))
	for i := range fanRows {
		fanMap[fanRows[i].UserID] = fanRows[i].FanCount
	}
	artMap := make(map[uint64]int64, len(artRows))
	for i := range artRows {
		artMap[artRows[i].UserID] = artRows[i].ArticleCount
	}

	// If no video data, return empty
	if len(videoRows) == 0 {
		resp.OK(c, gin.H{"creators": []gin.H{}, "dimension": "play_count"})
		return
	}

	// Resolve usernames
	uids := make([]uint64, 0, len(videoRows))
	for i := range videoRows {
		uids = append(uids, videoRows[i].UserID)
	}
	var users []model.User
	_ = a.DB.Where("id IN ?", uids).Find(&users).Error
	userName := make(map[uint64]string, len(users))
	for i := range users {
		userName[users[i].ID] = model.DisplayUsername(&users[i])
	}

	items := make([]gin.H, 0, len(videoRows))
	for i := range videoRows {
		items = append(items, gin.H{
			"user_id":       videoRows[i].UserID,
			"username":      userName[videoRows[i].UserID],
			"total_plays":   videoRows[i].PlayCount,
			"total_coins":   videoRows[i].CoinCount,
			"fans_count":    fanMap[videoRows[i].UserID],
			"video_count":   videoRows[i].VideoCount,
			"article_count": artMap[videoRows[i].UserID],
		})
	}
	resp.OK(c, gin.H{"creators": items, "dimension": "play_count"})
}

// AdminGetTimeSeries GET /admin/bi/time-series
func (a *API) AdminGetTimeSeries(c *gin.Context) {
	metric := strings.TrimSpace(c.DefaultQuery("metric", "plays"))
	granularity := strings.TrimSpace(c.DefaultQuery("granularity", "daily"))
	days, _ := strconv.Atoi(c.DefaultQuery("days", "30"))
	if days < 1 || days > 365 {
		days = 30
	}

	trunc := "DATE(created_at)"
	if granularity == "weekly" {
		trunc = "DATE(DATE_SUB(created_at, INTERVAL WEEKDAY(created_at) DAY))"
	}

	type point struct {
		Date  string `json:"date"`
		Value int64  `json:"value"`
	}
	var points []point

	switch metric {
	case "new_users":
		if err := a.DB.Model(&model.User{}).
			Select(trunc+" as date, COUNT(*) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
			Group("date").
			Order("date ASC").
			Find(&points).Error; err != nil {
			a.Log.Error("time series new_users failed", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	case "new_videos":
		if err := a.DB.Model(&model.Video{}).
			Select(trunc+" as date, COUNT(*) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY) AND status != 'deleted'", days).
			Group("date").
			Order("date ASC").
			Find(&points).Error; err != nil {
			a.Log.Error("time series new_videos failed", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	case "plays":
		fallthrough
	default:
		if err := a.DB.Model(&model.Video{}).
			Select(trunc+" as date, COALESCE(SUM(play_count), 0) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY) AND status != 'deleted'", days).
			Group("date").
			Order("date ASC").
			Find(&points).Error; err != nil {
			a.Log.Error("time series plays failed", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	}

	resp.OK(c, gin.H{
		"metric":      metric,
		"granularity": granularity,
		"points":      points,
	})
}

// AdminExportReport POST /admin/bi/export — export report CSV with TaskLog tracking
func (a *API) AdminExportReport(c *gin.Context) {
	adminID, _ := middleware.AdminID(c)

	var body struct {
		Metric string `json:"metric"` // plays / new_users / new_videos
		Days   int    `json:"days"`
	}
	_ = c.ShouldBindJSON(&body)
	if body.Metric == "" {
		body.Metric = "plays"
	}
	if body.Days < 1 || body.Days > 365 {
		body.Days = 30
	}

	// Create TaskLog for report_export
	now := time.Now()
	task := model.TaskLog{
		TaskType:  "report_export",
		TargetID:  adminID,
		Status:    "running",
		StartedAt: &now,
	}
	if err := a.DB.Create(&task).Error; err != nil {
		a.Log.Warn("create report_export tasklog failed", zap.Error(err))
	}

	type point struct {
		Date  string
		Value int64
	}
	var points []point
	var queryErr error

	switch body.Metric {
	case "new_users":
		queryErr = a.DB.Model(&model.User{}).
			Select("DATE(created_at) as date, COUNT(*) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", body.Days).
			Group("date").Order("date ASC").Find(&points).Error
	case "new_videos":
		queryErr = a.DB.Model(&model.Video{}).
			Select("DATE(created_at) as date, COUNT(*) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY) AND status != 'deleted'", body.Days).
			Group("date").Order("date ASC").Find(&points).Error
	case "plays":
		fallthrough
	default:
		queryErr = a.DB.Model(&model.Video{}).
			Select("DATE(created_at) as date, COALESCE(SUM(play_count), 0) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY) AND status != 'deleted'", body.Days).
			Group("date").Order("date ASC").Find(&points).Error
	}

	if queryErr != nil {
		// Mark TaskLog as failed
		a.DB.Model(&task).Updates(map[string]interface{}{
			"status":    "failed",
			"error_msg": fmt.Sprintf("query failed: %v", queryErr),
			"finished_at": time.Now(),
		})
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	var buf strings.Builder
	w := csv.NewWriter(&buf)
	_ = w.Write([]string{"date", body.Metric})
	for i := range points {
		_ = w.Write([]string{points[i].Date, strconv.FormatInt(points[i].Value, 10)})
	}
	w.Flush()

	// Mark TaskLog as success
	a.DB.Model(&task).Updates(map[string]interface{}{
		"status":    "success",
		"finished_at": time.Now(),
	})

	// Also record audit
	a.recordAudit(c, adminID, "export_report", "bi_report", task.ID, fmt.Sprintf(`{"metric":"%s","days":%d,"rows":%d}`, body.Metric, body.Days, len(points)))

	c.Header("Content-Type", "text/csv; charset=utf-8")
	c.Header("Content-Disposition", "attachment; filename=report.csv")
	c.String(http.StatusOK, buf.String())
}

// AdminListSavedReports GET /admin/bi/reports
func (a *API) AdminListSavedReports(c *gin.Context) {
	adminID, _ := middleware.AdminID(c)

	var rows []model.SavedReport
	if err := a.DB.Where("creator_id = ? OR is_public = 1", adminID).
		Order("created_at DESC").Find(&rows).Error; err != nil {
		a.Log.Error("list saved reports failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":           rows[i].ID,
			"creator_id":   rows[i].CreatorID,
			"name":         rows[i].Name,
			"description":  rows[i].Description,
			"query_config": rows[i].QueryConfig,
			"chart_type":   rows[i].ChartType,
			"is_public":    rows[i].IsPublic,
			"created_at":   rows[i].CreatedAt,
			"updated_at":   rows[i].UpdatedAt,
		})
	}
	resp.OK(c, gin.H{"reports": items})
}

// AdminSaveReport POST /admin/bi/reports
func (a *API) AdminSaveReport(c *gin.Context) {
	adminID, _ := middleware.AdminID(c)

	var body struct {
		Name        string `json:"name"         binding:"required"`
		Description string `json:"description"`
		QueryConfig string `json:"query_config" binding:"required"`
		ChartType   string `json:"chart_type"`
		IsPublic    bool   `json:"is_public"`
	}
	if err := c.ShouldBindJSON(&body); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if body.ChartType == "" {
		body.ChartType = "table"
	}

	r := model.SavedReport{
		CreatorID:   adminID,
		Name:        strings.TrimSpace(body.Name),
		Description: strings.TrimSpace(body.Description),
		QueryConfig: body.QueryConfig,
		ChartType:   body.ChartType,
		IsPublic:    body.IsPublic,
		CreatedAt:   time.Now(),
		UpdatedAt:   time.Now(),
	}
	if err := a.DB.Create(&r).Error; err != nil {
		a.Log.Error("save report failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("report saved", zap.Uint64("report_id", r.ID), zap.Uint64("admin_id", adminID))
	resp.OK(c, gin.H{"id": r.ID})
}

// AdminDeleteSavedReport DELETE /admin/bi/reports/:id
func (a *API) AdminDeleteSavedReport(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	adminID, _ := middleware.AdminID(c)

	var r model.SavedReport
	if err := a.DB.First(&r, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if r.CreatorID != adminID {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	if err := a.DB.Delete(&r).Error; err != nil {
		a.Log.Error("delete saved report failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("report deleted", zap.Uint64("report_id", id), zap.Uint64("admin_id", adminID))
	resp.OK(c, gin.H{"status": "deleted"})
}

// ──────────────────────────────────────────────
// BI Dashboard Summary / Article Stats / Engagement
// ──────────────────────────────────────────────

// AdminGetBISummary GET /admin/bi/summary — dashboard overview cards
func (a *API) AdminGetBISummary(c *gin.Context) {
	type card struct {
		Key   string `json:"key"`
		Label string `json:"label"`
		Value int64  `json:"value"`
		Icon  string `json:"icon,omitempty"`
	}

	var summary struct {
		Cards   []card `json:"cards"`
		Updated string `json:"updated"`
	}

	// Total users
	var totalUsers int64
	a.DB.Model(&model.User{}).Count(&totalUsers)
	summary.Cards = append(summary.Cards, card{Key: "total_users", Label: "注册用户", Value: totalUsers})

	// Total videos (published)
	var totalVideos int64
	a.DB.Model(&model.Video{}).Where("status != ?", "deleted").Count(&totalVideos)
	summary.Cards = append(summary.Cards, card{Key: "total_videos", Label: "视频总量", Value: totalVideos})

	// Total articles
	var totalArticles int64
	a.DB.Model(&model.Article{}).Where("status != ?", "deleted").Count(&totalArticles)
	summary.Cards = append(summary.Cards, card{Key: "total_articles", Label: "专栏文章", Value: totalArticles})

	// Total comments
	var totalComments int64
	a.DB.Model(&model.Comment{}).Count(&totalComments)
	summary.Cards = append(summary.Cards, card{Key: "total_comments", Label: "评论总量", Value: totalComments})

	// Today's plays
	var todayPlays int64
	a.DB.Model(&model.VideoViewHistory{}).Where("DATE(viewed_at) = CURDATE()").Count(&todayPlays)
	summary.Cards = append(summary.Cards, card{Key: "today_plays", Label: "今日播放", Value: todayPlays})

	// Today's new users
	var todayUsers int64
	a.DB.Model(&model.User{}).Where("DATE(created_at) = CURDATE()").Count(&todayUsers)
	summary.Cards = append(summary.Cards, card{Key: "today_new_users", Label: "今日新增用户", Value: todayUsers})

	// Total video plays (all time)
	var totalPlays int64
	totalPlaysRow := a.DB.Model(&model.Video{}).Where("status != ?", "deleted").Select("COALESCE(SUM(play_count), 0)").Row()
	_ = totalPlaysRow.Scan(&totalPlays)
	summary.Cards = append(summary.Cards, card{Key: "total_plays", Label: "累计播放", Value: totalPlays})

	// Total danmaku
	var totalDanmaku int64
	a.DB.Model(&model.Danmaku{}).Count(&totalDanmaku)
	summary.Cards = append(summary.Cards, card{Key: "total_danmaku", Label: "弹幕总量", Value: totalDanmaku})

	// Coins in circulation (sum of all user balances)
	var coinBalance int64
	coinRow := a.DB.Model(&model.User{}).Select("COALESCE(SUM(coin_balance_tenths), 0)").Row()
	_ = coinRow.Scan(&coinBalance)
	summary.Cards = append(summary.Cards, card{Key: "coins_circulation", Label: "流通硬币 (十分之一单位)", Value: coinBalance})

	summary.Updated = time.Now().Format("2006-01-02 15:04:05")
	resp.OK(c, summary)
}

// AdminGetArticleStats GET /admin/bi/article-stats
func (a *API) AdminGetArticleStats(c *gin.Context) {
	days, _ := strconv.Atoi(c.DefaultQuery("days", "30"))
	if days < 1 || days > 365 {
		days = 30
	}

	// Articles by category
	type catRow struct {
		Category string `json:"category"`
		Count    int64  `json:"count"`
	}
	byCategory := make([]catRow, 0)
	a.DB.Model(&model.Article{}).
		Select("category, COUNT(*) as count").
		Where("category != '' AND status != 'deleted'").
		Group("category").Order("count DESC").
		Find(&byCategory)

	// Top articles by views
	type topArticle struct {
		ID        uint64 `json:"id"`
		Title     string `json:"title"`
		ViewCount int64  `json:"view_count"`
		CommentCount int64 `json:"comment_count"`
		CreatedAt time.Time `json:"created_at"`
	}
	var topArticles []topArticle
	a.DB.Model(&model.Article{}).
		Select("id, title, view_count, comment_count, created_at").
		Where("status != 'deleted'").
		Order("view_count DESC").
		Limit(20).
		Find(&topArticles)

	// Article time series (new articles per day)
	type point struct {
		Date  string `json:"date"`
		Value int64  `json:"value"`
	}
	var tsNewArticles []point
	a.DB.Model(&model.Article{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&tsNewArticles)

	// Article view time series
	var tsArticleViews []point
	a.DB.Model(&model.ArticleViewHistory{}).
		Select("DATE(viewed_at) as date, COUNT(*) as value").
		Where("viewed_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&tsArticleViews)

	resp.OK(c, gin.H{
		"by_category":       byCategory,
		"top_articles":      topArticles,
		"new_articles_ts":   tsNewArticles,
		"article_views_ts":  tsArticleViews,
	})
}

// AdminGetEngagementStats GET /admin/bi/engagement-stats
func (a *API) AdminGetEngagementStats(c *gin.Context) {
	days, _ := strconv.Atoi(c.DefaultQuery("days", "30"))
	if days < 1 || days > 365 {
		days = 30
	}

	type point struct {
		Date  string `json:"date"`
		Value int64  `json:"value"`
	}

	// Comments per day
	var commentsTS []point
	a.DB.Model(&model.Comment{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&commentsTS)

	// Danmaku per day
	var danmakuTS []point
	a.DB.Model(&model.Danmaku{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&danmakuTS)

	// Likes per day
	var likesTS []point
	a.DB.Model(&model.VideoLike{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&likesTS)

	// Favorites per day
	var favsTS []point
	a.DB.Model(&model.VideoFavorite{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&favsTS)

	// Coins per day
	var coinsTS []point
	a.DB.Model(&model.VideoCoin{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&coinsTS)

	// Article coins per day
	var articleCoinsTS []point
	a.DB.Model(&model.ArticleCoin{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&articleCoinsTS)

	// Follows per day
	var followsTS []point
	a.DB.Model(&model.UserFollow{}).
		Select("DATE(created_at) as date, COUNT(*) as value").
		Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&followsTS)

	// Engagement totals
	var totalLikes, totalFavs, totalCoins, totalVideoCoins int64
	a.DB.Model(&model.VideoLike{}).Count(&totalLikes)
	a.DB.Model(&model.VideoFavorite{}).Count(&totalFavs)
	a.DB.Model(&model.VideoCoin{}).Count(&totalVideoCoins)
	a.DB.Model(&model.ArticleCoin{}).Count(&totalCoins)

	resp.OK(c, gin.H{
		"comments_ts":       commentsTS,
		"danmaku_ts":        danmakuTS,
		"likes_ts":          likesTS,
		"favs_ts":           favsTS,
		"coins_ts":          coinsTS,
		"article_coins_ts":  articleCoinsTS,
		"follows_ts":        followsTS,
		"total_likes":       totalLikes,
		"total_favs":        totalFavs,
		"total_video_coins": totalVideoCoins,
		"total_article_coins": totalCoins,
	})
}

// ──────────────────────────────────────────────
// Manuscript (video + article) stats
// ──────────────────────────────────────────────

// AdminGetManuscriptStats GET /admin/bi/manuscript-stats
func (a *API) AdminGetManuscriptStats(c *gin.Context) {
	// Video aggregate stats — count published + active as "published"
	var videoPublished, videoDraft, videoPending, videoRejected int64
	a.DB.Model(&model.Video{}).Where("status IN ('published','active')").Count(&videoPublished)
	a.DB.Model(&model.Video{}).Where("status IN ('draft','pending')").Count(&videoDraft)
	a.DB.Model(&model.Video{}).Where("status IN ('processing','pending_review')").Count(&videoPending)
	a.DB.Model(&model.Video{}).Where("status IN ('rejected','failed')").Count(&videoRejected)

	var videoTotalPlay, videoTotalCoin, videoTotalFav int64
	row := a.DB.Model(&model.Video{}).Where("status IN ('published','active')").Select("COALESCE(SUM(play_count),0), COALESCE(SUM(coin_count),0), COALESCE(SUM(fav_count),0)").Row()
	_ = row.Scan(&videoTotalPlay, &videoTotalCoin, &videoTotalFav)
	totalVideos := videoPublished + videoDraft + videoPending + videoRejected

	// Article aggregate stats
	var articlePublished, articleDraft, articlePending, articleRejected int64
	a.DB.Model(&model.Article{}).Where("status = 'published'").Count(&articlePublished)
	a.DB.Model(&model.Article{}).Where("status = 'draft'").Count(&articleDraft)
	a.DB.Model(&model.Article{}).Where("status = 'pending_review'").Count(&articlePending)
	a.DB.Model(&model.Article{}).Where("status = 'rejected'").Count(&articleRejected)

	var articleTotalView, articleTotalCoin, articleTotalFav int64
	row2 := a.DB.Model(&model.Article{}).Select("COALESCE(SUM(view_count),0), COALESCE(SUM(coin_count),0), COALESCE(SUM(fav_count),0)").Row()
	_ = row2.Scan(&articleTotalView, &articleTotalCoin, &articleTotalFav)
	totalArticles := articlePublished + articleDraft + articlePending + articleRejected

	// Dynamic count
	var totalDynamics int64
	a.DB.Model(&model.UserDynamic{}).Count(&totalDynamics)

	// Top videos by plays
	type topVid struct {
		ID        uint64    `json:"id"`
		Title     string    `json:"title"`
		PlayCount int64     `json:"play_count"`
		Zone      string    `json:"zone"`
		CreatedAt time.Time `json:"created_at"`
	}
	var topVideos []topVid
	a.DB.Model(&model.Video{}).
		Select("id, title, play_count, zone, created_at").
		Where("status IN ('published','active')").
		Order("play_count DESC").Limit(10).
		Find(&topVideos)

	// Top articles by views
	type topArt struct {
		ID        uint64    `json:"id"`
		Title     string    `json:"title"`
		ViewCount int64     `json:"view_count"`
		CreatedAt time.Time `json:"created_at"`
	}
	var topArticles []topArt
	a.DB.Model(&model.Article{}).
		Select("id, title, view_count, created_at").
		Where("status = 'published'").
		Order("view_count DESC").Limit(10).
		Find(&topArticles)

	// Video play time series (last 30 days)
	days, _ := strconv.Atoi(c.DefaultQuery("days", "30"))
	if days < 1 || days > 365 {
		days = 30
	}
	type point struct {
		Date  string `json:"date"`
		Value int64  `json:"value"`
	}
	var videoTs []point
	a.DB.Model(&model.VideoViewHistory{}).
		Select("DATE(viewed_at) as date, COUNT(*) as value").
		Where("viewed_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&videoTs)

	// Article view time series
	var articleTs []point
	a.DB.Model(&model.ArticleViewHistory{}).
		Select("DATE(viewed_at) as date, COUNT(*) as value").
		Where("viewed_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
		Group("date").Order("date ASC").
		Find(&articleTs)

	resp.OK(c, gin.H{
		"video_summary": gin.H{
			"total":        totalVideos,
			"published":    videoPublished,
			"draft":        videoDraft,
			"pending":      videoPending,
			"rejected":     videoRejected,
			"total_plays":  videoTotalPlay,
			"total_coins":  videoTotalCoin,
			"total_favs":   videoTotalFav,
		},
		"article_summary": gin.H{
			"total":        totalArticles,
			"published":    articlePublished,
			"draft":        articleDraft,
			"pending":      articlePending,
			"rejected":     articleRejected,
			"total_views":  articleTotalView,
			"total_coins":  articleTotalCoin,
			"total_favs":   articleTotalFav,
		},
		"total_dynamics": totalDynamics,
		"top_videos":     topVideos,
		"top_articles":   topArticles,
		"video_plays_ts": videoTs,
		"article_views_ts": articleTs,
	})
}
