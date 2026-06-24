package handler

import (
	"context"
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

// recordAudit writes an AuditLog entry for an admin operation.
// It is the shared audit-trail helper used across all admin handlers.
func (a *API) recordAudit(c *gin.Context, adminID uint64, action, resource string, targetID uint64, detail string) {
	if err := a.DB.Create(&model.AuditLog{
		AdminID:   adminID,
		Action:    action,
		Resource:  resource,
		TargetID:  targetID,
		Detail:    detail,
		IPAddress: c.ClientIP(),
		CreatedAt: time.Now(),
	}).Error; err != nil {
		a.Log.Error("write audit log failed",
			zap.Error(err),
			zap.String("action", action),
			zap.String("resource", resource),
			zap.Uint64("target_id", targetID),
		)
	}
}

// ──────────────────────────────────────────────
// Module 18: Queue & Task Visualization
// ──────────────────────────────────────────────

// AdminListTaskLogs GET /admin/ops/tasks — list task logs (filter: task_type, status)
func (a *API) AdminListTaskLogs(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}
	taskType := strings.TrimSpace(c.Query("task_type"))
	status := strings.TrimSpace(c.Query("status"))

	q := a.DB.Model(&model.TaskLog{})
	if taskType != "" {
		q = q.Where("task_type = ?", taskType)
	}
	if status != "" {
		q = q.Where("status = ?", status)
	}

	var total int64
	if err := q.Count(&total).Error; err != nil {
		a.Log.Error("count task logs", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	offset := (page - 1) * pageSize
	var rows []model.TaskLog
	if err := q.Order("created_at DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		a.Log.Error("list task logs", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":          rows[i].ID,
			"task_type":   rows[i].TaskType,
			"target_id":   rows[i].TargetID,
			"status":      rows[i].Status,
			"retry_count": rows[i].RetryCount,
			"error_msg":   rows[i].ErrorMsg,
			"started_at":  rows[i].StartedAt,
			"finished_at": rows[i].FinishedAt,
			"created_at":  rows[i].CreatedAt,
		})
	}

	totalPages := int((total + int64(pageSize) - 1) / int64(pageSize))
	if totalPages < 1 {
		totalPages = 1
	}
	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}

