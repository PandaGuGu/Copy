package handler

import (
	"context"
	"encoding/json"
	"fmt"
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
		var duration int64
		if rows[i].StartedAt != nil && rows[i].FinishedAt != nil {
			duration = rows[i].FinishedAt.Sub(*rows[i].StartedAt).Milliseconds()
		}
		items = append(items, gin.H{
			"id":          rows[i].ID,
			"task_type":   rows[i].TaskType,
			"type":        rows[i].TaskType, // 前端用 type
			"target_id":   rows[i].TargetID,
			"status":      rows[i].Status,
			"retry_count": rows[i].RetryCount,
			"error_msg":   rows[i].ErrorMsg,
			"error":       rows[i].ErrorMsg, // 前端用 error
			"duration":    duration,
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

	// Batch preload alert rules to avoid N+1 queries
	ruleIDSet := make(map[uint64]bool)
	for _, r := range rows {
		ruleIDSet[r.RuleID] = true
	}
	ruleIDs := make([]uint64, 0, len(ruleIDSet))
	for rid := range ruleIDSet {
		ruleIDs = append(ruleIDs, rid)
	}
	var rules []model.AlertRule
	ruleMap := make(map[uint64]*model.AlertRule, len(ruleIDs))
	if len(ruleIDs) > 0 {
		if err := a.DB.Where("id IN ?", ruleIDs).Find(&rules).Error; err == nil {
			for i := range rules {
				ruleMap[rules[i].ID] = &rules[i]
			}
		}
	}

	for i := range rows {
		// 根据阈值偏离程度计算告警级别
		var level string
		var ruleName string
		if rule, ok := ruleMap[rows[i].RuleID]; ok {
			ruleName = rule.Name
			if rule.Threshold != 0 {
				deviation := rows[i].Value / rule.Threshold
				switch {
				case rule.Operator == "gt" || rule.Operator == "gte":
					// For "greater than" rules, higher value is worse
					if deviation >= 2.0 {
						level = "critical"
					} else if deviation >= 1.3 {
						level = "warning"
					} else {
						level = "info"
					}
				case rule.Operator == "lt" || rule.Operator == "lte":
					// For "less than" rules, lower value is worse (invert)
					if deviation <= 0.5 {
						level = "critical"
					} else if deviation <= 0.7 {
						level = "warning"
					} else {
						level = "info"
					}
				default:
					level = "info"
				}
			} else {
				level = "info"
			}
		} else {
			level = "info"
		}
		msg := fmt.Sprintf("%s: 当前值 %.2f", ruleName, rows[i].Value)

		items = append(items, gin.H{
			"id":           rows[i].ID,
			"rule_id":      rows[i].RuleID,
			"rule_name":    ruleName,
			"value":        rows[i].Value,
			"status":       rows[i].Status,
			"level":        level,
			"message":      msg,
			"acknowledged": rows[i].Status == "resolved",
			"acked_by":     rows[i].AckedBy,
			"acked_at":     rows[i].AckedAt,
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

	items := []gin.H{}
	allOK := true

	// DB check + latency
	var dbLatency int64
	if sqlDB, err := a.DB.DB(); err != nil {
		items = append(items, gin.H{"name": "Database", "status": "error", "detail": err.Error(), "latency": 0})
		allOK = false
	} else {
		start := time.Now()
		if perr := sqlDB.PingContext(ctx); perr != nil {
			dbLatency = time.Since(start).Milliseconds()
			items = append(items, gin.H{"name": "Database", "status": "error", "detail": perr.Error(), "latency": dbLatency})
			allOK = false
		} else {
			dbLatency = time.Since(start).Milliseconds()
			detail := fmt.Sprintf("Connected (%dms)", dbLatency)

			// DB pool stats
			poolStats := sqlDB.Stats()
			detail += fmt.Sprintf(" | %d open, %d idle", poolStats.OpenConnections, poolStats.Idle)
			items = append(items, gin.H{"name": "Database", "status": "ok", "detail": detail, "latency": dbLatency})
		}
	}

	// Redis check + latency
	var redisLatency int64
	if a.Redis == nil {
		items = append(items, gin.H{"name": "Redis", "status": "unavailable", "detail": "not configured", "latency": 0})
		allOK = false
	} else {
		start := time.Now()
		if err := a.Redis.Ping(ctx).Err(); err != nil {
			redisLatency = time.Since(start).Milliseconds()
			items = append(items, gin.H{"name": "Redis", "status": "error", "detail": err.Error(), "latency": redisLatency})
			allOK = false
		} else {
			redisLatency = time.Since(start).Milliseconds()
			detail := fmt.Sprintf("Connected (%dms)", redisLatency)

			// Redis memory info
			if info, e := a.Redis.Info(ctx, "memory").Result(); e == nil {
				for _, line := range strings.Split(info, "\r\n") {
					if strings.HasPrefix(line, "used_memory_human:") {
						parts := strings.SplitN(line, ":", 2)
						if len(parts) == 2 {
							detail += " | " + strings.TrimSpace(parts[1])
						}
						break
					}
				}
			}
			items = append(items, gin.H{"name": "Redis", "status": "ok", "detail": detail, "latency": redisLatency})
		}
	}

	// OSS check - real probe
	ossStatus := "ok"
	ossDetail := "Available"
	var ossLatency int64
	if a.OSS == nil {
		ossStatus = "unavailable"
		ossDetail = "not configured"
		allOK = false
	} else {
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if lat, err := a.OSS.Ping(ctx); err != nil {
			ossStatus = "unavailable"
			ossDetail = fmt.Sprintf("probe failed: %v", err)
			allOK = false
		} else {
			ossLatency = lat
			ossDetail = fmt.Sprintf("ok (%dms)", lat)
		}
	}
	items = append(items, gin.H{"name": "Object Storage", "status": ossStatus, "detail": ossDetail, "latency": ossLatency})

	// RabbitMQ check - real probe
	mqStatus := "ok"
	mqDetail := "Available"
	if a.MQ == nil {
		mqStatus = "unavailable"
		mqDetail = "not configured"
		allOK = false
	} else if hc, ok := a.MQ.(interface{ IsAlive() bool }); ok && !hc.IsAlive() {
		mqStatus = "unavailable"
		mqDetail = "connection closed"
		allOK = false
	}
	items = append(items, gin.H{"name": "Message Queue", "status": mqStatus, "detail": mqDetail, "latency": 0})

	overall := "healthy"
	if !allOK {
		overall = "degraded"
	}

	resp.OK(c, gin.H{
		"items":      items,
		"overall":    overall,
		"checked_at": time.Now(),
	})
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
			"status_code": rows[i].Status, // 前端用 status_code
			"duration_ms": rows[i].DurationMs,
			"duration":    rows[i].DurationMs, // 前端用 duration
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
	RefreshType string   `json:"refresh_type"`
	Urls        []string `json:"urls"`
}

// AdminCreateCDNRefresh POST /admin/ops/cdn/refresh — create CDN refresh task
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
	req.RefreshType = strings.TrimSpace(req.RefreshType)
	if req.RefreshType == "" || len(req.Urls) == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.RefreshType != "url" && req.RefreshType != "directory" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	// Filter empty URLs
	cleanUrls := make([]string, 0, len(req.Urls))
	for _, u := range req.Urls {
		if u = strings.TrimSpace(u); u != "" {
			cleanUrls = append(cleanUrls, u)
		}
	}
	if len(cleanUrls) == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	urlsJSON, _ := json.Marshal(cleanUrls)
	task := model.CDNRefreshTask{
		RefreshType: req.RefreshType,
		Urls:        string(urlsJSON),
		Status:      "pending",
		RequestedBy: adminID,
	}
	if err := a.DB.Create(&task).Error; err != nil {
		a.Log.Error("create cdn refresh task", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_cdn_refresh", "cdn_refresh_task", task.ID, `{"refresh_type":"`+task.RefreshType+`","urls":`+string(urlsJSON)+`}`)
	// Parse URLs back for response
	var respUrls []string
	json.Unmarshal([]byte(task.Urls), &respUrls)
	resp.OK(c, gin.H{
		"id":           task.ID,
		"refresh_type": task.RefreshType,
		"urls":         respUrls,
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
		var urls []string
		if rows[i].Urls != "" {
			json.Unmarshal([]byte(rows[i].Urls), &urls)
		}
		if urls == nil {
			urls = []string{}
		}
		items = append(items, gin.H{
			"id":           rows[i].ID,
			"refresh_type": rows[i].RefreshType,
			"urls":         urls,
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

// AdminListOSSLifecycleRules GET /admin/ops/storage/lifecycle-rules — list lifecycle rules
func (a *API) AdminListOSSLifecycleRules(c *gin.Context) {
	var rules []model.OSSLifecycleRule
	if err := a.DB.Order("created_at DESC, id DESC").Find(&rules).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rules))
	for i := range rules {
		items = append(items, gin.H{
			"id":           rules[i].ID,
			"name":         rules[i].Name,
			"bucket":       rules[i].Bucket,
			"prefix":       rules[i].Prefix,
			"ia_days":      rules[i].IADays,
			"archive_days": rules[i].ArchiveDays,
			"delete_days":  rules[i].DeleteDays,
			"enabled":      rules[i].Enabled,
			"created_by":   rules[i].CreatedBy,
			"created_at":   rules[i].CreatedAt,
			"updated_at":   rules[i].UpdatedAt,
		})
	}
	resp.OK(c, gin.H{"items": items})
}

type ossLifecycleReq struct {
	Name        string `json:"name"`
	Bucket      string `json:"bucket"`
	Prefix      string `json:"prefix"`
	IADays      int    `json:"ia_days"`
	ArchiveDays int    `json:"archive_days"`
	DeleteDays  int    `json:"delete_days"`
	Enabled     *bool  `json:"enabled"`
}

// AdminCreateOSSLifecycleRule POST /admin/ops/storage/lifecycle-rules — create lifecycle rule
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
	if strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Bucket) == "" || strings.TrimSpace(req.Prefix) == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.IADays < 0 || req.ArchiveDays < 0 || req.DeleteDays < 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	rule := model.OSSLifecycleRule{
		Name:        strings.TrimSpace(req.Name),
		Bucket:      strings.TrimSpace(req.Bucket),
		Prefix:      strings.TrimSpace(req.Prefix),
		IADays:      req.IADays,
		ArchiveDays: req.ArchiveDays,
		DeleteDays:  req.DeleteDays,
		Enabled:     true,
		CreatedBy:   adminID,
	}
	if req.Enabled != nil {
		rule.Enabled = *req.Enabled
	}
	if err := a.DB.Create(&rule).Error; err != nil {
		a.Log.Error("create oss lifecycle rule", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_oss_lifecycle_rule", "oss_lifecycle_rule", rule.ID, fmt.Sprintf(`{"name":"%s","bucket":"%s","prefix":"%s"}`, rule.Name, rule.Bucket, rule.Prefix))
	resp.OK(c, gin.H{
		"id":           rule.ID,
		"name":         rule.Name,
		"bucket":       rule.Bucket,
		"prefix":       rule.Prefix,
		"ia_days":      rule.IADays,
		"archive_days": rule.ArchiveDays,
		"delete_days":  rule.DeleteDays,
		"enabled":      rule.Enabled,
	})
}

// AdminUpdateOSSLifecycleRule PUT /admin/ops/storage/lifecycle-rules/:id — update lifecycle rule
func (a *API) AdminUpdateOSSLifecycleRule(c *gin.Context) {
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
	var rule model.OSSLifecycleRule
	if err := a.DB.First(&rule, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	var req ossLifecycleReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	upd := map[string]interface{}{}
	if req.Name != "" {
		upd["name"] = strings.TrimSpace(req.Name)
	}
	if req.Bucket != "" {
		upd["bucket"] = strings.TrimSpace(req.Bucket)
	}
	if req.Prefix != "" {
		upd["prefix"] = strings.TrimSpace(req.Prefix)
	}
	if req.IADays >= 0 {
		upd["ia_days"] = req.IADays
	}
	if req.ArchiveDays >= 0 {
		upd["archive_days"] = req.ArchiveDays
	}
	if req.DeleteDays >= 0 {
		upd["delete_days"] = req.DeleteDays
	}
	if req.Enabled != nil {
		upd["enabled"] = *req.Enabled
	}
	if len(upd) == 0 {
		resp.OK(c, gin.H{"id": id, "updated": false})
		return
	}
	if err := a.DB.Model(&rule).Updates(upd).Error; err != nil {
		a.Log.Error("update oss lifecycle rule", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "update_oss_lifecycle_rule", "oss_lifecycle_rule", id, "")
	resp.OK(c, gin.H{"id": id, "updated": true})
}

// AdminDeleteOSSLifecycleRule DELETE /admin/ops/storage/lifecycle-rules/:id — delete lifecycle rule
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

// ──────────────────────────────────────────────
// Module 19b: Alert Evaluation Engine
// ──────────────────────────────────────────────

// AdminTriggerSync POST /admin/ops/sync/trigger — manually trigger data sync with TaskLog tracking
func (a *API) AdminTriggerSync(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	var body struct {
		SyncType string `json:"sync_type"` // es_videos / es_articles / es_users / play_counts / all
	}
	_ = c.ShouldBindJSON(&body)
	if body.SyncType == "" {
		body.SyncType = "all"
	}

	now := time.Now()
	task := model.TaskLog{
		TaskType:  "sync",
		TargetID:  adminID,
		Status:    "running",
		StartedAt: &now,
	}
	if err := a.DB.Create(&task).Error; err != nil {
		a.Log.Warn("create sync tasklog failed", zap.Error(err))
	}

	a.Log.Info("sync triggered",
		zap.Uint64("task_id", task.ID),
		zap.String("sync_type", body.SyncType),
		zap.Uint64("admin_id", adminID),
	)

	// Run sync in background
	go func(taskID uint64, syncType string) {
		var syncErr error
		ctx := context.Background()

		switch syncType {
		case "es_videos":
			syncErr = a.syncESIndexes(ctx, "videos")
		case "es_articles":
			syncErr = a.syncESIndexes(ctx, "articles")
		case "es_users":
			syncErr = a.syncESIndexes(ctx, "users")
		case "play_counts":
			syncErr = a.syncPlayCounts(ctx)
		case "all":
			if err := a.syncESIndexes(ctx, "videos"); err != nil {
				a.Log.Warn("sync es_videos failed", zap.Error(err))
			}
			if err := a.syncESIndexes(ctx, "articles"); err != nil {
				a.Log.Warn("sync es_articles failed", zap.Error(err))
			}
			if err := a.syncESIndexes(ctx, "users"); err != nil {
				a.Log.Warn("sync es_users failed", zap.Error(err))
			}
			if err := a.syncPlayCounts(ctx); err != nil {
				a.Log.Warn("sync play_counts failed", zap.Error(err))
			}
		}

		stmt := map[string]interface{}{"finished_at": time.Now()}
		if syncErr != nil {
			stmt["status"] = "failed"
			stmt["error_msg"] = syncErr.Error()
		} else {
			stmt["status"] = "success"
		}
		a.DB.Model(&model.TaskLog{}).Where("id = ?", taskID).Updates(stmt)

		// Write audit log directly (no gin context available in goroutine)
		audit := model.AuditLog{
			AdminID:   adminID,
			Action:    "sync_completed",
			Resource:  "sync",
			TargetID:  taskID,
			Detail:    fmt.Sprintf(`{"sync_type":"%s","status":"%s"}`, syncType, stmt["status"]),
			CreatedAt: time.Now(),
		}
		a.DB.Create(&audit)
	}(task.ID, body.SyncType)

	a.recordAudit(c, adminID, "sync_triggered", "sync", task.ID, fmt.Sprintf(`{"sync_type":"%s"}`, body.SyncType))
	resp.OK(c, gin.H{
		"task_id":   task.ID,
		"sync_type": body.SyncType,
		"status":    "running",
	})
}

// syncESIndexes bulk-reindexes all entities of a given type into Elasticsearch.
func (a *API) syncESIndexes(ctx context.Context, entityType string) error {
	if a.ES == nil || !a.ES.Enabled() {
		return fmt.Errorf("elasticsearch not configured")
	}
	switch entityType {
	case "videos":
		var ids []uint64
		if err := a.DB.Model(&model.Video{}).Pluck("id", &ids).Error; err != nil {
			return err
		}
		for _, id := range ids {
			_ = a.ES.IndexVideoFromDB(ctx, a.DB, id)
		}
		a.Log.Info("es sync done", zap.String("type", "videos"), zap.Int("count", len(ids)))
	case "articles":
		var ids []uint64
		if err := a.DB.Model(&model.Article{}).Pluck("id", &ids).Error; err != nil {
			return err
		}
		for _, id := range ids {
			_ = a.ES.IndexArticleFromDB(ctx, a.DB, id)
		}
		a.Log.Info("es sync done", zap.String("type", "articles"), zap.Int("count", len(ids)))
	case "users":
		var ids []uint64
		if err := a.DB.Model(&model.User{}).Pluck("id", &ids).Error; err != nil {
			return err
		}
		for _, id := range ids {
			_ = a.ES.IndexUserFromDB(ctx, a.DB, id)
		}
		a.Log.Info("es sync done", zap.String("type", "users"), zap.Int("count", len(ids)))
	}
	return nil
}

// syncPlayCounts flushes Redis play counter to MySQL VideoDailyStat.
func (a *API) syncPlayCounts(ctx context.Context) error {
	if a.Redis == nil {
		return fmt.Errorf("redis not configured")
	}
	// Trigger play count flush by reading the current counter keys
	keys, err := a.Redis.Keys(ctx, "play:*").Result()
	if err != nil {
		return fmt.Errorf("redis keys failed: %w", err)
	}
	count := 0
	for _, key := range keys {
		val, err := a.Redis.Get(ctx, key).Int64()
		if err != nil {
			continue
		}
		videoIDStr := strings.TrimPrefix(key, "play:")
		videoID, err := strconv.ParseUint(videoIDStr, 10, 64)
		if err != nil {
			continue
		}
		// Update play count directly
		a.DB.Model(&model.Video{}).Where("id = ?", videoID).Update("play_count", val)
		todayStr := time.Now().Format("2006-01-02")
		a.DB.Model(&model.VideoDailyStat{}).Where("video_id = ? AND date = ?", videoID, todayStr).
			Assign(model.VideoDailyStat{PlayCount: val}).
			FirstOrCreate(&model.VideoDailyStat{VideoID: videoID, Date: todayStr, PlayCount: val})
		count++
	}
	a.Log.Info("play count sync done", zap.Int("synced", count))
	return nil
}

func (a *API) AdminEvaluateAlerts(c *gin.Context) {
	adminID, _ := middleware.AdminID(c)

	var rules []model.AlertRule
	if err := a.DB.Where("enabled = ?", true).Find(&rules).Error; err != nil {
		a.Log.Error("evaluate: load rules", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// 采集系统指标
	metrics := a.collectMetrics(c.Request.Context())
	fired := 0

	for _, rule := range rules {
		val, ok := metrics[rule.Metric]
		if !ok {
			continue // 指标不可用则跳过
		}
		if !evaluateCond(val, rule.Operator, rule.Threshold) {
			continue
		}

		// 检查 DurationSec: 如果配置了持续时间，检查近期是否持续超标
		if rule.DurationSec > 0 {
			since := time.Now().Add(-time.Duration(rule.DurationSec) * time.Second)
			var recentCount int64
			a.DB.Model(&model.AlertRecord{}).Where("rule_id = ? AND created_at >= ? AND status = 'firing'", rule.ID, since).Count(&recentCount)
			// 简单策略: 最近 duration 内至少已有 1 条记录才触发（避免瞬时抖动）
			if recentCount == 0 {
				// 没有持续记录，插入一条占位不触发
				rec := model.AlertRecord{
					RuleID: rule.ID,
					Value:  val,
					Status: "firing",
				}
				a.DB.Create(&rec)
				continue
			}
		}

		rec := model.AlertRecord{
			RuleID: rule.ID,
			Value:  val,
			Status: "firing",
		}
		if err := a.DB.Create(&rec).Error; err != nil {
			a.Log.Warn("evaluate: create alert record", zap.Error(err), zap.String("metric", rule.Metric))
			continue
		}
		fired++

		// 告警通知（当前仅 log，后续可扩展 dingtalk/wecom/email）
		a.Log.Warn("ALERT FIRED",
			zap.String("rule", rule.Name),
			zap.String("metric", rule.Metric),
			zap.Float64("threshold", rule.Threshold),
			zap.Float64("actual", val),
			zap.String("channel", rule.Channel),
		)

		if adminID != 0 {
			a.recordAudit(c, adminID, "alert_evaluated", "alert_rule", rule.ID,
				fmt.Sprintf(`{"metric":"%s","value":%.2f,"threshold":%.2f}`, rule.Metric, val, rule.Threshold))
		}
	}

	resp.OK(c, gin.H{
		"rules_checked": len(rules),
		"fired":         fired,
		"evaluated_at":  time.Now(),
	})
}

func (a *API) collectMetrics(ctx context.Context) map[string]float64 {
	m := map[string]float64{}

	// DB 连接延迟
	if sqlDB, err := a.DB.DB(); err == nil {
		start := time.Now()
		if err := sqlDB.PingContext(ctx); err == nil {
			m["db_latency_ms"] = float64(time.Since(start).Milliseconds())
		}
	}

	// Redis 已用内存 (bytes)
	if a.Redis != nil {
		if info, err := a.Redis.Info(ctx, "memory").Result(); err == nil {
			// 简单解析 used_memory
			for _, line := range strings.Split(info, "\r\n") {
				if strings.HasPrefix(line, "used_memory:") {
					parts := strings.Split(line, ":")
					if len(parts) == 2 {
						if v, e := strconv.ParseFloat(strings.TrimSpace(parts[1]), 64); e == nil {
							m["redis_memory_bytes"] = v
						}
					}
					break
				}
			}
		}
	}

	// 任务失败率 (近1小时)
	var totalTasks, failedTasks int64
	since := time.Now().Add(-1 * time.Hour)
	a.DB.Model(&model.TaskLog{}).Where("created_at >= ?", since).Count(&totalTasks)
	a.DB.Model(&model.TaskLog{}).Where("created_at >= ? AND status = ?", since, "failed").Count(&failedTasks)
	if totalTasks > 0 {
		m["task_failure_rate"] = float64(failedTasks) / float64(totalTasks) * 100
	} else {
		m["task_failure_rate"] = 0
	}

	// 队列积压 (pending + retrying 的任务数)
	var queueDepth int64
	a.DB.Model(&model.TaskLog{}).Where("status IN ?", []string{"pending", "retrying"}).Count(&queueDepth)
	m["queue_depth"] = float64(queueDepth)

	return m
}

func evaluateCond(value float64, op string, threshold float64) bool {
	switch op {
	case ">", "gt":
		return value > threshold
	case "<", "lt":
		return value < threshold
	case ">=", "gte":
		return value >= threshold
	case "<=", "lte":
		return value <= threshold
	case "==", "eq":
		return value == threshold
	default:
		return false
	}
}
