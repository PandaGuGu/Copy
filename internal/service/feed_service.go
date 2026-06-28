// Package service — feed recommendation pipeline.
//
// Architecture:
//
//	Request → Profile(Lambda) → Candidate Pool(Redis) → MMR(k,λ) → Cache → Response
//
// Candidate pools are refreshed every 60s by a background goroutine.
// Results for anonymous users are cached by zone, logged-in users by uid.
package service

import (
	"context"
	"encoding/json"
	"log"
	"math"
	"sync"
	"time"

	"github.com/redis/go-redis/v9"
	"gorm.io/gorm"

	"minibili/internal/model"
)

// ─── Service ─────────────────────────────────────────────────

// FeedService is the recommendation pipeline service.
type FeedService struct {
	DB      *gorm.DB
	Redis   *redis.Client
	Profile *UserProfileService

	cancel context.CancelFunc
	mu     sync.RWMutex
	pool   map[string][]VideoFeatures // segment → candidates
	hot    []VideoFeatures             // global hot pool
}

// NewFeedService creates and starts the feed service.
func NewFeedService(db *gorm.DB, rdb *redis.Client) *FeedService {
	fs := &FeedService{
		DB:      db,
		Redis:   rdb,
		Profile: &UserProfileService{DB: db, Redis: rdb},
		pool:    make(map[string][]VideoFeatures),
	}
	ctx, cancel := context.WithCancel(context.Background())
	fs.cancel = cancel
	go fs.warmLoop(ctx)
	return fs
}

// Shutdown stops the background warm loop.
func (fs *FeedService) Shutdown() {
	if fs.cancel != nil {
		fs.cancel()
	}
}

// ─── Public API ───────────────────────────────────────────────

// RecommendationResult is the response from the feed pipeline.
type RecommendationResult struct {
	Items      []*model.Video `json:"items"`
	NextCursor uint64         `json:"next_cursor"`
}

// GetRecommendation returns diversity-ranked recommendations for a user.
// uid=0 means anonymous (default λ=0.7, no segment filtering).
func (fs *FeedService) GetRecommendation(ctx context.Context, uid uint64, limit int) *RecommendationResult {
	candidates := fs.getCandidates(uid)

	lambda := fs.getUserLambda(ctx, uid)
	result := fs.rerank(candidates, limit, lambda)
	return &RecommendationResult{Items: result}
}

// GetZoneRecommendation returns diversity-ranked videos within a zone.
func (fs *FeedService) GetZoneRecommendation(ctx context.Context, uid uint64, zone string, limit int) *RecommendationResult {
	candidates := fs.getZoneCandidates(uid, zone)

	lambda := fs.getUserLambda(ctx, uid)
	result := fs.rerank(candidates, limit, lambda)
	return &RecommendationResult{Items: result}
}

// ─── Candidate pools ──────────────────────────────────────────

func (fs *FeedService) getCandidates(uid uint64) []VideoFeatures {
	fs.mu.RLock()
	defer fs.mu.RUnlock()

	// Segmented users get a filtered pool.
	if uid != 0 {
		if p := fs.Profile.cacheGet(context.Background(), uid); p != nil && p.Segment != SegMix {
			if seg, ok := fs.pool[p.Segment]; ok && len(seg) > 0 {
				return seg
			}
		}
	}
	return fs.hot
}

func (fs *FeedService) getZoneCandidates(uid uint64, zone string) []VideoFeatures {
	zp := zoneParent(zone)

	fs.mu.RLock()
	hot := fs.hot
	fs.mu.RUnlock()

	// Filter global pool by zone.
	filtered := make([]VideoFeatures, 0, len(hot))
	for _, f := range hot {
		if f.ZoneParent == zp || f.Video.Zone == zone {
			filtered = append(filtered, f)
		}
	}
	return filtered
}

// ─── Re-ranking wrapper ──────────────────────────────────────

