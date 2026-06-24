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
