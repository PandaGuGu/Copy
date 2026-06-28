package handler

import (
	"bufio"
	"fmt"
	"net/http"
	"os"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"

	"minibili/internal/data"
	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
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
			switch {
			case effectiveBaseURL != "" && effectiveBaseURL != a.Cfg.DeepSeekBaseURL:
				// Using provider-specific key
			default:
				effectiveAPIKey = a.Cfg.DeepSeekAPIKey
			}
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

	// List all providers for the frontend
	providers, _ := data.ListLLMProviders(a.DB)

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
		"providers":      providers,
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

// ── LLM Provider CRUD ──

// AdminListLLMProviders GET /api/v1/admin/llm-config/providers
func (a *API) AdminListLLMProviders(c *gin.Context) {
	providers, err := data.ListLLMProviders(a.DB)
	if err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"providers": providers})
}

// AdminCreateLLMProvider POST /api/v1/admin/llm-config/providers
func (a *API) AdminCreateLLMProvider(c *gin.Context) {
	var req struct {
		Name      string `json:"name"`
		BaseURL   string `json:"base_url"`
		Model     string `json:"model"`
		APIKey    string `json:"api_key"`
		IsDefault bool   `json:"is_default"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.BaseURL) == "" ||
		strings.TrimSpace(req.Model) == "" || strings.TrimSpace(req.APIKey) == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	prov := model.LLMProvider{
		Name:      strings.TrimSpace(req.Name),
		BaseURL:   strings.TrimRight(strings.TrimSpace(req.BaseURL), "/"),
		Model:     strings.TrimSpace(req.Model),
		APIKey:    strings.TrimSpace(req.APIKey),
		IsDefault: req.IsDefault,
		IsEnabled: true,
	}

	if prov.IsDefault {
		_ = data.SetDefaultLLMProvider(a.DB, 0) // unset all first (handled inside Create)
	}

	if err := data.CreateLLMProvider(a.DB, &prov); err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	adminID, _ := middleware.AdminID(c)
	a.recordAudit(c, adminID, "create_llm_provider", "llm_config", prov.ID,
		fmt.Sprintf("name=%s model=%s", prov.Name, prov.Model))

	resp.OK(c, gin.H{"provider": prov})
}

// AdminUpdateLLMProvider PUT /api/v1/admin/llm-config/providers/:id
func (a *API) AdminUpdateLLMProvider(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req struct {
		Name      string `json:"name"`
		BaseURL   string `json:"base_url"`
		Model     string `json:"model"`
		APIKey    string `json:"api_key"`
		IsDefault bool   `json:"is_default"`
		IsEnabled bool   `json:"is_enabled"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	prov := &model.LLMProvider{
		Name:      strings.TrimSpace(req.Name),
		BaseURL:   strings.TrimRight(strings.TrimSpace(req.BaseURL), "/"),
		Model:     strings.TrimSpace(req.Model),
		APIKey:    strings.TrimSpace(req.APIKey),
		IsDefault: req.IsDefault,
		IsEnabled: req.IsEnabled,
	}

	if prov.IsDefault {
		_ = data.SetDefaultLLMProvider(a.DB, id)
	}

	if err := data.UpdateLLMProvider(a.DB, id, prov); err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Also sync to legacy LLMConfig if this is the default
	if prov.IsDefault {
		updated, _ := data.GetLLMProvider(a.DB, id)
		if updated != nil {
			_ = data.SaveLLMConfig(a.DB, &model.LLMConfig{
				ID:      model.LLMConfigRowID,
				APIKey:  updated.APIKey,
				BaseURL: updated.BaseURL,
				Model:   updated.Model,
			})
			if a.Cfg != nil {
				a.Cfg.DeepSeekAPIKey = updated.APIKey
				a.Cfg.DeepSeekBaseURL = updated.BaseURL
				a.Cfg.DeepSeekModel = updated.Model
				if strings.TrimSpace(updated.APIKey) != "" {
					a.Cfg.AgentEnabled = true
				}
			}
		}
	}

	adminID, _ := middleware.AdminID(c)
	a.recordAudit(c, adminID, "update_llm_provider", "llm_config", id, fmt.Sprintf("name=%s model=%s", prov.Name, prov.Model))
	a.AdminListLLMProviders(c)
}

// AdminDeleteLLMProvider DELETE /api/v1/admin/llm-config/providers/:id
func (a *API) AdminDeleteLLMProvider(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if err := data.DeleteLLMProvider(a.DB, id); err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	adminID, _ := middleware.AdminID(c)
	a.recordAudit(c, adminID, "delete_llm_provider", "llm_config", id, "")
	resp.OK(c, gin.H{})
}

// AdminSetDefaultLLMProvider POST /api/v1/admin/llm-config/providers/:id/set-default
func (a *API) AdminSetDefaultLLMProvider(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if err := data.SetDefaultLLMProvider(a.DB, id); err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	// Sync to in-memory config
	prov, _ := data.GetLLMProvider(a.DB, id)
	if prov != nil && a.Cfg != nil {
		a.Cfg.DeepSeekAPIKey = prov.APIKey
		a.Cfg.DeepSeekBaseURL = prov.BaseURL
		a.Cfg.DeepSeekModel = prov.Model
		if strings.TrimSpace(prov.APIKey) != "" {
			a.Cfg.AgentEnabled = true
		}
	}
	adminID, _ := middleware.AdminID(c)
	a.recordAudit(c, adminID, "set_default_llm_provider", "llm_config", id, prov.Name)
	a.AdminListLLMProviders(c)
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