// AdminRetryTask POST /admin/ops/tasks/:id/retry — retry a failed task
func (a *API) AdminRetryTask(c *gin.Context) {
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

	var tl model.TaskLog
	if err := a.DB.First(&tl, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if tl.Status != "failed" && tl.Status != "retrying" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	if err := a.DB.Model(&tl).Updates(map[string]interface{}{
		"status":      "retrying",
		"retry_count": tl.RetryCount + 1,
		"error_msg":   "",
		"started_at":  &now,
		"finished_at": nil,
	}).Error; err != nil {
		a.Log.Error("retry task update", zap.Error(err), zap.Uint64("task_id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Re-publish transcode tasks to the queue if applicable.
	if tl.TaskType == "transcode" && a.MQ != nil {
		body := []byte(strconv.FormatUint(tl.TargetID, 10))
		if err := a.MQ.PublishTranscode(c.Request.Context(), body); err != nil {
			a.Log.Warn("re-publish transcode task", zap.Error(err), zap.Uint64("target_id", tl.TargetID))
		}
	}

	a.Log.Info("admin retried task", zap.Uint64("task_id", id), zap.Uint64("admin_id", adminID))
	a.recordAudit(c, adminID, "retry_task", "task", id, `{"task_type":"`+tl.TaskType+`","target_id":`+strconv.FormatUint(tl.TargetID, 10)+`}`)
	resp.OK(c, gin.H{"id": id, "status": "retrying"})
}

// AdminGetQueueStats GET /admin/ops/queue-stats — RabbitMQ queue stats (task_logs aggregate)
func (a *API) AdminGetQueueStats(c *gin.Context) {
	type statRow struct {
		Status string
		N      int64
	}
	var byStatus []statRow
	_ = a.DB.Model(&model.TaskLog{}).
		Select("status, COUNT(*) as n").
		Where("status IN ?", []string{"pending", "running", "retrying", "failed"}).
		Group("status").Find(&byStatus).Error

	statusMap := map[string]int64{}
	for i := range byStatus {
		statusMap[byStatus[i].Status] = byStatus[i].N
	}

	type typeRow struct {
		TaskType string
		N        int64
	}
	var byType []typeRow
	_ = a.DB.Model(&model.TaskLog{}).
		Select("task_type, COUNT(*) as n").
		Where("status IN ?", []string{"pending", "running", "retrying"}).
		Group("task_type").Find(&byType).Error

	typeMap := map[string]int64{}
	for i := range byType {
		typeMap[byType[i].TaskType] = byType[i].N
	}

	// Recent failure rate (last 1h).
	var totalRecent, failedRecent int64
	since := time.Now().Add(-time.Hour)
	a.DB.Model(&model.TaskLog{}).Where("created_at >= ?", since).Count(&totalRecent)
	a.DB.Model(&model.TaskLog{}).Where("created_at >= ? AND status = ?", since, "failed").Count(&failedRecent)

	resp.OK(c, gin.H{
		"by_status": statusMap,
		"by_type":   typeMap,
		"recent_1h": gin.H{
			"total":        totalRecent,
			"failed":       failedRecent,
			"failure_rate": failureRate(totalRecent, failedRecent),
		},
	})
}

func failureRate(total, failed int64) float64 {
	if total == 0 {
		return 0
	}
	return float64(failed) / float64(total) * 100
}

// ──────────────────────────────────────────────
// Module 19: Monitoring & Alerting
// ──────────────────────────────────────────────

// AdminListAlertRules GET /admin/ops/alert-rules — list alert rules
func (a *API) AdminListAlertRules(c *gin.Context) {
	var rules []model.AlertRule
	q := a.DB.Model(&model.AlertRule{})
	if enabled := c.Query("enabled"); enabled != "" {
		if enabled == "true" {
			q = q.Where("enabled = ?", true)
		} else if enabled == "false" {
			q = q.Where("enabled = ?", false)
		}
	}
	if metric := strings.TrimSpace(c.Query("metric")); metric != "" {
		q = q.Where("metric = ?", metric)
	}
	if err := q.Order("created_at DESC, id DESC").Find(&rules).Error; err != nil {
		a.Log.Error("list alert rules", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rules))
	for i := range rules {
		items = append(items, gin.H{
			"id":           rules[i].ID,
			"name":         rules[i].Name,
			"metric":       rules[i].Metric,
			"threshold":    rules[i].Threshold,
			"operator":     rules[i].Operator,
			"duration_sec": rules[i].DurationSec,
			"channel":      rules[i].Channel,
			"channel_conf": rules[i].ChannelConf,
			"enabled":      rules[i].Enabled,
			"created_at":   rules[i].CreatedAt,
			"updated_at":   rules[i].UpdatedAt,
		})
	}
	resp.OK(c, gin.H{"items": items})
}

type alertRuleReq struct {
	Name        string  `json:"name"`
	Metric      string  `json:"metric"`
	Threshold   float64 `json:"threshold"`
	Operator    string  `json:"operator"`
	DurationSec int     `json:"duration_sec"`
	Channel     string  `json:"channel"`
	ChannelConf string  `json:"channel_conf"`
	Enabled     *bool   `json:"enabled"`
}

// AdminCreateAlertRule POST /admin/ops/alert-rules — create alert rule
func (a *API) AdminCreateAlertRule(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var req alertRuleReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Metric) == "" || strings.TrimSpace(req.Operator) == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.Channel == "" {
		req.Channel = "log"
	}
	rule := model.AlertRule{
		Name:        strings.TrimSpace(req.Name),
		Metric:      strings.TrimSpace(req.Metric),
		Threshold:   req.Threshold,
		Operator:    strings.TrimSpace(req.Operator),
		DurationSec: req.DurationSec,
		Channel:     req.Channel,
		ChannelConf: req.ChannelConf,
		Enabled:     true,
	}
	if req.Enabled != nil {
		rule.Enabled = *req.Enabled
	}
	if err := a.DB.Create(&rule).Error; err != nil {
		a.Log.Error("create alert rule", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_alert_rule", "alert_rule", rule.ID, `{"name":"`+rule.Name+`","metric":"`+rule.Metric+`"}`)
	resp.OK(c, gin.H{
		"id":        rule.ID,
		"name":      rule.Name,
		"metric":    rule.Metric,
		"threshold": rule.Threshold,
		"operator":  rule.Operator,
		"enabled":   rule.Enabled,
	})
}

// AdminUpdateAlertRule PUT /admin/ops/alert-rules/:id — update alert rule
func (a *API) AdminUpdateAlertRule(c *gin.Context) {
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
	var req alertRuleReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var rule model.AlertRule
	if err := a.DB.First(&rule, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	updates := map[string]interface{}{}
	if strings.TrimSpace(req.Name) != "" {
		updates["name"] = strings.TrimSpace(req.Name)
	}
	if strings.TrimSpace(req.Metric) != "" {
		updates["metric"] = strings.TrimSpace(req.Metric)
	}
	updates["threshold"] = req.Threshold
	if strings.TrimSpace(req.Operator) != "" {
		updates["operator"] = strings.TrimSpace(req.Operator)
	}
	updates["duration_sec"] = req.DurationSec
	if req.Channel != "" {
		updates["channel"] = req.Channel
	}
	updates["channel_conf"] = req.ChannelConf
	if req.Enabled != nil {
		updates["enabled"] = *req.Enabled
	}
	if err := a.DB.Model(&rule).Updates(updates).Error; err != nil {
		a.Log.Error("update alert rule", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "update_alert_rule", "alert_rule", id, "")
	resp.OK(c, gin.H{"id": id, "ok": true})
}

// AdminDeleteAlertRule DELETE /admin/ops/alert-rules/:id — delete alert rule
func (a *API) AdminDeleteAlertRule(c *gin.Context) {
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
	if err := a.DB.Delete(&model.AlertRule{}, id).Error; err != nil {
		a.Log.Error("delete alert rule", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "delete_alert_rule", "alert_rule", id, "")
	resp.OK(c, gin.H{"id": id, "deleted": true})
}

// AdminToggleAlertRule POST /admin/ops/alert-rules/:id/toggle — enable/disable
func (a *API) AdminToggleAlertRule(c *gin.Context) {
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
	var rule model.AlertRule
	if err := a.DB.First(&rule, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	newEnabled := !rule.Enabled
	if err := a.DB.Model(&rule).Update("enabled", newEnabled).Error; err != nil {
		a.Log.Error("toggle alert rule", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "toggle_alert_rule", "alert_rule", id, `{"enabled":`+strconv.FormatBool(newEnabled)+`}`)
	resp.OK(c, gin.H{"id": id, "enabled": newEnabled})
}

// AdminListAlertRecords GET /admin/ops/alert-records — list fired alerts
func (a *API) AdminListAlertRecords(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}
	status := strings.TrimSpace(c.Query("status"))
	q := a.DB.Model(&model.AlertRecord{})
	if status != "" {
		q = q.Where("status = ?", status)
	}
	var total int64
	if err := q.Count(&total).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	offset := (page - 1) * pageSize
	var rows []model.AlertRecord
	if err := q.Order("created_at DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":         rows[i].ID,
			"rule_id":    rows[i].RuleID,
			"value":      rows[i].Value,
			"status":     rows[i].Status,
			"acked_by":   rows[i].AckedBy,
			"acked_at":   rows[i].AckedAt,
			"created_at": rows[i].CreatedAt,
		})
	}
	totalPages := int((total + int64(pageSize) - 1) / int64(pageSize))
	if totalPages < 1 {
		totalPages = 1
	}
	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}

// AdminAckAlert POST /admin/ops/alert-records/:id/ack — acknowledge an alert
func (a *API) AdminAckAlert(c *gin.Context) {
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
	var rec model.AlertRecord
	if err := a.DB.First(&rec, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if rec.Status == "resolved" {
		resp.OK(c, gin.H{"id": id, "status": "resolved"})
		return
	}
	now := time.Now()
	if err := a.DB.Model(&rec).Updates(map[string]interface{}{
		"status":   "resolved",
		"acked_by": adminID,
		"acked_at": &now,
	}).Error; err != nil {
		a.Log.Error("ack alert", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "ack_alert", "alert_record", id, "")
	resp.OK(c, gin.H{"id": id, "status": "resolved", "acked_at": now})
}

// AdminGetSystemHealth GET /admin/ops/health — comprehensive system health (DB/Redis/OSS/RabbitMQ check)
func (a *API) AdminGetSystemHealth(c *gin.Context) {
	ctx, cancel := context.WithTimeout(c.Request.Context(), 5*time.Second)
	defer cancel()

	health := gin.H{}
	allOK := true

	// DB check
	dbStatus := "ok"
	dbDetail := ""
	if sqlDB, err := a.DB.DB(); err != nil {
		dbStatus = "error"
		dbDetail = err.Error()
		allOK = false
	} else if err := sqlDB.PingContext(ctx); err != nil {
		dbStatus = "error"
		dbDetail = err.Error()
		allOK = false
	}
	health["db"] = gin.H{"status": dbStatus, "detail": dbDetail}

	// Redis check
	redisStatus := "ok"
	redisDetail := ""
	if a.Redis == nil {
		redisStatus = "unavailable"
		redisDetail = "redis client not configured"
		allOK = false
	} else if err := a.Redis.Ping(ctx).Err(); err != nil {
		redisStatus = "error"
		redisDetail = err.Error()
		allOK = false
	}
	health["redis"] = gin.H{"status": redisStatus, "detail": redisDetail}

	// OSS check
	ossStatus := "ok"
	ossDetail := ""
	if a.OSS == nil {
		ossStatus = "unavailable"
		ossDetail = "oss not configured"
		allOK = false
	}
	health["oss"] = gin.H{"status": ossStatus, "detail": ossDetail}

	// RabbitMQ check (best-effort: the MQ interface only exposes PublishTranscode).
	mqStatus := "ok"
	mqDetail := ""
	if a.MQ == nil {
		mqStatus = "unavailable"
		mqDetail = "message queue not configured"
		allOK = false
	}
	health["rabbitmq"] = gin.H{"status": mqStatus, "detail": mqDetail}

	overall := "healthy"
	if !allOK {
		overall = "degraded"
	}
	health["overall"] = overall
	health["checked_at"] = time.Now()
	resp.OK(c, health)
}

// ──────────────────────────────────────────────
// Module 20: Trace & Log Retrieval
// ──────────────────────────────────────────────

// AdminSearchTraces GET /admin/ops/traces — search traces by trace_id, request_id, user_id, path, method
func (a *API) AdminSearchTraces(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	q := a.DB.Model(&model.TraceRecord{})
	if v := strings.TrimSpace(c.Query("trace_id")); v != "" {
		q = q.Where("trace_id = ?", v)
	}
	if v := strings.TrimSpace(c.Query("request_id")); v != "" {
		q = q.Where("request_id = ?", v)
	}
	if v := strings.TrimSpace(c.Query("user_id")); v != "" {
		if uid, err := strconv.ParseUint(v, 10, 64); err == nil {
			q = q.Where("user_id = ?", uid)
		}
	}
	if v := strings.TrimSpace(c.Query("path")); v != "" {
		q = q.Where("path LIKE ?", "%"+v+"%")
	}
	if v := strings.TrimSpace(c.Query("method")); v != "" {
		q = q.Where("method = ?", strings.ToUpper(v))
	}
	if v := strings.TrimSpace(c.Query("status")); v != "" {
		if code, err := strconv.Atoi(v); err == nil {
			q = q.Where("status = ?", code)
		}
	}

	var total int64
	if err := q.Count(&total).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	offset := (page - 1) * pageSize
	var rows []model.TraceRecord
	if err := q.Order("created_at DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":          rows[i].ID,
			"trace_id":    rows[i].TraceID,
			"request_id":  rows[i].RequestID,
			"user_id":     rows[i].UserID,
			"path":        rows[i].Path,
			"method":      rows[i].Method,
			"status":      rows[i].Status,
			"duration_ms": rows[i].DurationMs,
			"error_msg":   rows[i].ErrorMsg,
			"created_at":  rows[i].CreatedAt,
		})
	}
	totalPages := int((total + int64(pageSize) - 1) / int64(pageSize))
	if totalPages < 1 {
		totalPages = 1
	}
	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}

// AdminGetTrace GET /admin/ops/traces/:id — get trace detail
func (a *API) AdminGetTrace(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var tr model.TraceRecord
	if err := a.DB.First(&tr, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	resp.OK(c, gin.H{
		"id":          tr.ID,
		"trace_id":    tr.TraceID,
		"request_id":  tr.RequestID,
		"user_id":     tr.UserID,
		"path":        tr.Path,
		"method":      tr.Method,
		"status":      tr.Status,
		"duration_ms": tr.DurationMs,
		"error_msg":   tr.ErrorMsg,
		"created_at":  tr.CreatedAt,
	})
}

// ──────────────────────────────────────────────
// Module 22: CDN & Storage Ops
// ──────────────────────────────────────────────

type cdnRefreshReq struct {
	Type   string `json:"type"`
	Target string `json:"target"`
}

// AdminCreateCDNRefresh POST /admin/ops/cdn/refresh — create CDN refresh task (body: type, target)
func (a *API) AdminCreateCDNRefresh(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var req cdnRefreshReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Type = strings.TrimSpace(req.Type)
	req.Target = strings.TrimSpace(req.Target)
	if req.Type == "" || req.Target == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.Type != "url" && req.Type != "directory" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	task := model.CDNRefreshTask{
		Type:        req.Type,
		Target:      req.Target,
		Status:      "pending",
		RequestedBy: adminID,
	}
	if err := a.DB.Create(&task).Error; err != nil {
		a.Log.Error("create cdn refresh task", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_cdn_refresh", "cdn_refresh_task", task.ID, `{"type":"`+task.Type+`","target":"`+task.Target+`"}`)
	resp.OK(c, gin.H{
		"id":           task.ID,
		"type":         task.Type,
		"target":       task.Target,
		"status":       task.Status,
		"requested_by": task.RequestedBy,
		"created_at":   task.CreatedAt,
	})
}

// AdminListCDNRefreshTasks GET /admin/ops/cdn/refresh — list refresh tasks
func (a *API) AdminListCDNRefreshTasks(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}
	status := strings.TrimSpace(c.Query("status"))
	q := a.DB.Model(&model.CDNRefreshTask{})
	if status != "" {
		q = q.Where("status = ?", status)
	}
	var total int64
	if err := q.Count(&total).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	offset := (page - 1) * pageSize
	var rows []model.CDNRefreshTask
	if err := q.Order("created_at DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":           rows[i].ID,
			"type":         rows[i].Type,
			"target":       rows[i].Target,
			"status":       rows[i].Status,
			"requested_by": rows[i].RequestedBy,
			"finished_at":  rows[i].FinishedAt,
			"created_at":   rows[i].CreatedAt,
		})
	}
	totalPages := int((total + int64(pageSize) - 1) / int64(pageSize))
	if totalPages < 1 {
		totalPages = 1
	}
	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}

// AdminListOSSLifecycleRules GET /admin/ops/oss/lifecycle — list lifecycle rules
func (a *API) AdminListOSSLifecycleRules(c *gin.Context) {
	var rules []model.OSSLifecycleRule
	if err := a.DB.Order("created_at DESC, id DESC").Find(&rules).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rules))
	for i := range rules {
		items = append(items, gin.H{
			"id":         rules[i].ID,
			"prefix":     rules[i].Prefix,
			"action":     rules[i].Action,
			"days":       rules[i].Days,
			"enabled":    rules[i].Enabled,
			"created_by": rules[i].CreatedBy,
			"created_at": rules[i].CreatedAt,
			"updated_at": rules[i].UpdatedAt,
		})
	}
	resp.OK(c, gin.H{"items": items})
}

type ossLifecycleReq struct {
	Prefix  string `json:"prefix"`
	Action  string `json:"action"`
	Days    int    `json:"days"`
	Enabled *bool  `json:"enabled"`
}

// AdminCreateOSSLifecycleRule POST /admin/ops/oss/lifecycle — create lifecycle rule
func (a *API) AdminCreateOSSLifecycleRule(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var req ossLifecycleReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if strings.TrimSpace(req.Prefix) == "" || strings.TrimSpace(req.Action) == "" || req.Days < 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	switch req.Action {
	case "delete", "transition_to_ia", "transition_to_archive":
	default:
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	rule := model.OSSLifecycleRule{
		Prefix:    strings.TrimSpace(req.Prefix),
		Action:    req.Action,
		Days:      req.Days,
		Enabled:   true,
		CreatedBy: adminID,
	}
	if req.Enabled != nil {
		rule.Enabled = *req.Enabled
	}
	if err := a.DB.Create(&rule).Error; err != nil {
		a.Log.Error("create oss lifecycle rule", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_oss_lifecycle_rule", "oss_lifecycle_rule", rule.ID, `{"prefix":"`+rule.Prefix+`","action":"`+rule.Action+`","days":`+strconv.Itoa(rule.Days)+`}`)
	resp.OK(c, gin.H{
		"id":      rule.ID,
		"prefix":  rule.Prefix,
		"action":  rule.Action,
		"days":    rule.Days,
		"enabled": rule.Enabled,
	})
}

// AdminDeleteOSSLifecycleRule DELETE /admin/ops/oss/lifecycle/:id — delete lifecycle rule
func (a *API) AdminDeleteOSSLifecycleRule(c *gin.Context) {
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
	if err := a.DB.Delete(&model.OSSLifecycleRule{}, id).Error; err != nil {
		a.Log.Error("delete oss lifecycle rule", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "delete_oss_lifecycle_rule", "oss_lifecycle_rule", id, "")
	resp.OK(c, gin.H{"id": id, "deleted": true})
}
