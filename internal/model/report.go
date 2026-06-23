package model

import "time"

// Report is a user-submitted content report.
// TargetType: "video" / "article" / "dynamic" / "comment" / "user"
// Status: "pending" (default) / "resolved" / "dismissed"
type Report struct {
	ID           uint64    `gorm:"primaryKey"`
	ReporterID   uint64    `gorm:"index;not null"`
	TargetType   string    `gorm:"size:16;index;not null"`
	TargetID     uint64    `gorm:"index;not null"`
	Reason       string    `gorm:"size:1000;not null"`
	Status       string    `gorm:"size:16;index;not null;default:pending"`
	HandlerNote  string    `gorm:"size:500"`
	HandledBy    uint64    `gorm:"default:0"`
	CreatedAt    time.Time `gorm:"index"`
	HandledAt    *time.Time
}
