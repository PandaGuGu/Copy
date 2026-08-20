package service

import (
	"math"
	"time"

	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/model"
)

// ItemCF offline computation (SPEC F17 phase ①, NFR-REC-2).
//
// Builds a user→(video, weight) interaction matrix from 7 behavior tables,
// computes cosine similarity between every video pair sharing ≥1 user,
// and persists pairs with score >= SimThreshold into `video_similarities`.
// Designed to run nightly (see worker/scheduler.go scheduleItemCF).

const (
	// SimThreshold is the minimum cosine similarity to persist (SPEC F17).
	SimThreshold = 0.15

	// Behavior weights per SPEC F17 phase ①.
	wLike    = 1.0 // video_likes
	wCoin    = 3.0 // video_coins
	wFav     = 2.0 // video_favorites
	wWatch   = 0.5 // video_view_histories
	wComment = 1.5 // comments (video_id)
	wDanmaku = 1.0 // danmakus
)

// userItem is one (video, weight) interaction of a user.
type userItem struct {
	videoID uint64
	weight  float64
}

// ComputeItemCF recomputes the video_similarities table.
// It clears previous rows first, then batch-inserts new pairs.
func ComputeItemCF(db *gorm.DB, log *zap.Logger) (int64, error) {
	start := time.Now()
	log.Info("itemcf: compute started")

	// 1. Build user → interactions map.
	userItems := make(map[uint64][]userItem)

	load := func(table string, weight float64) error {
		type row struct {
			UserID  uint64
			VideoID uint64
		}
		var rows []row
		if err := db.Table(table).Select("user_id, video_id").Find(&rows).Error; err != nil {
			return err
		}
		for _, r := range rows {
			userItems[r.UserID] = append(userItems[r.UserID], userItem{videoID: r.VideoID, weight: weight})
		}
		return nil
	}

	// 2. Load all seven behavior sources.
	if err := load("video_likes", wLike); err != nil {
		log.Error("itemcf: load video_likes", zap.Error(err))
		return 0, err
	}
	if err := load("video_coins", wCoin); err != nil {
		log.Error("itemcf: load video_coins", zap.Error(err))
		return 0, err
	}
	if err := load("video_favorites", wFav); err != nil {
		log.Error("itemcf: load video_favorites", zap.Error(err))
		return 0, err
	}
	if err := load("video_view_histories", wWatch); err != nil {
		log.Error("itemcf: load video_view_histories", zap.Error(err))
		return 0, err
	}
	if err := load("comments", wComment); err != nil {
		log.Error("itemcf: load comments", zap.Error(err))
		return 0, err
	}
	if err := load("danmakus", wDanmaku); err != nil {
		log.Error("itemcf: load danmakus", zap.Error(err))
		return 0, err
	}

	// 3. Per-user norm² and pairwise co-occurrence accumulation.
	norm2 := make(map[uint64]float64)   // videoID → Σ w²
	pairs := make(map[[2]uint64]float64) // {a,b} (a<b) → Σ wA·wB

	for _, items := range userItems {
		// Skip users with a single interaction (no pair possible).
		if len(items) < 2 {
			continue
		}
		// Deduplicate within one user (same video from multiple tables).
		seen := make(map[uint64]float64)
		for _, it := range items {
			seen[it.videoID] += it.weight
		}
		ids := make([]uint64, 0, len(seen))
		for id, w := range seen {
			norm2[id] += w * w
			ids = append(ids, id)
		}
		for i := 0; i < len(ids); i++ {
			for j := i + 1; j < len(ids); j++ {
				a, b := ids[i], ids[j]
				if a > b {
					a, b = b, a
				}
				pairs[[2]uint64{a, b}] += seen[ids[i]] * seen[ids[j]]
			}
		}
	}

	// 4. Cosine similarity = Σ(wA·wB) / (‖A‖·‖B‖), keep pairs >= threshold.
	type simRow struct {
		videoID   uint64
		similarID uint64
		score     float64
	}
	rows := make([]simRow, 0, len(pairs))
	for pair, dot := range pairs {
		na, okA := norm2[pair[0]]
		nb, okB := norm2[pair[1]]
		if !okA || !okB || na <= 0 || nb <= 0 {
			continue
		}
		score := dot / (math.Sqrt(na) * math.Sqrt(nb))
		if score >= SimThreshold && score > 0 && score <= 1 {
			rows = append(rows, simRow{videoID: pair[0], similarID: pair[1], score: score})
		}
	}

	// 5. Persist: clear old table, batch insert new rows.
	if err := db.Exec("DELETE FROM video_similarities").Error; err != nil {
		log.Error("itemcf: clear video_similarities", zap.Error(err))
		return 0, err
	}
	if len(rows) > 0 {
		now := time.Now()
		models := make([]model.VideoSimilarity, 0, len(rows))
		for _, r := range rows {
			models = append(models, model.VideoSimilarity{
				VideoID:   r.videoID,
				SimilarID: r.similarID,
				Score:     r.score,
				UpdatedAt: now,
			})
		}
		// Batch insert in chunks of 2000 to stay under MySQL packet limits.
		const chunk = 2000
		for i := 0; i < len(models); i += chunk {
			end := i + chunk
			if end > len(models) {
				end = len(models)
			}
			if err := db.Create(models[i:end]).Error; err != nil {
				log.Error("itemcf: insert video_similarities", zap.Error(err))
				return 0, err
			}
		}
	}

	log.Info("itemcf: compute finished",
		zap.Int("users", len(userItems)),
		zap.Int("pairs_persisted", len(rows)),
		zap.Duration("elapsed", time.Since(start)),
	)
	return int64(len(rows)), nil
}
