package data

import (
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/model"
)

// defaultCapReasonTemplates are seeded once when the template table is empty.
var defaultCapReasonTemplates = []string{
	"刷屏/恶意刷弹幕",
	"引战/辱骂他人",
	"恶意营销/广告",
	"涉黄涉暴内容",
	"冒充他人/侵权",
}

// SeedCapReasonTemplates seeds default capability reason templates if empty.
func SeedCapReasonTemplates(db *gorm.DB, lg *zap.Logger) {
	var cnt int64
	if err := db.Model(&model.CapabilityReasonTemplate{}).Count(&cnt).Error; err != nil || cnt > 0 {
		return
	}
	for _, content := range defaultCapReasonTemplates {
		if err := db.Create(&model.CapabilityReasonTemplate{Content: content}).Error; err != nil {
			lg.Warn("seed capability reason template", zap.Error(err))
		}
	}
	lg.Info("seeded default capability reason templates", zap.Int("count", len(defaultCapReasonTemplates)))
}