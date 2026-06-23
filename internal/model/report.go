package model

import "time"

// ReportReasonTypes lists the preset report reason categories.
var ReportReasonTypes = []struct {
	Type  string
	Label string
}{
	{"nsfw", "色情低俗"},
	{"violence", "暴力血腥"},
	{"spam", "垃圾广告"},
	{"harassment", "引战谩骂"},
	{"illegal", "违法信息"},
	{"copyright", "侵权投诉"},
	{"other", "其他"},
}

// ReportReasonLabel returns the Chinese label for a reason type, or the raw string.
func ReportReasonLabel(t string) string {
	for _, r := range ReportReasonTypes {
		if r.Type == t {
			return r.Label
		}
	}
	return t
}

// Report is a user-submitted content report.
// TargetType: "video" / "article" / "dynamic" / "comment" / "user"
// Status: "pending" (default) / "resolved" / "dismissed"
type Report struct {
	ID           uint64    `gorm:"primaryKey"`
	ReporterID   uint64    `gorm:"index;not null"`
	TargetType   string    `gorm:"size:16;index;not null"`
	TargetID     uint64    `gorm:"index;not null"`
	ReasonType   string    `gorm:"size:32;index;not null;default:other"`
	ReasonDetail string    `gorm:"size:1000"`
	Status       string    `gorm:"size:16;index;not null;default:pending"`
	HandlerNote  string    `gorm:"size:500"`
	HandledBy    uint64    `gorm:"default:0"`
	CreatedAt    time.Time `gorm:"index"`
	HandledAt    *time.Time
}
