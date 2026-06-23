package aigateway

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

// ChatMessage is an OpenAI-compatible chat message.
type ChatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

// Client calls an OpenAI-compatible chat/completions endpoint.
type Client struct {
	APIKey     string
	BaseURL    string
	Model      string
	HTTPClient *http.Client
	// DBConfig provides runtime DB overrides (takes precedence over static fields).
	DBConfig func() (apiKey, baseURL, model string)
}

type chatCompletionReq struct {
	Model       string        `json:"model"`
	Messages    []ChatMessage `json:"messages"`
	Temperature float64       `json:"temperature,omitempty"`
	Stream      bool          `json:"stream"`
}

type chatCompletionResp struct {
	Choices []struct {
		Message ChatMessage `json:"message"`
	} `json:"choices"`
	Error *struct {
		Message string `json:"message"`
	} `json:"error,omitempty"`
}

// resolveConfig returns effective apiKey, baseURL, model preferring DB overrides.
func (c *Client) resolveConfig() (apiKey, baseURL, model string) {
	apiKey = c.APIKey
	baseURL = c.BaseURL
	model = c.Model
	if c.DBConfig != nil {
		if dbKey, dbBase, dbModel := c.DBConfig(); dbKey != "" {
			apiKey = dbKey
			baseURL = dbBase
			model = dbModel
		}
	}
	return
}

// Complete returns the assistant text for the given messages.
func (c *Client) Complete(ctx context.Context, messages []ChatMessage) (string, error) {
	apiKey, base, model := c.resolveConfig()
	if c == nil || strings.TrimSpace(apiKey) == "" {
		return "", fmt.Errorf("llm: api key not configured")
	}
	base = strings.TrimRight(base, "/")
	if base == "" {
		base = "https://api.deepseek.com"
	}
	if model == "" {
		model = "deepseek-chat"
	}
	body, err := json.Marshal(chatCompletionReq{
		Model:       model,
		Messages:    messages,
		Temperature: 0.7,
		Stream:      false,
	})
	if err != nil {
		return "", err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, base+"/v1/chat/completions", bytes.NewReader(body))
	if err != nil {
		return "", err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+apiKey)

	hc := c.HTTPClient
	if hc == nil {
		hc = &http.Client{Timeout: 90 * time.Second}
	}
	res, err := hc.Do(req)
	if err != nil {
		return "", err
	}
	defer res.Body.Close()
	raw, err := io.ReadAll(io.LimitReader(res.Body, 2<<20))
	if err != nil {
		return "", err
	}
	if res.StatusCode < 200 || res.StatusCode >= 300 {
		return "", fmt.Errorf("llm: http %d: %s", res.StatusCode, truncate(string(raw), 400))
	}
	var out chatCompletionResp
	if err := json.Unmarshal(raw, &out); err != nil {
		return "", err
	}
	if out.Error != nil && out.Error.Message != "" {
		return "", fmt.Errorf("llm: %s", out.Error.Message)
	}
	if len(out.Choices) == 0 || strings.TrimSpace(out.Choices[0].Message.Content) == "" {
		return "", fmt.Errorf("llm: empty completion")
	}
	return strings.TrimSpace(out.Choices[0].Message.Content), nil
}

func truncate(s string, n int) string {
	s = strings.TrimSpace(s)
	if len(s) <= n {
		return s
	}
	return s[:n] + "…"
}
