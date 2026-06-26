package handler

import (
	"encoding/json"
	"fmt"
	"hash/fnv"
	"net/http"
	"strconv"
	"strings"
	"time"

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

// AdminDeleteFeatureFlag DELETE /admin/config/feature-flags/:id — delete feature flag
func (a *API) AdminDeleteFeatureFlag(c *gin.Context) {
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
	if err := a.DB.Delete(&flag).Error; err != nil {
		a.Log.Error("delete feature flag", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "delete_feature_flag", "feature_flag", id, `{"key":"`+flag.Key+`"}`)
	resp.OK(c, gin.H{"id": id, "ok": true})
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
	if err := a.DB.Where("`key` = ?", key).First(&flag).Error; err != nil {
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
		hasSnapshot := len(rows[i].Snapshot) > 0
		items = append(items, gin.H{
			"id":            rows[i].ID,
			"version":       rows[i].Version,
			"title":         rows[i].Title,
			"type":          rows[i].Type,
			"description":   rows[i].Description,
			"notes":         rows[i].Notes,
			"status":        rows[i].Status,
			"deployed_by":   rows[i].DeployedBy,
			"pushed_by":     rows[i].PushedBy,
			"rolled_back_by": rows[i].RolledBackBy,
			"has_snapshot":  hasSnapshot,
			"released_at":   rows[i].ReleasedAt,
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
	Title       string `json:"title"`
	Type        string `json:"type"`     // canary / full / hotfix
	Description string `json:"description"`
	Notes       string `json:"notes"`    // release notes / markdown
}

// AdminCreateRelease POST /admin/config/releases — record a new release with auto-snapshot
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
	releaseType := strings.TrimSpace(req.Type)
	if releaseType == "" {
		releaseType = "canary"
	}
	if releaseType != "canary" && releaseType != "full" && releaseType != "hotfix" {
		releaseType = "canary"
	}

	// Auto-snapshot all current feature flags
	snapshot := a.buildConfigSnapshot(strings.TrimSpace(req.Version))
	raw, _ := json.Marshal(snapshot)

	rec := model.ReleaseRecord{
		Version:     strings.TrimSpace(req.Version),
		Title:       strings.TrimSpace(req.Title),
		Type:        releaseType,
		Description: req.Description,
		Notes:       req.Notes,
		Status:      "draft",
		Snapshot:    string(raw),
		DeployedBy:  adminID,
	}
	if err := a.DB.Create(&rec).Error; err != nil {
		a.Log.Error("create release record", zap.Error(err), zap.String("version", rec.Version))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_release", "release", rec.ID, `{"version":"`+rec.Version+`"}`)
	resp.OK(c, gin.H{
		"id":           rec.ID,
		"version":      rec.Version,
		"title":        rec.Title,
		"type":         rec.Type,
		"description":  rec.Description,
		"notes":        rec.Notes,
		"has_snapshot": len(rec.Snapshot) > 0,
		"status":       rec.Status,
		"deployed_by":  rec.DeployedBy,
		"created_at":   rec.CreatedAt,
	})
}

// AdminRollbackRelease POST /admin/config/releases/:id/rollback — deploy an older release
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

	// Re-deploy this release: mark current deployed as rolled_back, apply this one
	applied, skipped := 0, 0
	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		// Demote currently deployed
		if err := tx.Model(&model.ReleaseRecord{}).Where("status = ? AND id != ?", "deployed", id).
			Update("status", "rolled_back").Error; err != nil {
			return err
		}
		now := time.Now()
		if err := tx.Model(&rec).Updates(map[string]interface{}{
			"status":         "deployed",
			"rolled_back_by": adminID,
			"released_at":    now,
		}).Error; err != nil {
			return err
		}
		// Apply the snapshot configs to live DB
		if rec.Snapshot != "" {
			var snap struct {
				FeatureFlags []struct {
					Key         string `json:"key"`
					Enabled     bool   `json:"enabled"`
					RolloutPct  int    `json:"rollout_pct"`
					Whitelist   string `json:"whitelist"`
				} `json:"feature_flags"`
			}
			if err := json.Unmarshal([]byte(rec.Snapshot), &snap); err == nil {
				for _, ff := range snap.FeatureFlags {
					var existing model.FeatureFlag
					if err := tx.Where("`key` = ?", ff.Key).First(&existing).Error; err != nil {
						skipped++
						continue
					}
					if err := tx.Model(&existing).Updates(map[string]interface{}{
						"enabled":     ff.Enabled,
						"rollout_pct": ff.RolloutPct,
						"whitelist":   ff.Whitelist,
					}).Error; err != nil {
						skipped++
						continue
					}
					applied++
				}
			}
		}
		return nil
	}); err != nil {
		a.Log.Error("rollback release", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "rollback_release", "release", id,
		fmt.Sprintf(`{"version":"%s","applied":%d,"skipped":%d}`, rec.Version, applied, skipped))
	resp.OK(c, gin.H{
		"id":      id,
		"version": rec.Version,
		"status":  "deployed",
		"applied": applied,
		"skipped": skipped,
	})
}

// ──────────────────────────────────────────────
// Config Snapshot, Export, and Publish
// ──────────────────────────────────────────────

// buildConfigSnapshot collects all feature flags + system settings into a JSON snapshot.
func (a *API) buildConfigSnapshot(version string) gin.H {
	// Feature flags
	var flags []model.FeatureFlag
	a.DB.Order("id ASC").Find(&flags)
	ff := make([]gin.H, 0, len(flags))
	for _, f := range flags {
		ff = append(ff, gin.H{
			"key":         f.Key,
			"description": f.Description,
			"enabled":     f.Enabled,
			"rollout_pct": f.RolloutPct,
			"whitelist":   f.Whitelist,
		})
	}

	// System settings from runtime config
	cfg := a.Cfg
	settings := gin.H{
		"video_upload_disabled":   cfg.VideoUploadDisabled,
		"video_review_required":   cfg.VideoReviewRequired,
		"article_review_required": cfg.ArticleReviewRequired,
		"agent_enabled":           cfg.AgentEnabled,
		"agent_daily_quota":       cfg.AgentDailyQuota,
		"agent_max_history":       cfg.AgentMaxHistory,
		"agent_history_ttl":       cfg.AgentHistoryTTL.String(),
		"agent_request_timeout":   cfg.AgentRequestTimeout.String(),
	}

	return gin.H{
		"version":       version,
		"exported_at":   time.Now().UTC().Format(time.RFC3339),
		"feature_flags": ff,
		"settings":      settings,
	}
}

// AdminExportRelease GET /admin/config/releases/:id/export — download the config snapshot for a release
func (a *API) AdminExportRelease(c *gin.Context) {
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

	// If no snapshot yet, build one on-the-fly.
	snapshot := rec.Snapshot
	if snapshot == "" {
		raw, _ := json.Marshal(a.buildConfigSnapshot(rec.Version))
		snapshot = string(raw)
	}

	filename := fmt.Sprintf("config-snapshot-v%s.json", rec.Version)
	c.Header("Content-Disposition", "attachment; filename="+filename)
	c.Header("Content-Type", "application/json; charset=utf-8")
	c.String(http.StatusOK, snapshot)
}

// AdminExportConfig GET /admin/config/export — quick export current configs (no release required)
func (a *API) AdminExportConfig(c *gin.Context) {
	snapshot := a.buildConfigSnapshot("current")
	raw, err := json.Marshal(snapshot)
	if err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	filename := fmt.Sprintf("config-snapshot-%s.json", time.Now().UTC().Format("20060102-150405"))
	c.Header("Content-Disposition", "attachment; filename="+filename)
	c.Header("Content-Type", "application/json; charset=utf-8")
	c.String(http.StatusOK, string(raw))
}

// AdminDeployRelease POST /admin/config/releases/:id/deploy — apply release config to live system
func (a *API) AdminDeployRelease(c *gin.Context) {
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

	// Parse snapshot
	var snap struct {
		FeatureFlags []struct {
			Key         string `json:"key"`
			Enabled     bool   `json:"enabled"`
			RolloutPct  int    `json:"rollout_pct"`
			Whitelist   string `json:"whitelist"`
		} `json:"feature_flags"`
	}
	if err := json.Unmarshal([]byte(rec.Snapshot), &snap); err != nil {
		a.Log.Error("parse release snapshot", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Apply each feature flag from snapshot to live DB
	applied := 0
	skipped := 0
	for _, ff := range snap.FeatureFlags {
		var existing model.FeatureFlag
		err := a.DB.Where("`key` = ?", ff.Key).First(&existing).Error
		if err != nil {
			skipped++
			continue
		}
		upd := map[string]interface{}{
			"enabled":     ff.Enabled,
			"rollout_pct": ff.RolloutPct,
			"whitelist":   ff.Whitelist,
		}
		if err := a.DB.Model(&existing).Updates(upd).Error; err != nil {
			a.Log.Warn("apply flag failed", zap.Error(err), zap.String("key", ff.Key))
			skipped++
			continue
		}
		applied++
	}

	// Mark previous deployed as rolled_back, set this one as deployed
	now := time.Now()
	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Model(&model.ReleaseRecord{}).Where("status = ?", "deployed").
			Update("status", "rolled_back").Error; err != nil {
			return err
		}
		return tx.Model(&rec).Updates(map[string]interface{}{
			"status":      "deployed",
			"pushed_by":   adminID,
			"released_at": now,
		}).Error
	}); err != nil {
		a.Log.Error("deploy release", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.recordAudit(c, adminID, "deploy_release", "release", id,
		fmt.Sprintf(`{"version":"%s","applied":%d,"skipped":%d}`, rec.Version, applied, skipped))

	resp.OK(c, gin.H{
		"id":          rec.ID,
		"version":     rec.Version,
		"status":      "deployed",
		"applied":     applied,
		"skipped":     skipped,
		"released_at": now,
	})
}

// AdminGetReleaseSnapshot GET /admin/config/releases/:id/snapshot — view snapshot inline
func (a *API) AdminGetReleaseSnapshot(c *gin.Context) {
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
	if rec.Snapshot == "" {
		// Build on-the-fly
		raw, _ := json.Marshal(a.buildConfigSnapshot(rec.Version))
		rec.Snapshot = string(raw)
	}
	var snap interface{}
	if err := json.Unmarshal([]byte(rec.Snapshot), &snap); err != nil {
		// Return as string
		resp.OK(c, gin.H{"id": id, "snapshot": rec.Snapshot})
		return
	}
	resp.OK(c, gin.H{"id": id, "version": rec.Version, "snapshot": snap})
}
