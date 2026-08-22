package model

import "time"

// UserCapabilityRestriction records a capability-level restriction on a user.
// A user whose account is active can still lose individual capabilities
// (发布/评论/弹幕/私信/直播/投币) without a full ban. Each active row forbids
// that one capability for the user.
//
// Status: "active" (restricting) / "restored" (lifted, kept for audit).
type UserCapabilityRestriction struct {
	ID         uint64     `gorm:"primaryKey" json:"id"`
	UserID     uint64     `gorm:"index;not null" json:"user_id"`
	Capability string     `gorm:"size:32;index;not null" json:"capability"`
	Reason     string     `gorm:"size:300" json:"reason"`
	ExpiresAt  *time.Time `gorm:"index" json:"expires_at"`
	Status     string     `gorm:"size:16;not null;default:active;index" json:"status"`
	CreatedBy  uint64     `gorm:"not null;default:0" json:"created_by"`
	CreatedAt  time.Time  `gorm:"index" json:"created_at"`
	RestoredAt *time.Time `json:"restored_at"`
}

func (UserCapabilityRestriction) TableName() string { return "user_capability_restrictions" }

// CapabilityReasonTemplate is a reusable preset reason for capability
// restrictions, manageable by ops (运营可增删改).
type CapabilityReasonTemplate struct {
	ID        uint64    `gorm:"primaryKey" json:"id"`
	Content   string    `gorm:"size:200;not null" json:"content"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

func (CapabilityReasonTemplate) TableName() string { return "usercap_reason_templates" }
