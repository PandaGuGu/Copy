package handler

import (
	"encoding/json"
	"net/http"
	"regexp"
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
		ID         uint64     `json:"id"`
		ListType   string     `json:"list_type"`
		TargetType string     `json:"target_type"`
		Target     string     `json:"target"`
		Reason     string     `json:"reason"`
		ExpiresAt  *time.Time `json:"expires_at"`
		CreatedBy  uint64     `json:"created_by"`
		Creator    gin.H      `json:"creator"`
		CreatedAt  time.Time  `json:"created_at"`
	}
	items := make([]item, 0, len(entries))
	for _, e := range entries {
		items = append(items, item{
			ID:         e.ID,
			ListType:   e.ListType,
			TargetType: e.TargetType,
			Target:     e.Target,
			Reason:     e.Reason,
			ExpiresAt:  e.ExpiresAt,
			CreatedBy:  e.CreatedBy,
			Creator:    creatorBriefs[e.CreatedBy],
			CreatedAt:  e.CreatedAt,
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
		ListType   string     `json:"list_type"`
		TargetType string     `json:"target_type"`
		Target     string     `json:"target"`
		Reason     string     `json:"reason"`
		ExpiresAt  *time.Time `json:"expires_at"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.ListType = strings.TrimSpace(req.ListType)
	req.Target = strings.TrimSpace(req.Target)
	req.Reason = strings.TrimSpace(req.Reason)
	targetType := strings.TrimSpace(req.TargetType)
	if targetType == "" {
		targetType = "user"
	}
	targetTypeValid := map[string]bool{"user": true, "ip": true, "device": true, "content": true}
	if !targetTypeValid[targetType] {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

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
		ListType:   req.ListType,
		TargetType: targetType,
		Target:     req.Target,
		Reason:     req.Reason,
		ExpiresAt:  req.ExpiresAt,
		CreatedBy:  adminID,
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
		"id":          entry.ID,
		"list_type":   entry.ListType,
		"target_type": entry.TargetType,
		"target":      entry.Target,
		"reason":      entry.Reason,
		"expires_at":  entry.ExpiresAt,
		"created_by":  entry.CreatedBy,
		"created_at":  entry.CreatedAt,
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

// AdminUpdateBWEntry PUT /api/v1/admin/risk/bw-list/:id
func (a *API) AdminUpdateBWEntry(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var req struct {
		ListType   string     `json:"list_type"`
		TargetType string     `json:"target_type"`
		Target     string     `json:"target"`
		Reason     string     `json:"reason"`
		ExpiresAt  *time.Time `json:"expires_at"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var entry model.BlackWhiteList
	if err := a.DB.First(&entry, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	updates := map[string]interface{}{}
	if v := strings.TrimSpace(req.ListType); v != "" {
		if v != "blacklist" && v != "whitelist" {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		updates["list_type"] = v
	}
	if v := strings.TrimSpace(req.TargetType); v != "" {
		targetTypeValid := map[string]bool{"user": true, "ip": true, "device": true, "content": true}
		if !targetTypeValid[v] {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		updates["target_type"] = v
	}
	if v := strings.TrimSpace(req.Target); v != "" {
		updates["target"] = v
	}
	if v := strings.TrimSpace(req.Reason); v != "" {
		updates["reason"] = v
	}
	if req.ExpiresAt != nil {
		updates["expires_at"] = req.ExpiresAt
	}

	if len(updates) > 0 {
		if err := a.DB.Model(&entry).Updates(updates).Error; err != nil {
			a.Log.Error("update bw entry", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	}

	a.Log.Info("admin updated bw entry",
		zap.Uint64("entry_id", id),
		zap.Uint64("admin_id", adminID),
	)

	a.DB.First(&entry, id)
	resp.OK(c, gin.H{
		"id":          entry.ID,
		"list_type":   entry.ListType,
		"target_type": entry.TargetType,
		"target":      entry.Target,
		"reason":      entry.Reason,
		"expires_at":  entry.ExpiresAt,
		"created_by":  entry.CreatedBy,
		"created_at":  entry.CreatedAt,
	})
}

// ─── P0: 风控执行引擎 ───

// ScanContentRisk checks text against all enabled risk rules, black/white lists,
// and executes corresponding actions. Returns true if the content should be blocked.
func (a *API) ScanContentRisk(targetType string, targetID uint64, text string) bool {
	if text == "" {
		return false
	}

	// 1. Resolve owner user ID for black/white list lookups.
	ownerID := a.resolveContentOwner(targetType, targetID)

	// 2. Check whitelist — whitelisted users bypass ALL rules.
	if ownerID > 0 && a.isWhitelisted(ownerID) {
		return false
	}

	// 3. Check blacklist — if owner is blacklisted, apply blocking action immediately.
	if ownerID > 0 && a.isBlacklisted(ownerID) {
		a.DB.Create(&model.RiskHitLog{
			RuleID: 0, RuleName: "blacklist",
			TargetID: targetID, TargetType: targetType,
			MatchText: text[:min(len(text), 200)], Action: "reject",
		})
		a.notifyRiskHit(targetType, targetID, "黑名单规则", "reject")
		return true
	}

	// 4. Load enabled rules ordered by priority.
	var rules []model.RiskRule
	if err := a.DB.Where("enabled = 1").Order("priority DESC").Find(&rules).Error; err != nil {
		return false
	}

	// 5. Compile regex patterns once (cache keyed by rule ID).
	regexCache := make(map[uint64]*regexp.Regexp)

	blocked := false
	for _, r := range rules {
		if r.Pattern == "" {
			continue
		}

		matched := false

		switch r.Category {
		case "rate_limit":
			matched = a.checkRateLimit(r, ownerID)
		default:
			// keyword / behavior / device_fingerprint — all use pattern matching
			matched = a.matchPattern(r, text, regexCache)
		}

		if !matched {
			continue
		}

		// Log the hit.
		a.DB.Create(&model.RiskHitLog{
			RuleID: r.ID, RuleName: r.Name,
			TargetID: targetID, TargetType: targetType,
			MatchText: text[:min(len(text), 200)], Action: r.Action,
		})

		switch r.Action {
		case "reject":
			a.notifyRiskHit(targetType, targetID, r.Name, "reject")
			blocked = true
		case "auto_ban":
			a.notifyRiskHit(targetType, targetID, r.Name, "auto_ban")
			if ownerID > 0 {
				a.banUser(ownerID, r.Name, time.Duration(r.DurationSec)*time.Second)
			}
			blocked = true
		case "quarantine":
			if targetType == "comment" {
				a.DB.Model(&model.Comment{}).Where("id = ?", targetID).Update("approved", false)
			}
			if targetType == "article_comment" {
				a.DB.Model(&model.ArticleComment{}).Where("id = ?", targetID).Update("approved", false)
			}
			if targetType == "dynamic_comment" {
				a.DB.Model(&model.DynamicComment{}).Where("id = ?", targetID).Update("approved", false)
			}
			a.notifyRiskHit(targetType, targetID, r.Name, "quarantine")
		case "notify_admin":
			a.notifyAdminRiskAlert(r.Name, text[:min(len(text), 100)], targetType, targetID)
		}
	}
	return blocked
}

// matchPattern determines whether a rule's pattern matches the given text.
// Patterns delimited by / are treated as regex; otherwise plain substring.
func (a *API) matchPattern(r model.RiskRule, text string, cache map[uint64]*regexp.Regexp) bool {
	pattern := strings.TrimSpace(r.Pattern)
	if len(pattern) >= 2 && pattern[0] == '/' && pattern[len(pattern)-1] == '/' {
		// Regex pattern: /pattern/
		expr := pattern[1 : len(pattern)-1]
		re, ok := cache[r.ID]
		if !ok {
			var err error
			re, err = regexp.Compile("(?i)" + expr)
			if err != nil {
				// Fall back to substring match on the raw expression.
				return strings.Contains(strings.ToLower(text), strings.ToLower(expr))
			}
			cache[r.ID] = re
		}
		return re.MatchString(text)
	}
	// Plain substring match (case-insensitive).
	return strings.Contains(strings.ToLower(text), strings.ToLower(pattern))
}

// checkRateLimit checks whether the user has exceeded the rate limit defined by the rule.
// Pattern format: {"max_count": 5, "window_sec": 3600}
// Uses risk_rate_counters table for accurate per-window tracking.
func (a *API) checkRateLimit(r model.RiskRule, userID uint64) bool {
	if userID == 0 {
		return false
	}

	type rateConfig struct {
		MaxCount  int `json:"max_count"`
		WindowSec int `json:"window_sec"`
	}
	var cfg rateConfig
	if err := json.Unmarshal([]byte(r.Pattern), &cfg); err != nil {
		return false
	}
	if cfg.MaxCount <= 0 || cfg.WindowSec <= 0 {
		return false
	}

	windowDur := time.Duration(cfg.WindowSec) * time.Second
	now := time.Now()

	// Find or create counter for this rule+user in current window
	var counter model.RiskRateCounter
	err := a.DB.Where("rule_id = ? AND user_id = ? AND window_start <= ?", r.ID, userID, now).
		Order("window_start DESC").
		First(&counter).Error

	if err != nil || counter.WindowStart.Add(windowDur).Before(now) {
		// Start new window
		windowStart := now.Truncate(windowDur)
		counter = model.RiskRateCounter{
			RuleID:      r.ID,
			UserID:      userID,
			WindowStart: windowStart,
			Count:       0,
		}
		a.DB.Create(&counter)
	}

	if counter.Count >= cfg.MaxCount {
		return true
	}

	// Increment
	a.DB.Model(&counter).Update("count", counter.Count+1)
	return false
}

// resolveContentOwner resolves the owner user ID for a given content target.
func (a *API) resolveContentOwner(targetType string, targetID uint64) uint64 {
	switch targetType {
	case "comment":
		var c model.Comment
		if err := a.DB.Select("user_id").First(&c, targetID).Error; err == nil {
			return c.UserID
		}
	case "article_comment":
		var ac model.ArticleComment
		if err := a.DB.Select("user_id").First(&ac, targetID).Error; err == nil {
			return ac.UserID
		}
	case "dynamic_comment":
		var dc model.DynamicComment
		if err := a.DB.Select("user_id").First(&dc, targetID).Error; err == nil {
			return dc.UserID
		}
	case "video":
		var v model.Video
		if err := a.DB.Select("user_id").First(&v, targetID).Error; err == nil {
			return v.UserID
		}
	case "danmaku":
		var d model.Danmaku
		if err := a.DB.Select("user_id").First(&d, targetID).Error; err == nil {
			return d.UserID
		}
	}
	return 0
}

// isWhitelisted returns true if the user is in the whitelist and not expired.
func (a *API) isWhitelisted(userID uint64) bool {
	var count int64
	a.DB.Model(&model.BlackWhiteList{}).
		Where("list_type = 'whitelist' AND target_type = 'user' AND target = ?", strconv.FormatUint(userID, 10)).
		Where("expires_at IS NULL OR expires_at > ?", time.Now()).
		Count(&count)
	return count > 0
}

// isBlacklisted returns true if the user is in the blacklist and not expired.
func (a *API) isBlacklisted(userID uint64) bool {
	var count int64
	a.DB.Model(&model.BlackWhiteList{}).
		Where("list_type = 'blacklist' AND target_type = 'user' AND target = ?", strconv.FormatUint(userID, 10)).
		Where("expires_at IS NULL OR expires_at > ?", time.Now()).
		Count(&count)
	return count > 0
}

// banUser sets the user status to banned with optional expiry.
func (a *API) banUser(userID uint64, reason string, duration time.Duration) {
	now := time.Now()
	updates := map[string]interface{}{
		"status":        "banned",
		"banned_at":     now,
		"banned_reason": reason,
	}
	if duration > 0 {
		exp := now.Add(duration)
		updates["ban_expires_at"] = exp
	} else {
		updates["ban_expires_at"] = nil // permanent
	}
	if err := a.DB.Model(&model.User{}).Where("id = ?", userID).Updates(updates).Error; err != nil {
		a.Log.Error("ban user failed", zap.Uint64("user_id", userID), zap.Error(err))
		return
	}
	a.Log.Info("user banned by risk engine",
		zap.Uint64("user_id", userID),
		zap.String("reason", reason),
		zap.Duration("duration", duration),
	)
}

// CleanExpiredBWEntries removes expired black/white list entries.
func (a *API) CleanExpiredBWEntries() error {
	res := a.DB.Where("expires_at IS NOT NULL AND expires_at <= ?", time.Now()).
		Delete(&model.BlackWhiteList{})
	if res.Error != nil {
		return res.Error
	}
	if res.RowsAffected > 0 {
		a.Log.Info("cleaned expired bw-list entries", zap.Int64("count", res.RowsAffected))
	}
	return nil
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
