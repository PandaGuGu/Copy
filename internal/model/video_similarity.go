package model

import "time"

// VideoSimilarity stores precomputed ItemCF pairwise similarity between videos.
//
// Filled by the offline job (service/itemcf.go) every night, consumed by the
// online feed recall (service/feed_service.go). Only pairs with
// similarity >= threshold (0.15) are persisted to keep the table bounded.
type VideoSimilarity struct {
	ID        uint64  `gorm:"primaryKey"`
	VideoID   uint64  `gorm:"index:idx_sim_video_pair,priority:1;not null"`
	SimilarID uint64  `gorm:"index:idx_sim_video_pair,priority:2;not null"`
	// Score is cosine similarity in [0,1]; higher = more alike.
	Score float64 `gorm:"type:double precision;not null"`
	// UpdatedAt is the last recompute time (job stamps it).
	UpdatedAt time.Time `gorm:"index"`
}

// TableName fixes the table name (avoids GORM pluralization surprises).
func (VideoSimilarity) TableName() string { return "video_similarities" }
