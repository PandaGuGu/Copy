package handler

import (
	"encoding/csv"
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
		Where("zone != ''").
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
			avg = float64(rows[i].PlayCount) / float64(rows[i].Count)
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
	dimension := strings.TrimSpace(c.DefaultQuery("dimension", "play_count"))
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "20"))
	if limit < 1 || limit > 100 {
		limit = 20
	}

	type agg struct {
		UserID uint64
		Val    int64
	}
	var rows []agg

	switch dimension {
	case "coin_count":
		if err := a.DB.Model(&model.Video{}).
			Select("user_id, COALESCE(SUM(coin_count), 0) as val").
			Group("user_id").
			Order("val DESC").
			Limit(limit).
			Find(&rows).Error; err != nil {
			a.Log.Error("creator coin stats failed", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	case "fan_count":
		if err := a.DB.Model(&model.UserFollow{}).
			Select("followee_id as user_id, COUNT(*) as val").
			Group("followee_id").
			Order("val DESC").
			Limit(limit).
			Find(&rows).Error; err != nil {
			a.Log.Error("creator fan stats failed", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	case "play_count":
		fallthrough
	default:
		if err := a.DB.Model(&model.Video{}).
			Select("user_id, COALESCE(SUM(play_count), 0) as val").
			Group("user_id").
			Order("val DESC").
			Limit(limit).
			Find(&rows).Error; err != nil {
			a.Log.Error("creator play stats failed", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	}

	if len(rows) == 0 {
		resp.OK(c, gin.H{"creators": []gin.H{}})
		return
	}

	uids := make([]uint64, 0, len(rows))
	for i := range rows {
		uids = append(uids, rows[i].UserID)
	}

	var users []model.User
	_ = a.DB.Where("id IN ?", uids).Find(&users).Error
	userName := make(map[uint64]string, len(users))
	for i := range users {
		userName[users[i].ID] = model.DisplayUsername(&users[i])
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"user_id":      rows[i].UserID,
			"username":     userName[rows[i].UserID],
			dimension:      rows[i].Val,
		})
	}
	resp.OK(c, gin.H{"creators": items, "dimension": dimension})
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
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
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
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", days).
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

// AdminExportReport POST /admin/bi/export
func (a *API) AdminExportReport(c *gin.Context) {
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

	type point struct {
		Date  string
		Value int64
	}
	var points []point

	switch body.Metric {
	case "new_users":
		_ = a.DB.Model(&model.User{}).
			Select("DATE(created_at) as date, COUNT(*) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", body.Days).
			Group("date").Order("date ASC").Find(&points).Error
	case "new_videos":
		_ = a.DB.Model(&model.Video{}).
			Select("DATE(created_at) as date, COUNT(*) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", body.Days).
			Group("date").Order("date ASC").Find(&points).Error
	case "plays":
		fallthrough
	default:
		_ = a.DB.Model(&model.Video{}).
			Select("DATE(created_at) as date, COALESCE(SUM(play_count), 0) as value").
			Where("created_at >= DATE_SUB(CURDATE(), INTERVAL ? DAY)", body.Days).
			Group("date").Order("date ASC").Find(&points).Error
	}

	var buf strings.Builder
	w := csv.NewWriter(&buf)
	_ = w.Write([]string{"date", body.Metric})
	for i := range points {
		_ = w.Write([]string{points[i].Date, strconv.FormatInt(points[i].Value, 10)})
	}
	w.Flush()

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
