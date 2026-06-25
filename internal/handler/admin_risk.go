package handler

import (
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

// ---------- Risk Rules ----------

// AdminListRiskRules GET /api/v1/admin/risk/rules
func (a *API) AdminListRiskRules(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	category := strings.TrimSpace(c.Query("category"))
	enabledStr := strings.TrimSpace(c.Query("enabled"))

	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tx := a.DB.Model(&model.RiskRule{})
	if category != "" {
		tx = tx.Where("category = ?", category)
	}
	if enabledStr != "" {
		if enabled, err := strconv.ParseBool(enabledStr); err == nil {
			tx = tx.Where("enabled = ?", enabled)
		}
	}

	var total int64
	tx.Count(&total)

	var rules []model.RiskRule
	tx.Order("priority DESC, id ASC").Offset((page - 1) * pageSize).Limit(pageSize).Find(&rules)

	resp.OK(c, gin.H{
		"items":     rules,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	})
}

// AdminCreateRiskRule POST /api/v1/admin/risk/rules
func (a *API) AdminCreateRiskRule(c *gin.Context) {
	adminID, _ := middleware.AdminID(c)

	var req struct {
		Name        string `json:"name"`
		Category    string `json:"category"`
		RuleType    string `json:"rule_type"`
		Pattern     string `json:"pattern"`
		Action      string `json:"action"`
		DurationSec int    `json:"duration_sec"`
		Priority    int    `json:"priority"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.Name = strings.TrimSpace(req.Name)
	req.Category = strings.TrimSpace(req.Category)
	req.RuleType = strings.TrimSpace(req.RuleType)
	req.Pattern = strings.TrimSpace(req.Pattern)
	req.Action = strings.TrimSpace(req.Action)

	if req.Name == "" || req.Category == "" || req.RuleType == "" || req.Pattern == "" || req.Action == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	categoryValid := map[string]bool{
		"keyword":            true,
		"rate_limit":         true,
		"device_fingerprint": true,
		"behavior":           true,
	}
	if !categoryValid[req.Category] {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	actionValid := map[string]bool{
		"reject":        true,
		"quarantine":    true,
		"notify_admin":  true,
		"auto_ban":      true,
	}
	if !actionValid[req.Action] {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	if req.DurationSec < 0 {
		req.DurationSec = 0
	}

	r := model.RiskRule{
		Name:        req.Name,
		Category:    req.Category,
		RuleType:    req.RuleType,
		Pattern:     req.Pattern,
		Action:      req.Action,
		DurationSec: req.DurationSec,
		Enabled:     true,
		Priority:    req.Priority,
	}
	if err := a.DB.Create(&r).Error; err != nil {
		a.Log.Error("create risk rule", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin created risk rule",
		zap.Uint64("rule_id", r.ID),
		zap.String("name", r.Name),
		zap.Uint64("admin_id", adminID),
	)

	resp.OK(c, r)
}

// AdminUpdateRiskRule PUT /api/v1/admin/risk/rules/:id
func (a *API) AdminUpdateRiskRule(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var req struct {
		Name        string `json:"name"`
		Category    string `json:"category"`
		RuleType    string `json:"rule_type"`
		Pattern     string `json:"pattern"`
		Action      string `json:"action"`
		DurationSec *int   `json:"duration_sec"`
		Priority    *int   `json:"priority"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var r model.RiskRule
	if err := a.DB.First(&r, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	updates := map[string]interface{}{}

	if v := strings.TrimSpace(req.Name); v != "" {
		updates["name"] = v
	}
	if v := strings.TrimSpace(req.Category); v != "" {
		categoryValid := map[string]bool{
			"keyword": true, "rate_limit": true, "device_fingerprint": true, "behavior": true,
		}
		if !categoryValid[v] {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		updates["category"] = v
	}
	if v := strings.TrimSpace(req.RuleType); v != "" {
		updates["rule_type"] = v
	}
	if v := strings.TrimSpace(req.Pattern); v != "" {
		updates["pattern"] = v
	}
	if v := strings.TrimSpace(req.Action); v != "" {
		actionValid := map[string]bool{
			"reject": true, "quarantine": true, "notify_admin": true, "auto_ban": true,
		}
		if !actionValid[v] {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		updates["action"] = v
	}
	if req.DurationSec != nil {
		d := *req.DurationSec
		if d < 0 {
			d = 0
		}
		updates["duration_sec"] = d
	}
	if req.Priority != nil {
		updates["priority"] = *req.Priority
	}

	if len(updates) == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	updates["updated_at"] = time.Now()

	if err := a.DB.Model(&r).Updates(updates).Error; err != nil {
		a.Log.Error("update risk rule", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin updated risk rule",
		zap.Uint64("rule_id", id),
		zap.Uint64("admin_id", adminID),
	)

	// Reload
	a.DB.First(&r, id)
	resp.OK(c, r)
}

// AdminDeleteRiskRule DELETE /api/v1/admin/risk/rules/:id
func (a *API) AdminDeleteRiskRule(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var r model.RiskRule
	if err := a.DB.First(&r, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if err := a.DB.Delete(&r).Error; err != nil {
		a.Log.Error("delete risk rule", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin deleted risk rule",
		zap.Uint64("rule_id", id),
		zap.String("name", r.Name),
		zap.Uint64("admin_id", adminID),
	)

	resp.OK(c, nil)
}

// AdminToggleRiskRule POST /api/v1/admin/risk/rules/:id/toggle
func (a *API) AdminToggleRiskRule(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var r model.RiskRule
	if err := a.DB.First(&r, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	newEnabled := !r.Enabled
	if err := a.DB.Model(&r).Updates(map[string]interface{}{
		"enabled":    newEnabled,
		"updated_at": time.Now(),
	}).Error; err != nil {
		a.Log.Error("toggle risk rule", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin toggled risk rule",
		zap.Uint64("rule_id", id),
		zap.Bool("enabled", newEnabled),
		zap.Uint64("admin_id", adminID),
	)

	resp.OK(c, gin.H{"id": id, "enabled": newEnabled})
}

// ---------- Black/White List ----------

// AdminListBWList GET /api/v1/admin/risk/bw-list
func (a *API) AdminListBWList(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	listType := strings.TrimSpace(c.Query("list_type"))
	target := strings.TrimSpace(c.Query("target"))

	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tx := a.DB.Model(&model.BlackWhiteList{})
	if listType != "" {
		tx = tx.Where("list_type = ?", listType)
	}
	if target != "" {
		tx = tx.Where("target LIKE ?", "%"+target+"%")
	}

	var total int64
	tx.Count(&total)

	var entries []model.BlackWhiteList
	tx.Order("created_at DESC").Offset((page - 1) * pageSize).Limit(pageSize).Find(&entries)

	// Batch load creator info
	creatorSet := make(map[uint64]bool)
	for _, e := range entries {
		creatorSet[e.CreatedBy] = true
	}
	creatorBriefs := loadUserBriefs(a.DB, creatorSet)

	type item struct {
		ID        uint64     `json:"id"`
		ListType  string     `json:"list_type"`
		Target    string     `json:"target"`
		Reason    string     `json:"reason"`
		ExpiresAt *time.Time `json:"expires_at"`
		CreatedBy uint64     `json:"created_by"`
		Creator   gin.H      `json:"creator"`
		CreatedAt time.Time  `json:"created_at"`
	}
	items := make([]item, 0, len(entries))
	for _, e := range entries {
		items = append(items, item{
			ID:        e.ID,
			ListType:  e.ListType,
			Target:    e.Target,
			Reason:    e.Reason,
			ExpiresAt: e.ExpiresAt,
			CreatedBy: e.CreatedBy,
			Creator:   creatorBriefs[e.CreatedBy],
			CreatedAt: e.CreatedAt,
		})
	}

	// Counts
	var blackCount, whiteCount int64
	a.DB.Model(&model.BlackWhiteList{}).Where("list_type = 'blacklist'").Count(&blackCount)
	a.DB.Model(&model.BlackWhiteList{}).Where("list_type = 'whitelist'").Count(&whiteCount)

	resp.OK(c, gin.H{
		"items":        items,
		"total":        total,
		"page":         page,
		"page_size":    pageSize,
		"black_count":  blackCount,
		"white_count":  whiteCount,
	})
}

// AdminCreateBWEntry POST /api/v1/admin/risk/bw-list
func (a *API) AdminCreateBWEntry(c *gin.Context) {
	adminID, _ := middleware.AdminID(c)

	var req struct {
		ListType  string `json:"list_type"`
		Target    string `json:"target"`
		Reason    string `json:"reason"`
		ExpiresAt *time.Time `json:"expires_at"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.ListType = strings.TrimSpace(req.ListType)
	req.Target = strings.TrimSpace(req.Target)
	req.Reason = strings.TrimSpace(req.Reason)

	if req.ListType != "blacklist" && req.ListType != "whitelist" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.Target == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if len([]rune(req.Reason)) > 200 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	entry := model.BlackWhiteList{
		ListType:  req.ListType,
		Target:    req.Target,
		Reason:    req.Reason,
		ExpiresAt: req.ExpiresAt,
		CreatedBy: adminID,
	}
	if err := a.DB.Create(&entry).Error; err != nil {
		a.Log.Error("create bw entry", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin created bw entry",
		zap.Uint64("entry_id", entry.ID),
		zap.String("list_type", entry.ListType),
		zap.String("target", entry.Target),
		zap.Uint64("admin_id", adminID),
	)

	resp.OK(c, gin.H{
		"id":         entry.ID,
		"list_type":  entry.ListType,
		"target":     entry.Target,
		"reason":     entry.Reason,
		"expires_at": entry.ExpiresAt,
		"created_by": entry.CreatedBy,
		"created_at": entry.CreatedAt,
	})
}

// AdminDeleteBWEntry DELETE /api/v1/admin/risk/bw-list/:id
func (a *API) AdminDeleteBWEntry(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var entry model.BlackWhiteList
	if err := a.DB.First(&entry, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if err := a.DB.Delete(&entry).Error; err != nil {
		a.Log.Error("delete bw entry", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin deleted bw entry",
		zap.Uint64("entry_id", id),
		zap.String("list_type", entry.ListType),
		zap.String("target", entry.Target),
		zap.Uint64("admin_id", adminID),
	)

	resp.OK(c, nil)
}

// ─── P0: 风控执行引擎 ───

// ScanContentRisk checks text against all enabled risk rules and logs hits.
func (a *API) ScanContentRisk(targetType string, targetID uint64, text string) bool {
	if text == "" { return false }
	var rules []model.RiskRule
	if err := a.DB.Where("enabled = 1").Order("priority DESC").Find(&rules).Error; err != nil {
		return false
	}
	blocked := false
	for _, r := range rules {
		if r.Pattern == "" { continue }
		matched := strings.Contains(strings.ToLower(text), strings.ToLower(r.Pattern))
		if !matched { continue }
		a.DB.Create(&model.RiskHitLog{
			RuleID: r.ID, RuleName: r.Name,
			TargetID: targetID, TargetType: targetType,
			MatchText: text[:min(len(text), 200)], Action: r.Action,
		})
		if r.Action == "reject" || r.Action == "auto_ban" {
			a.notifyRiskHit(targetType, targetID, r.Name, r.Action)
			blocked = true
		}
		if r.Action == "quarantine" && targetType == "comment" {
			a.DB.Model(&model.Comment{}).Where("id = ?", targetID).Update("approved", false)
			a.notifyRiskHit(targetType, targetID, r.Name, "quarantine")
		}
		// notify_admin: push alert to connected admin Dashboards
		if r.Action == "notify_admin" {
			a.notifyAdminRiskAlert(r.Name, text[:min(len(text), 100)], targetType, targetID)
		}
	}
	return blocked
}

// notifyRiskHit sends a notification to the content creator when risk hits.
func (a *API) notifyRiskHit(targetType string, targetID uint64, ruleName, action string) {
	var ownerID uint64
	switch targetType {
	case "comment":
		var c model.Comment
		if err := a.DB.Select("user_id").First(&c, targetID).Error; err == nil {
			ownerID = c.UserID
		}
	case "danmaku":
		// Danmaku doesn't have user lookup easily accessible here; skip for now
		return
	}
	if ownerID == 0 {
		return
	}
	actionText := "已被拦截"
	if action == "quarantine" {
		actionText = "已被隐藏待审核"
	}
	_ = a.DB.Create(&model.NotificationRecord{
		RecipientID: ownerID, RecipientType: "user",
		Channel: "in_app", Title: "内容风控通知",
		Content: "您发布的内容因违反「" + ruleName + "」规则，"+ actionText + "。",
		RelatedType: targetType, RelatedID: targetID, Status: "pending",
	})
}

// notifyAdminRiskAlert pushes a risk alert to admins via ChatHub.
func (a *API) notifyAdminRiskAlert(ruleName string, matchedText string, targetType string, targetID uint64) {
	// Push to admin notification channel — any connected admin sees this
	// For MVP, we store a notification record visible to all admins
	_ = a.DB.Create(&model.NotificationRecord{
		RecipientID: 0, RecipientType: "admin",
		Channel: "in_app", Title: "风控告警：" + ruleName,
		Content: "触发内容：" + matchedText + "（类型：" + targetType + "，ID：" + strconv.FormatUint(targetID, 10) + "）",
		RelatedType: targetType, RelatedID: targetID, Status: "pending",
	})
	if a.ChatHub != nil {
		a.ChatHub.PushJSON(0, gin.H{
			"type": "admin_alert",
			"data": gin.H{"rule": ruleName, "target_type": targetType, "target_id": targetID},
		})
	}
}

// AdminGetRiskStats GET /admin/risk/stats
// Returns risk hit trend data: hit rate, top rules, time series.
func (a *API) AdminGetRiskStats(c *gin.Context) {
	days := c.DefaultQuery("days", "7")

	var totalHits int64
	var uniqueRules int64
	a.DB.Model(&model.RiskHitLog{}).Count(&totalHits)
	a.DB.Raw(`SELECT COUNT(DISTINCT rule_name) FROM risk_hit_logs`).Scan(&uniqueRules)

	// Top 10 triggered rules
	var topRules []gin.H
	rows, _ := a.DB.Raw(`
		SELECT rule_name, action, COUNT(*) as hit_count, MAX(created_at) as last_hit
		FROM risk_hit_logs
		WHERE created_at >= DATE_SUB(NOW(), INTERVAL ? DAY)
		GROUP BY rule_name, action
		ORDER BY hit_count DESC
		LIMIT 10
	`, days).Rows()
	if rows != nil {
		for rows.Next() {
			var ruleName, action string
			var hitCount int64
			var lastHit time.Time
			rows.Scan(&ruleName, &action, &hitCount, &lastHit)
			topRules = append(topRules, gin.H{
				"rule_name": ruleName, "action": action,
				"hit_count": hitCount, "last_hit": lastHit,
			})
		}
		rows.Close()
	}

	// Daily trend (hits per day)
	var dailyTrend []gin.H
	rows2, _ := a.DB.Raw(`
		SELECT DATE(created_at) as date, COUNT(*) as count
		FROM risk_hit_logs
		WHERE created_at >= DATE_SUB(NOW(), INTERVAL ? DAY)
		GROUP BY DATE(created_at)
		ORDER BY date ASC
	`, days).Rows()
	if rows2 != nil {
		for rows2.Next() {
			var date string
			var count int64
			rows2.Scan(&date, &count)
			dailyTrend = append(dailyTrend, gin.H{"date": date, "count": count})
		}
		rows2.Close()
	}

	resp.OK(c, gin.H{
		"total_hits":   totalHits,
		"unique_rules": uniqueRules,
		"top_rules":    topRules,
		"daily_trend":  dailyTrend,
	})
}

func min(a, b int) int { if a < b { return a }; return b }
