package handler

import (
	"bufio"
	"fmt"
	"net/http"
	"os"
	"strings"

	"github.com/gin-gonic/gin"

	"minibili/internal/data"
	"minibili/internal/errcode"
	"minibili/internal/pkg/resp"
)

// AdminGetLLMConfig GET /api/v1/admin/llm-config
func (a *API) AdminGetLLMConfig(c *gin.Context) {
	cfg := data.LoadLLMConfig(a.DB)

	// Determine effective values: DB overrides .env
	effectiveAPIKey := strings.TrimSpace(cfg.APIKey)
	effectiveBaseURL := strings.TrimSpace(cfg.BaseURL)
	effectiveModel := strings.TrimSpace(cfg.Model)

	if a.Cfg != nil {
		if effectiveAPIKey == "" {
			effectiveAPIKey = a.Cfg.DeepSeekAPIKey
		}
		if effectiveBaseURL == "" {
			effectiveBaseURL = a.Cfg.DeepSeekBaseURL
		}
		if effectiveModel == "" {
			effectiveModel = a.Cfg.DeepSeekModel
		}
	}

	// Mask API key for display
	apiKeyDisplay := ""
	if strings.TrimSpace(effectiveAPIKey) != "" {
		if len(effectiveAPIKey) <= 8 {
			apiKeyDisplay = "***"
		} else {
			apiKeyDisplay = effectiveAPIKey[:4] + "****" + effectiveAPIKey[len(effectiveAPIKey)-4:]
		}
	}

	configured := strings.TrimSpace(effectiveAPIKey) != ""

	// Show env values as reference
	envAPIKey := ""
	envMasked := ""
	if a.Cfg != nil {
		envAPIKey = a.Cfg.DeepSeekAPIKey
		if strings.TrimSpace(envAPIKey) != "" {
			if len(envAPIKey) <= 8 {
				envMasked = "***"
			} else {
				envMasked = envAPIKey[:4] + "****" + envAPIKey[len(envAPIKey)-4:]
			}
		}
	}

	resp.OK(c, gin.H{
		"base_url":       effectiveBaseURL,
		"model":          effectiveModel,
		"api_key":        apiKeyDisplay,
		"configured":     configured,
		"from_env":       a.Cfg != nil && strings.TrimSpace(cfg.APIKey) == "" && strings.TrimSpace(a.Cfg.DeepSeekAPIKey) != "",
		"env_base_url":   a.envStr("DEEPSEEK_BASE_URL"),
		"env_model":      a.envStr("DEEPSEEK_MODEL"),
		"env_api_key":    envMasked,
		"db_base_url":    strings.TrimSpace(cfg.BaseURL),
		"db_model":       strings.TrimSpace(cfg.Model),
		"db_api_key_set": strings.TrimSpace(cfg.APIKey) != "",
	})
}

// AdminPutLLMConfig PUT /api/v1/admin/llm-config
func (a *API) AdminPutLLMConfig(c *gin.Context) {
	var req struct {
		BaseURL string `json:"base_url"`
		Model   string `json:"model"`
		APIKey  string `json:"api_key"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	cfg := data.LoadLLMConfig(a.DB)

	baseURLChanged := false
	modelChanged := false
	apiKeyChanged := false

	if strings.TrimSpace(req.BaseURL) != "" {
		newURL := strings.TrimRight(strings.TrimSpace(req.BaseURL), "/")
		if newURL != strings.TrimSpace(cfg.BaseURL) {
			cfg.BaseURL = newURL
			baseURLChanged = true
		}
	}
	if strings.TrimSpace(req.Model) != "" {
		newModel := strings.TrimSpace(req.Model)
		if newModel != strings.TrimSpace(cfg.Model) {
			cfg.Model = newModel
			modelChanged = true
		}
	}
	if req.APIKey != "" && !strings.Contains(req.APIKey, "****") {
		// Only update if a real key is provided (not the masked version)
		newKey := strings.TrimSpace(req.APIKey)
		if newKey != strings.TrimSpace(cfg.APIKey) {
			cfg.APIKey = newKey
			apiKeyChanged = true
		}
	}

	if !baseURLChanged && !modelChanged && !apiKeyChanged {
		// No changes, just return current state
		a.AdminGetLLMConfig(c)
		return
	}

	if err := data.SaveLLMConfig(a.DB, &cfg); err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Write changes back to .env file for persistence across restarts
	if baseURLChanged || modelChanged || apiKeyChanged {
		_ = writeEnvFile(baseURLChanged, modelChanged, apiKeyChanged,
			cfg.BaseURL, cfg.Model, cfg.APIKey)
	}

	// Update in-memory config for immediate effect
	if a.Cfg != nil {
		if baseURLChanged {
			a.Cfg.DeepSeekBaseURL = cfg.BaseURL
		}
		if modelChanged {
			a.Cfg.DeepSeekModel = cfg.Model
		}
		if apiKeyChanged {
			a.Cfg.DeepSeekAPIKey = cfg.APIKey
			// If API key is now set, auto-enable agent
			if strings.TrimSpace(cfg.APIKey) != "" && !a.Cfg.AgentEnabled {
				a.Cfg.AgentEnabled = true
			}
		}
	}

	a.AdminGetLLMConfig(c)
}

func (a *API) envStr(key string) string {
	if a.Cfg == nil {
		return ""
	}
	return strings.TrimSpace(os.Getenv(key))
}

// writeEnvFile updates DEEPSEEK_* keys in the local .env file.
// This is a best-effort operation; failures are logged but not propagated.
func writeEnvFile(
	updateBaseURL, updateModel, updateAPIKey bool,
	baseURL, model, apiKey string,
) error {
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

	// Update matching keys
	baseDone, modelDone, keyDone := !updateBaseURL, !updateModel, !updateAPIKey
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
		switch key {
		case "DEEPSEEK_BASE_URL":
			if updateBaseURL && !baseDone {
				lines[i] = fmt.Sprintf("DEEPSEEK_BASE_URL=%s", baseURL)
				baseDone = true
			}
		case "DEEPSEEK_MODEL":
			if updateModel && !modelDone {
				lines[i] = fmt.Sprintf("DEEPSEEK_MODEL=%s", model)
				modelDone = true
			}
		case "DEEPSEEK_API_KEY":
			if updateAPIKey && !keyDone {
				lines[i] = fmt.Sprintf("DEEPSEEK_API_KEY=%s", apiKey)
				keyDone = true
			}
		}
	}

	// Append any keys that didn't exist in the file
	if updateBaseURL && !baseDone {
		lines = append(lines, fmt.Sprintf("DEEPSEEK_BASE_URL=%s", baseURL))
	}
	if updateModel && !modelDone {
		lines = append(lines, fmt.Sprintf("DEEPSEEK_MODEL=%s", model))
	}
	if updateAPIKey && !keyDone {
		lines = append(lines, fmt.Sprintf("DEEPSEEK_API_KEY=%s", apiKey))
	}

	// Write back
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