func (fs *FeedService) rerank(pool []VideoFeatures, limit int, lambda float64) []*model.Video {
	// Try Redis cache for default lambda (high hit rate).
	if math.Abs(lambda-DefaultLambda) < 0.01 {
		if cached := fs.cacheGetResult("rerank:default"); cached != nil {
			n := len(cached)
			if limit < n {
				n = limit
			}
			return cached[:n]
		}
	}

	adjustedK := limit
	if adjustedK > len(pool) {
		adjustedK = len(pool)
	}

	result := MMRVideos(pool, adjustedK, lambda)
	if len(result) > limit {
		result = result[:limit]
	}
	return result
}

// ─── User lambda ──────────────────────────────────────────────

func (fs *FeedService) getUserLambda(ctx context.Context, uid uint64) float64 {
	if p := fs.Profile.GetProfile(ctx, uid); p != nil {
		return p.Lambda
	}
	return DefaultLambda
}

// ─── Cache helpers ────────────────────────────────────────────

const (
	rerankCacheKey = "feed:rerank:default"
	rerankCacheTTL = 30 * time.Second
)

func (fs *FeedService) cacheGetResult(key string) []*model.Video {
	data, err := fs.Redis.Get(context.Background(), key).Bytes()
	if err != nil {
		return nil
	}
	var videos []*model.Video
	if json.Unmarshal(data, &videos) != nil {
		return nil
	}
	return videos
}

// ─── Background warm loop ────────────────────────────────────

const warmInterval = 60 * time.Second

func (fs *FeedService) warmLoop(ctx context.Context) {
	ticker := time.NewTicker(warmInterval)
	defer ticker.Stop()

	// Warm immediately on startup.
	fs.warmOnce(context.Background())

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			fs.warmOnce(context.Background())
		}
	}
}

func (fs *FeedService) warmOnce(ctx context.Context) {
	// Global hot pool: top 300 published videos by play count.
	hot, err := fs.fetchCandidates(ctx, "", 300)
	if err != nil {
		log.Printf("[feed] warm hot pool: %v", err)
		return
	}

	segments := make(map[string][]VideoFeatures)

	// Segment pools: filter from hot pool by zone parent.
	for _, s := range []string{SegAnime, SegGame, SegTech, SegLife} {
		pool := fs.filterSegmentCandidates(hot, s)
		if len(pool) > 0 {
			segments[s] = pool
		}
	}

	fs.mu.Lock()
	fs.hot = hot
	fs.pool = segments
	fs.mu.Unlock()

	// Pre-compute default re-rank result.
	defaultResult := MMRVideos(hot, 50, DefaultLambda)
	data, _ := json.Marshal(defaultResult)
	fs.Redis.Set(ctx, rerankCacheKey, data, rerankCacheTTL)

	log.Printf("[feed] warmed: hot=%d segments=%d", len(hot), len(segments))
}

func (fs *FeedService) fetchCandidates(ctx context.Context, zone string, limit int) ([]VideoFeatures, error) {
	query := fs.DB.WithContext(ctx).Where("status = ?", "published")
	if zone != "" {
		zp := zoneParent(zone)
		query = query.Where("zone = ? OR zone LIKE ?", zp, zp+"-%")
	}
	var videos []model.Video
	err := query.Order("play_count DESC, created_at DESC").
		Limit(limit).Find(&videos).Error
	if err != nil {
		return nil, err
	}
	features := make([]VideoFeatures, len(videos))
	for i := range videos {
		features[i] = ExtractFeatures(&videos[i])
	}
	return features, nil
}

func (fs *FeedService) filterSegmentCandidates(
	pool []VideoFeatures,
	segment string,
) []VideoFeatures {
	zones, ok := SegmentZones[segment]
	if !ok || len(zones) == 0 {
		return pool
	}
	zoneSet := make(map[string]bool, len(zones))
	for _, z := range zones {
		zoneSet[z] = true
	}

	filtered := make([]VideoFeatures, 0, len(pool))
	others := make([]VideoFeatures, 0)

	for _, f := range pool {
		if zoneSet[f.ZoneParent] {
			filtered = append(filtered, f)
		} else {
			others = append(others, f)
		}
	}

	// Keep at least 100 candidates (fill with others if needed).
	if len(filtered) < 100 {
		needed := 100 - len(filtered)
		if needed > len(others) {
			needed = len(others)
		}
		filtered = append(filtered, others[:needed]...)
	}
	return filtered
}
