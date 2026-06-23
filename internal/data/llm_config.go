package data

import (
	"gorm.io/gorm"
	"minibili/internal/model"
)

// LoadLLMConfig returns the DB-stored LLM config (empty APIKey if not set).
func LoadLLMConfig(db *gorm.DB) model.LLMConfig {
	var c model.LLMConfig
	_ = db.First(&c, model.LLMConfigRowID).Error
	return c
}

// SaveLLMConfig upserts the LLM config singleton row.
func SaveLLMConfig(db *gorm.DB, cfg *model.LLMConfig) error {
	cfg.ID = model.LLMConfigRowID
	return db.Save(cfg).Error
}
