package model

import "time"

// Appeal is a user-side governance appeal (申诉) against a governance action:
// an account ban, a content takedown, or a warning.
//
// TargetType: "user" / "video" / "article" / "dynamic" / "comment"
// ReasonType: "ban" / "takedown" / "warn"
// Status: "pending" (default) / "approved" / "rejected"
type Appeal struct {
	ID             uint64     `gorm:"primaryKey" json:"id"`
	UserID         uint64     `gorm:"index;not null" json:"user_id"`
	TargetType     string     `gorm:"size:16;index;not null" json:"target_type"`
	TargetID       uint64     `gorm:"index;not null" json:"target_id"`
	ReasonType     string     `gorm:"size:32;index;not null" json:"reason_type"`
	SourceReportID uint64     `gorm:"default:0" json:"source_report_id"`
	Content        string     `gorm:"size:2000;not null" json:"content"`
	EvidenceURLs   string     `gorm:"size:2000" json:"evidence_urls"`
	Status         string     `gorm:"size:16;index;not null;default:pending" json:"status"`
	AdminNote      string     `gorm:"size:500" json:"admin_note"`
	HandledBy      uint64     `gorm:"default:0" json:"handled_by"`
	Capability     string     `gorm:"size:32" json:"capability"` // used when TargetType == "capability"
	CreatedAt      time.Time  `gorm:"index" json:"created_at"`
	HandledAt      *time.Time `json:"handled_at"`
}

// AppealTargetTypes lists accepted appeal target types.
var AppealTargetTypes = []string{"user", "video", "article", "dynamic", "comment", "capability"}

// AppealReasonTypes lists accepted governance actions an appeal can contest.
var AppealReasonTypes = []string{"ban", "takedown", "warn"}

// AppealStatusLabel returns a human-readable label for an appeal status.
func AppealStatusLabel(s string) string {
	switch s {
	case "pending":
		return "待处理"
	case "approved":
		return "已通过"
	case "rejected":
		return "已驳回"
	}
	return s
}
