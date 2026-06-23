package model

import "time"

// LLMConfig stores the active LLM provider configuration (singleton row, ID=1).
// Values here override the .env defaults when present.
type LLMConfig struct {
	ID        uint64    `gorm:"primaryKey"`
	APIKey    string    `gorm:"size:512"`
	BaseURL   string    `gorm:"size:512"`
	Model     string    `gorm:"size:128"`
	UpdatedAt time.Time
}

const LLMConfigRowID uint64 = 1
