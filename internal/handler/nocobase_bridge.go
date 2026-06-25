package handler

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strconv"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ──────────────────────────────────────────────
// NocoBase Bridge — Go ↔ NocoBase REST 互通
// ──────────────────────────────────────────────

// NocoBaseConfig holds connection settings for the NocoBase instance.
// Read from .env: NOCOBASE_URL (default http://localhost:13000)
//                NOCOBASE_API_KEY (optional, for internal auth)
type NocoBaseConfig struct {
	BaseURL string
	APIKey  string
}

// ──────────────────────────────────────────────
// Go → NocoBase: Create external resources
// ──────────────────────────────────────────────

// createNocoBaseTicket creates a ticket in NocoBase's tickets collection.
func (a *API) createNocoBaseTicket(reporterID uint64, subject, description, category string) (uint64, error) {
	cfg := a.getNocoBaseConfig()
	if cfg.BaseURL == "" {
		return 0, fmt.Errorf("nocoBase not configured")
	}
	payload := map[string]interface{}{
		"reporter_id": reporterID,
		"subject":     subject,
		"description": description,
		"category":    category,
		"status":      "open",
		"priority":    "normal",
		"source":      "minibili",
	}
	return a.nocoBaseCreate(cfg, "tickets", payload)
}

// createNocoBaseCopyrightComplaint creates a copyright complaint in NocoBase.
func (a *API) createNocoBaseCopyrightComplaint(complainantID uint64, relatedID uint64, relatedType, description string, evidenceURLs []string) (uint64, error) {
	cfg := a.getNocoBaseConfig()
	if cfg.BaseURL == "" {
		return 0, fmt.Errorf("nocoBase not configured")
	}
	payload := map[string]interface{}{
		"complainant_id": complainantID,
		"related_id":     relatedID,
		"related_type":   relatedType,
		"description":    description,
		"evidence_urls":  evidenceURLs,
		"status":         "pending",
		"source":         "minibili",
	}
	return a.nocoBaseCreate(cfg, "copyright_complaints", payload)
}

// createNocoBaseCSConversation creates a customer service conversation in NocoBase.
func (a *API) createNocoBaseCSConversation(userID uint64, subject string) (uint64, error) {
	cfg := a.getNocoBaseConfig()
	if cfg.BaseURL == "" {
		return 0, fmt.Errorf("nocoBase not configured")
	}
	payload := map[string]interface{}{
		"user_id": userID,
		"subject": subject,
		"status":  "waiting",
		"source":  "minibili",
	}
	return a.nocoBaseCreate(cfg, "cs_conversations", payload)
}

// nocoBaseCreate sends a POST to NocoBase's collection API.
func (a *API) nocoBaseCreate(cfg NocoBaseConfig, collection string, data map[string]interface{}) (uint64, error) {
	body, _ := json.Marshal(data)
	url := cfg.BaseURL + "/api/" + collection + ":create"
	req, err := http.NewRequest("POST", url, bytes.NewReader(body))
	if err != nil {
		return 0, err
	}
	req.Header.Set("Content-Type", "application/json")
	if cfg.APIKey != "" {
		req.Header.Set("Authorization", "Bearer "+cfg.APIKey)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return 0, fmt.Errorf("nocobase request failed: %w", err)
	}
	defer resp.Body.Close()
	respBody, _ := io.ReadAll(resp.Body)
	var result struct {
		Data struct {
			ID uint64 `json:"id"`
		} `json:"data"`
	}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return 0, fmt.Errorf("nocobase response parse failed: %w (body: %s)", err, string(respBody))
	}
	return result.Data.ID, nil
}

// getNocoBaseConfig reads NocoBase connection config from environment.
func (a *API) getNocoBaseConfig() NocoBaseConfig {
	return NocoBaseConfig{
		BaseURL: os.Getenv("NOCOBASE_URL"),
		APIKey:  os.Getenv("NOCOBASE_API_KEY"),
	}
}

// ──────────────────────────────────────────────
// NocoBase → Go: Action endpoints
// ──────────────────────────────────────────────

// NocoBaseTakedownVideo handles a takedown request from NocoBase workflow.
// POST /api/v1/internal/takedown/video/:id
func (a *API) NocoBaseTakedownVideo(c *gin.Context) {
	a.handleNocoBaseAction(c, "video", "takedown")
}

// NocoBaseRestoreVideo handles a restore request from NocoBase workflow.
// POST /api/v1/internal/restore/video/:id
func (a *API) NocoBaseRestoreVideo(c *gin.Context) {
	a.handleNocoBaseAction(c, "video", "published")
}

