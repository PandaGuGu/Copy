package model

import "time"

// LLMConfig stores the active LLM provider configuration (singleton row, ID=1).
// Legacy — kept for backward compatibility; new code uses LLMProvider.
// Values here override the .env defaults when present.
type LLMConfig struct {
	ID        uint64    `gorm:"primaryKey"`
	APIKey    string    `gorm:"size:512"`
	BaseURL   string    `gorm:"size:512"`
	Model     string    `gorm:"size:128"`
	UpdatedAt time.Time
}

const LLMConfigRowID uint64 = 1

// LLMProvider is a multi-vendor LLM configuration entry.
// Replaces the singleton LLMConfig with support for multiple providers
// (DeepSeek, Agnes, 阶跃星辰, 阿里百炼, etc.).
type LLMProvider struct {
	ID          uint64    `gorm:"primaryKey" json:"id"`
	Name        string    `gorm:"size:80;not null" json:"name"`           // display name, e.g. "DeepSeek V4"
	BaseURL     string    `gorm:"size:512;not null" json:"base_url"`       // OpenAI-compatible endpoint
	Model       string    `gorm:"size:128;not null" json:"model"`          // model identifier
	APIKey      string    `gorm:"size:512;not null" json:"-"`              // hidden in JSON responses
	APIKeyMask  string    `gorm:"size:20" json:"api_key_mask"`             // "sk-****abcd" for display
	IsDefault   bool      `gorm:"not null;default:0;index" json:"is_default"`
	IsEnabled   bool      `gorm:"not null;default:1;index" json:"is_enabled"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
}
