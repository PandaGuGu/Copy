package data

import (
	"gorm.io/gorm"
	"minibili/internal/model"
)

// LoadLLMConfig returns the DB-stored LLM config from the default LLMProvider.
// Falls back to legacy LLMConfig if no providers exist.
func LoadLLMConfig(db *gorm.DB) model.LLMConfig {
	// Try LLMProvider first
	var prov model.LLMProvider
	if err := db.Where("is_default = ? AND is_enabled = ?", true, true).First(&prov).Error; err == nil {
		return model.LLMConfig{
			ID:      model.LLMConfigRowID,
			APIKey:  prov.APIKey,
			BaseURL: prov.BaseURL,
			Model:   prov.Model,
		}
	}
	// Fallback to legacy singleton
	var c model.LLMConfig
	_ = db.First(&c, model.LLMConfigRowID).Error
	return c
}

// SaveLLMConfig upserts the LLM config singleton row (legacy compat).
// Also syncs to the default LLMProvider.
func SaveLLMConfig(db *gorm.DB, cfg *model.LLMConfig) error {
	cfg.ID = model.LLMConfigRowID
	if err := db.Save(cfg).Error; err != nil {
		return err
	}
	// Sync to LLMProvider
	var prov model.LLMProvider
	err := db.Where("is_default = ?", true).First(&prov).Error
	if err != nil {
		prov = model.LLMProvider{
			Name:      "默认配置",
			BaseURL:   cfg.BaseURL,
			Model:     cfg.Model,
			APIKey:    cfg.APIKey,
			IsDefault: true,
			IsEnabled: true,
		}
		maskAPIKey(&prov)
		return db.Create(&prov).Error
	}
	if cfg.APIKey != "" && cfg.APIKey != prov.APIKey {
		prov.APIKey = cfg.APIKey
		maskAPIKey(&prov)
	}
	if cfg.BaseURL != prov.BaseURL {
		prov.BaseURL = cfg.BaseURL
	}
	if cfg.Model != prov.Model {
		prov.Model = cfg.Model
	}
	return db.Save(&prov).Error
}

// ── LLMProvider CRUD ──

func ListLLMProviders(db *gorm.DB) ([]model.LLMProvider, error) {
	var providers []model.LLMProvider
	err := db.Order("is_default DESC, name ASC").Find(&providers).Error
	return providers, err
}

func GetLLMProvider(db *gorm.DB, id uint64) (*model.LLMProvider, error) {
	var p model.LLMProvider
	err := db.First(&p, id).Error
	return &p, err
}

func CreateLLMProvider(db *gorm.DB, p *model.LLMProvider) error {
	maskAPIKey(p)
	return db.Create(p).Error
}

func UpdateLLMProvider(db *gorm.DB, id uint64, p *model.LLMProvider) error {
	var existing model.LLMProvider
	if err := db.First(&existing, id).Error; err != nil {
		return err
	}
	existing.Name = p.Name
	existing.BaseURL = p.BaseURL
	existing.Model = p.Model
	existing.IsEnabled = p.IsEnabled
	existing.IsDefault = p.IsDefault
	if p.APIKey != "" && !isMaskedKey(p.APIKey) {
		existing.APIKey = p.APIKey
		maskAPIKey(&existing)
	}
	return db.Save(&existing).Error
}

func DeleteLLMProvider(db *gorm.DB, id uint64) error {
	return db.Delete(&model.LLMProvider{}, id).Error
}

func SetDefaultLLMProvider(db *gorm.DB, id uint64) error {
	// Unset current default
	db.Model(&model.LLMProvider{}).Where("is_default = ?", true).Update("is_default", false)
	// Set new default
	return db.Model(&model.LLMProvider{}).Where("id = ?", id).Update("is_default", true).Error
}

// ── helpers ──

func maskAPIKey(p *model.LLMProvider) {
	k := p.APIKey
	if len(k) <= 8 {
		p.APIKeyMask = "***"
	} else {
		p.APIKeyMask = k[:4] + "****" + k[len(k)-4:]
	}
}

func isMaskedKey(k string) bool {
	return len(k) > 0 && (k == "***" || (len(k) >= 8 && k[4:8] == "****"))
}
