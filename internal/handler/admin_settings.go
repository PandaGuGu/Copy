package handler

import (
	"bufio"
	"fmt"
	"net/http"
	"os"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"

	"minibili/internal/errcode"
	"minibili/internal/pkg/resp"
)

// AdminGetSettings GET /api/v1/admin/settings
func (a *API) AdminGetSettings(c *gin.Context) {
	cfg := a.Cfg
	if cfg == nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{
		"video_upload_disabled":   cfg.VideoUploadDisabled,
		"video_review_required":   cfg.VideoReviewRequired,
		"article_review_required": cfg.ArticleReviewRequired,
		"agent_enabled":           cfg.AgentEnabled,
		"agent_daily_quota":       cfg.AgentDailyQuota,
		"agent_max_history":       cfg.AgentMaxHistory,
		"agent_history_ttl":       cfg.AgentHistoryTTL.String(),
		"agent_request_timeout":   cfg.AgentRequestTimeout.String(),
	})
}

// AdminPutSettings PUT /api/v1/admin/settings
func (a *API) AdminPutSettings(c *gin.Context) {
	var req struct {
		VideoUploadDisabled   *bool   `json:"video_upload_disabled"`
		VideoReviewRequired   *bool   `json:"video_review_required"`
		ArticleReviewRequired *bool   `json:"article_review_required"`
		AgentEnabled          *bool   `json:"agent_enabled"`
		AgentDailyQuota       *int    `json:"agent_daily_quota"`
		AgentMaxHistory       *int    `json:"agent_max_history"`
		AgentHistoryTTL       *string `json:"agent_history_ttl"`
		AgentRequestTimeout   *string `json:"agent_request_timeout"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	cfg := a.Cfg
	if cfg == nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	envUpdates := make(map[string]string)

	if req.VideoUploadDisabled != nil {
		cfg.VideoUploadDisabled = *req.VideoUploadDisabled
		envUpdates["VIDEO_UPLOAD_DISABLED"] = strconv.FormatBool(*req.VideoUploadDisabled)
	}
	if req.VideoReviewRequired != nil {
		cfg.VideoReviewRequired = *req.VideoReviewRequired
		envUpdates["VIDEO_REVIEW_REQUIRED"] = strconv.FormatBool(*req.VideoReviewRequired)
	}
	if req.ArticleReviewRequired != nil {
		cfg.ArticleReviewRequired = *req.ArticleReviewRequired
		envUpdates["ARTICLE_REVIEW_REQUIRED"] = strconv.FormatBool(*req.ArticleReviewRequired)
	}
	if req.AgentEnabled != nil {
		cfg.AgentEnabled = *req.AgentEnabled
		envUpdates["AGENT_ENABLED"] = strconv.FormatBool(*req.AgentEnabled)
	}
	if req.AgentDailyQuota != nil {
		cfg.AgentDailyQuota = *req.AgentDailyQuota
		envUpdates["AGENT_DAILY_QUOTA"] = strconv.Itoa(*req.AgentDailyQuota)
	}
	if req.AgentMaxHistory != nil {
		cfg.AgentMaxHistory = *req.AgentMaxHistory
		envUpdates["AGENT_MAX_HISTORY"] = strconv.Itoa(*req.AgentMaxHistory)
	}
	if req.AgentHistoryTTL != nil {
		s := strings.TrimSpace(*req.AgentHistoryTTL)
		if s != "" {
			envUpdates["AGENT_HISTORY_TTL"] = s
		}
	}
	if req.AgentRequestTimeout != nil {
		s := strings.TrimSpace(*req.AgentRequestTimeout)
		if s != "" {
			envUpdates["AGENT_REQUEST_TIMEOUT"] = s
		}
	}

	if len(envUpdates) == 0 {
		a.AdminGetSettings(c)
		return
	}

	_ = updateEnvKeys(envUpdates)

	a.AdminGetSettings(c)
}

// updateEnvKeys updates specific key=value pairs in the local .env file.
func updateEnvKeys(updates map[string]string) error {
	const envPath = ".env"

	f, err := os.Open(envPath)
	if err != nil {
		return fmt.Errorf("open .env: %w", err)
	}
	defer f.Close()

	var lines []string
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		lines = append(lines, scanner.Text())
	}
	if err := scanner.Err(); err != nil {
		return fmt.Errorf("read .env: %w", err)
	}
	f.Close()

	done := make(map[string]bool, len(updates))
	for i, line := range lines {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" || strings.HasPrefix(trimmed, "#") {
			continue
		}
		eq := strings.Index(trimmed, "=")
		if eq < 0 {
			continue
		}
		key := strings.TrimSpace(trimmed[:eq])
		if val, ok := updates[key]; ok {
			lines[i] = key + "=" + val
			done[key] = true
		}
	}

	for k, v := range updates {
		if !done[k] {
			lines = append(lines, k+"="+v)
		}
	}

	out, err := os.Create(envPath)
	if err != nil {
		return fmt.Errorf("create .env: %w", err)
	}
	defer out.Close()

	for _, line := range lines {
		fmt.Fprintln(out, line)
	}
	return out.Close()
}
