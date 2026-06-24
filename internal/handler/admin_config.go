package handler

import (
	"encoding/json"
	"hash/fnv"
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ──────────────────────────────────────────────
// Module 21: Release & Config Management
// ──────────────────────────────────────────────

// AdminListFeatureFlags GET /admin/config/feature-flags — list feature flags
func (a *API) AdminListFeatureFlags(c *gin.Context) {
	var flags []model.FeatureFlag
	q := a.DB.Model(&model.FeatureFlag{})
	if enabled := c.Query("enabled"); enabled == "true" {
		q = q.Where("enabled = ?", true)
	} else if enabled == "false" {
		q = q.Where("enabled = ?", false)
	}
	if err := q.Order("created_at DESC, id DESC").Find(&flags).Error; err != nil {
		a.Log.Error("list feature flags", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(flags))
	for i := range flags {
		items = append(items, gin.H{
			"id":          flags[i].ID,
			"key":         flags[i].Key,
			"description": flags[i].Description,
			"enabled":     flags[i].Enabled,
			"rollout_pct": flags[i].RolloutPct,
			"whitelist":   flags[i].Whitelist,
			"created_at":  flags[i].CreatedAt,
			"updated_at":  flags[i].UpdatedAt,
		})
	}
	resp.OK(c, gin.H{"items": items})
}

type featureFlagReq struct {
	Key         string `json:"key"`
	Description string `json:"description"`
	Enabled     *bool  `json:"enabled"`
	RolloutPct  *int   `json:"rollout_pct"`
	Whitelist   string `json:"whitelist"` // JSON array of user IDs
}

// AdminCreateFeatureFlag POST /admin/config/feature-flags — create feature flag
func (a *API) AdminCreateFeatureFlag(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var req featureFlagReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Key = strings.TrimSpace(req.Key)
	if req.Key == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	// Validate whitelist JSON if provided.
	if strings.TrimSpace(req.Whitelist) != "" {
		var tmp []interface{}
		if err := json.Unmarshal([]byte(req.Whitelist), &tmp); err != nil {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
	}
	flag := model.FeatureFlag{
		Key:         req.Key,
		Description: req.Description,
		Enabled:     false,
		RolloutPct:  0,
		Whitelist:   req.Whitelist,
	}
	if req.Enabled != nil {
		flag.Enabled = *req.Enabled
	}
	if req.RolloutPct != nil {
		if *req.RolloutPct < 0 || *req.RolloutPct > 100 {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		flag.RolloutPct = *req.RolloutPct
	}
	if err := a.DB.Create(&flag).Error; err != nil {
		a.Log.Error("create feature flag", zap.Error(err), zap.String("key", flag.Key))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_feature_flag", "feature_flag", flag.ID, `{"key":"`+flag.Key+`"}`)
	resp.OK(c, gin.H{
		"id":          flag.ID,
		"key":         flag.Key,
		"description": flag.Description,
		"enabled":     flag.Enabled,
		"rollout_pct": flag.RolloutPct,
		"whitelist":   flag.Whitelist,
	})
}

// AdminUpdateFeatureFlag PUT /admin/config/feature-flags/:id — update feature flag
func (a *API) AdminUpdateFeatureFlag(c *gin.Context) {
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
	var req featureFlagReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var flag model.FeatureFlag
	if err := a.DB.First(&flag, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	updates := map[string]interface{}{}
	if strings.TrimSpace(req.Key) != "" {
		updates["key"] = strings.TrimSpace(req.Key)
	}
	updates["description"] = req.Description
	if req.Enabled != nil {
		updates["enabled"] = *req.Enabled
	}
	if req.RolloutPct != nil {
		if *req.RolloutPct < 0 || *req.RolloutPct > 100 {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		updates["rollout_pct"] = *req.RolloutPct
	}
	if strings.TrimSpace(req.Whitelist) != "" {
		var tmp []interface{}
		if err := json.Unmarshal([]byte(req.Whitelist), &tmp); err != nil {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		updates["whitelist"] = req.Whitelist
	}
	if err := a.DB.Model(&flag).Updates(updates).Error; err != nil {
		a.Log.Error("update feature flag", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "update_feature_flag", "feature_flag", id, "")
	resp.OK(c, gin.H{"id": id, "ok": true})
}

// AdminToggleFeatureFlag POST /admin/config/feature-flags/:id/toggle — enable/disable
func (a *API) AdminToggleFeatureFlag(c *gin.Context) {
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
	var flag model.FeatureFlag
	if err := a.DB.First(&flag, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	newEnabled := !flag.Enabled
	if err := a.DB.Model(&flag).Update("enabled", newEnabled).Error; err != nil {
		a.Log.Error("toggle feature flag", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "toggle_feature_flag", "feature_flag", id, `{"enabled":`+strconv.FormatBool(newEnabled)+`}`)
	resp.OK(c, gin.H{"id": id, "enabled": newEnabled})
}

// AdminCheckFeatureFlag GET /config/feature-flags/:key — public check if a feature is enabled
// Logic: enabled=true OR user_id in whitelist OR rollout_pct covers user → enabled=true
func (a *API) AdminCheckFeatureFlag(c *gin.Context) {
	key := strings.TrimSpace(c.Param("key"))
	if key == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var flag model.FeatureFlag
	if err := a.DB.Where("key = ?", key).First(&flag).Error; err != nil {
		// Unknown flag defaults to disabled (fail-closed).
		resp.OK(c, gin.H{"key": key, "enabled": false})
		return
	}

	enabled := false
	reason := "disabled"

	// 1. Global enabled toggle.
	if flag.Enabled {
		enabled = true
		reason = "enabled"
	}

	// 2. Whitelist: if the current user (optional JWT) is in the whitelist.
	uid, hasUser := middleware.UserID(c)
	if !enabled && hasUser && strings.TrimSpace(flag.Whitelist) != "" {
		if inWhitelist(flag.Whitelist, uid) {
			enabled = true
			reason = "whitelist"
		}
	}

	// 3. Rollout percentage: hash the user_id deterministically; covered if hash%100 < rollout_pct.
	if !enabled && hasUser && flag.RolloutPct > 0 {
		if rolloutCoversUser(uid, flag.RolloutPct) {
			enabled = true
			reason = "rollout"
		}
	}

	resp.OK(c, gin.H{
		"key":     flag.Key,
		"enabled": enabled,
		"reason":  reason,
	})
}

// inWhitelist checks whether uid appears in the whitelist JSON array.
// The array may contain numbers or strings; both are matched.
func inWhitelist(whitelistJSON string, uid uint64) bool {
	raw := strings.TrimSpace(whitelistJSON)
	if raw == "" {
		return false
	}
	var arr []json.Number
	if err := json.Unmarshal([]byte(raw), &arr); err != nil {
		// Fallback: try generic interface array.
		var generic []interface{}
		if err := json.Unmarshal([]byte(raw), &generic); err != nil {
			return false
		}
		uidStr := strconv.FormatUint(uid, 10)
		for i := range generic {
			switch v := generic[i].(type) {
			case float64:
				if uint64(v) == uid {
					return true
				}
			case string:
				if v == uidStr {
					return true
				}
			}
		}
		return false
	}
	uidStr := strconv.FormatUint(uid, 10)
	for i := range arr {
		s := string(arr[i])
		if s == uidStr {
			return true
		}
	}
	return false
}

// rolloutCoversUser uses FNV-1a hash of the user_id to produce a stable 0–99 bucket.
// The user is covered when the bucket < rolloutPct.
func rolloutCoversUser(uid uint64, rolloutPct int) bool {
	h := fnv.New32a()
	_, _ = h.Write([]byte(strconv.FormatUint(uid, 10)))
	bucket := int(h.Sum32() % 100)
	return bucket < rolloutPct
}

// AdminListReleases GET /admin/config/releases — list release records
func (a *API) AdminListReleases(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}
	q := a.DB.Model(&model.ReleaseRecord{})
	if status := strings.TrimSpace(c.Query("status")); status != "" {
		q = q.Where("status = ?", status)
	}
	var total int64
	if err := q.Count(&total).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	offset := (page - 1) * pageSize
	var rows []model.ReleaseRecord
	if err := q.Order("created_at DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":            rows[i].ID,
			"version":       rows[i].Version,
			"description":   rows[i].Description,
			"status":        rows[i].Status,
			"deployed_by":   rows[i].DeployedBy,
			"rolled_back_by": rows[i].RolledBackBy,
			"created_at":    rows[i].CreatedAt,
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

type releaseReq struct {
	Version     string `json:"version"`
	Description string `json:"description"`
}

// AdminCreateRelease POST /admin/config/releases — record a new release
func (a *API) AdminCreateRelease(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var req releaseReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if strings.TrimSpace(req.Version) == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	// Mark previously active releases as rolled_back (only one active at a time).
	_ = a.DB.Model(&model.ReleaseRecord{}).Where("status = ?", "active").Update("status", "rolled_back").Error

	rec := model.ReleaseRecord{
		Version:     strings.TrimSpace(req.Version),
		Description: req.Description,
		Status:      "active",
		DeployedBy:  adminID,
	}
	if err := a.DB.Create(&rec).Error; err != nil {
		a.Log.Error("create release record", zap.Error(err), zap.String("version", rec.Version))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_release", "release", rec.ID, `{"version":"`+rec.Version+`"}`)
	resp.OK(c, gin.H{
		"id":          rec.ID,
		"version":     rec.Version,
		"description": rec.Description,
		"status":      rec.Status,
		"deployed_by": rec.DeployedBy,
		"created_at":  rec.CreatedAt,
	})
}

// AdminRollbackRelease POST /admin/config/releases/:id/rollback — rollback to a previous release
func (a *API) AdminRollbackRelease(c *gin.Context) {
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
	var rec model.ReleaseRecord
	if err := a.DB.First(&rec, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if rec.Status == "rolled_back" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Demote any currently-active release, then mark the target as active again.
	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Model(&model.ReleaseRecord{}).Where("status = ? AND id != ?", "active", id).
			Update("status", "rolled_back").Error; err != nil {
			return err
		}
		return tx.Model(&rec).Updates(map[string]interface{}{
			"status":         "active",
			"rolled_back_by": adminID,
		}).Error
	}); err != nil {
		a.Log.Error("rollback release", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "rollback_release", "release", id, `{"version":"`+rec.Version+`"}`)
	resp.OK(c, gin.H{"id": id, "version": rec.Version, "status": "active", "rolled_back_by": adminID})
}
