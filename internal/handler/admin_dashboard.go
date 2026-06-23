package handler

import (
	"time"

	"github.com/gin-gonic/gin"

	"minibili/internal/pkg/resp"
)

// AdminDashboard GET /api/v1/admin/dashboard
func (a *API) AdminDashboard(c *gin.Context) {
	db := a.DB
	now := time.Now()
	today := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, now.Location())

	type trendPoint struct {
		Date  string `json:"date"`
		Users int64  `json:"users"`
		Videos int64 `json:"videos"`
	}

	var totalUsers, todayUsers, totalVideos, totalArticles, totalComments, todayVideos, todayArticles int64
	var pendingVideos, pendingArticles int64

	db.Raw("SELECT COUNT(*) FROM users WHERE anonymized_at IS NULL").Scan(&totalUsers)
	db.Raw("SELECT COUNT(*) FROM users WHERE anonymized_at IS NULL AND created_at >= ?", today).Scan(&todayUsers)
	db.Raw("SELECT COUNT(*) FROM videos WHERE status = 'published'").Scan(&totalVideos)
	db.Raw("SELECT COUNT(*) FROM videos WHERE status = 'pending_review'").Scan(&pendingVideos)
	db.Raw("SELECT COUNT(*) FROM videos WHERE status = 'published' AND created_at >= ?", today).Scan(&todayVideos)
	db.Raw("SELECT COUNT(*) FROM articles WHERE status = 'published'").Scan(&totalArticles)
	db.Raw("SELECT COUNT(*) FROM articles WHERE status = 'pending_review'").Scan(&pendingArticles)
	db.Raw("SELECT COUNT(*) FROM articles WHERE status = 'published' AND created_at >= ?", today).Scan(&todayArticles)
	// Total comments across all 3 types
	var vc, ac, dc int64
	db.Raw("SELECT COUNT(*) FROM comments").Scan(&vc)
	db.Raw("SELECT COUNT(*) FROM article_comments").Scan(&ac)
	db.Raw("SELECT COUNT(*) FROM dynamic_comments").Scan(&dc)
	totalComments = vc + ac + dc

	// 7-day trend
	var trends []trendPoint
	for i := 6; i >= 0; i-- {
		d := today.AddDate(0, 0, -i)
		next := d.AddDate(0, 0, 1)
		dateStr := d.Format("01-02")
		var du, dv int64
		db.Raw("SELECT COUNT(*) FROM users WHERE anonymized_at IS NULL AND created_at >= ? AND created_at < ?", d, next).Scan(&du)
		db.Raw("SELECT COUNT(*) FROM videos WHERE status = 'published' AND created_at >= ? AND created_at < ?", d, next).Scan(&dv)
		trends = append(trends, trendPoint{Date: dateStr, Users: du, Videos: dv})
	}

	resp.OK(c, gin.H{
		"total_users":      totalUsers,
		"today_users":      todayUsers,
		"total_videos":     totalVideos,
		"total_articles":   totalArticles,
		"total_comments":   totalComments,
		"today_videos":     todayVideos,
		"today_articles":   todayArticles,
		"pending_videos":   pendingVideos,
		"pending_articles": pendingArticles,
		"trend":            trends,
	})
}