// NocoBaseBanUser handles a ban request from NocoBase workflow.
// POST /api/v1/internal/ban/user/:id
func (a *API) NocoBaseBanUser(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if err := a.DB.Model(&model.User{}).Where("id = ?", id).Update("status", "banned").Error; err != nil {
		a.Log.Error("nocoBase ban user failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.Log.Info("nocoBase ban user", zap.Uint64("user_id", id))
	resp.OK(c, gin.H{"user_id": id, "status": "banned"})
}

// NocoBaseUnbanUser handles an unban request from NocoBase workflow.
// POST /api/v1/internal/unban/user/:id
func (a *API) NocoBaseUnbanUser(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if err := a.DB.Model(&model.User{}).Where("id = ?", id).Update("status", "active").Error; err != nil {
		a.Log.Error("nocoBase unban user failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.Log.Info("nocoBase unban user", zap.Uint64("user_id", id))
	resp.OK(c, gin.H{"user_id": id, "status": "active"})
}

// NocoBaseGetUserInfo returns user info for NocoBase to display in ticket/cs forms.
// GET /api/v1/internal/user/:id
func (a *API) NocoBaseGetUserInfo(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var u model.User
	if err := a.DB.Select("id, username, nickname, avatar_url, status, created_at").First(&u, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	resp.OK(c, gin.H{
		"id": u.ID, "username": u.Username, "nickname": u.Nickname,
		"avatar_url": u.AvatarURL, "status": u.Status, "created_at": u.CreatedAt,
	})
}

// NocoBaseGetVideoInfo returns video info for NocoBase to display in copyright forms.
// GET /api/v1/internal/video/:id
func (a *API) NocoBaseGetVideoInfo(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var v model.Video
	if err := a.DB.Select("id, user_id, title, cover_url, status, play_count, created_at").First(&v, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	resp.OK(c, gin.H{
		"id": v.ID, "user_id": v.UserID, "title": v.Title,
		"cover_url": v.CoverURL, "status": v.Status,
		"play_count": v.PlayCount, "created_at": v.CreatedAt,
	})
}

// NocoBaseWebhook is a generic webhook receiver for NocoBase workflow triggers.
// POST /api/v1/internal/nocobase-webhook
func (a *API) NocoBaseWebhook(c *gin.Context) {
	var body map[string]interface{}
	if err := c.ShouldBindJSON(&body); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	action, _ := body["action"].(string)
	resourceType, _ := body["resource_type"].(string)
	resourceID, _ := body["resource_id"].(float64)

	a.Log.Info("nocoBase webhook received",
		zap.String("action", action),
		zap.String("resource_type", resourceType),
		zap.Float64("resource_id", resourceID),
	)

	// Dispatch based on action
	switch action {
	case "ticket_resolved":
		// NocoBase ticket resolved → sync status to Mini-Bili ticket if exists
		if resourceID > 0 {
			a.DB.Model(&model.Ticket{}).Where("id = ?", uint64(resourceID)).
				Updates(map[string]interface{}{"status": "resolved"})
		}
	case "copyright_accepted":
		// Copyright complaint accepted → takedown content
		if resourceID > 0 {
			var cp model.CopyrightComplaint
			if err := a.DB.First(&cp, uint64(resourceID)).Error; err == nil {
				_ = a.takedownRelatedContent(cp.RelatedID, cp.RelatedType)
			}
		}
	case "risk_ban":
		// Risk rule triggered ban → ban user
		if resourceID > 0 {
			a.DB.Model(&model.User{}).Where("id = ?", uint64(resourceID)).Update("status", "banned")
		}
	}

	resp.OK(c, gin.H{"received": true})
}

// handleNocoBaseAction is a helper for takedown/restore actions.
func (a *API) handleNocoBaseAction(c *gin.Context, resourceType, targetStatus string) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Verify internal API key
	apiKey := c.GetHeader("X-Internal-API-Key")
	if apiKey == "" || apiKey != os.Getenv("NOCOBASE_API_KEY") {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	adminID, _ := middleware.AdminID(c)

	switch resourceType {
	case "video":
		if err := a.DB.Model(&model.Video{}).Where("id = ?", id).Update("status", targetStatus).Error; err != nil {
			a.Log.Error("nocoBase action failed", zap.String("action", targetStatus), zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	case "article":
		if err := a.DB.Model(&model.Article{}).Where("id = ?", id).Update("status", targetStatus).Error; err != nil {
			a.Log.Error("nocoBase action failed", zap.String("action", targetStatus), zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	}

	a.Log.Info("nocoBase action executed",
		zap.String("action", targetStatus),
		zap.String("resource_type", resourceType),
		zap.Uint64("resource_id", id),
		zap.Uint64("operator_id", adminID),
	)

	resp.OK(c, gin.H{"resource_type": resourceType, "resource_id": id, "status": targetStatus})
}
