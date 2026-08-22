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
	hot    []VideoFeatures            // global hot pool
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

	// ItemCF recall: for logged-in users, prepend videos similar to
	// the ones they interacted with (from offline video_similarities).
	if uid != 0 {
		if itemcf := fs.itemCFRecall(ctx, uid, limit); len(itemcf) > 0 {
			candidates = mergeCandidates(itemcf, candidates)
		}
	}

	lambda := fs.getUserLambda(ctx, uid)
	result := fs.rerank(candidates, limit, lambda)
	return &RecommendationResult{Items: result}
}

// itemCFRecall fetches videos similar to the user's recent interactions
// from the offline-computed video_similarities table (SPEC F17, NFR-REC-2).
// Falls back to empty when the table is not yet populated (cold start).
func (fs *FeedService) itemCFRecall(ctx context.Context, uid uint64, limit int) []VideoFeatures {
	// 1. Gather the user's most recent interacted video IDs (across 3 tables).
	type idRow struct{ VideoID uint64 }
	var interacted []uint64
	seen := make(map[uint64]bool)
	collect := func(table string) {
		var rows []idRow
		if err := fs.DB.WithContext(ctx).Table(table).
			Select("video_id").
			Where("user_id = ?", uid).
			Order("id DESC").
			Limit(50).Find(&rows).Error; err != nil {
			return
		}
		for _, r := range rows {
			if !seen[r.VideoID] {
				seen[r.VideoID] = true
				interacted = append(interacted, r.VideoID)
			}
		}
	}
	collect("video_likes")
	collect("video_coins")
	collect("video_favorites")

	if len(interacted) == 0 {
		return nil
	}

	// 2. Look up similar videos from the offline table.
	var sims []model.VideoSimilarity
	if err := fs.DB.WithContext(ctx).
		Where("video_id IN ?", interacted).
		Order("score DESC").
		Limit(limit * 2).Find(&sims).Error; err != nil || len(sims) == 0 {
		return nil
	}

	// 3. Resolve candidate videos (published only).
	similarIDs := make([]uint64, 0, len(sims))
	simSeen := make(map[uint64]bool)
	for _, s := range sims {
		if !simSeen[s.SimilarID] && !seen[s.SimilarID] {
			simSeen[s.SimilarID] = true
			similarIDs = append(similarIDs, s.SimilarID)
		}
	}
	if len(similarIDs) == 0 {
		return nil
	}

	var videos []model.Video
	if err := fs.DB.WithContext(ctx).
		Where("id IN ? AND status = ?", similarIDs, "published").
		Find(&videos).Error; err != nil || len(videos) == 0 {
		return nil
	}

	// Rank by similarity score (sims are already score DESC, preserve order).
	scoreRank := make(map[uint64]int, len(sims))
	for i, s := range sims {
		if _, ok := scoreRank[s.SimilarID]; !ok {
			scoreRank[s.SimilarID] = i
		}
	}
	byID := make(map[uint64]*model.Video, len(videos))
	for i := range videos {
		byID[videos[i].ID] = &videos[i]
	}
	ordered := make([]*model.Video, 0, len(videos))
	for _, sid := range similarIDs {
		if v, ok := byID[sid]; ok {
			ordered = append(ordered, v)
		}
	}
	// Secondary sort by similarity rank for stability.
	for i := 1; i < len(ordered); i++ {
		for j := i; j > 0 && scoreRank[ordered[j].ID] < scoreRank[ordered[j-1].ID]; j-- {
			ordered[j], ordered[j-1] = ordered[j-1], ordered[j]
		}
	}

	features := make([]VideoFeatures, 0, len(ordered))
	for _, v := range ordered {
		features = append(features, ExtractFeatures(v))
	}
	return features
}

// mergeCandidates prepends itemCF candidates to the base pool,
// deduplicating by video ID (base pool wins on duplicates).
func mergeCandidates(itemCF, base []VideoFeatures) []VideoFeatures {
	seen := make(map[uint64]bool, len(base))
	for _, f := range base {
		if f.Video != nil {
			seen[f.Video.ID] = true
		}
	}
	merged := make([]VideoFeatures, 0, len(itemCF)+len(base))
	for _, f := range itemCF {
		if f.Video != nil && !seen[f.Video.ID] {
			seen[f.Video.ID] = true
			merged = append(merged, f)
		}
	}
	return append(merged, base...)
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

	// Two-stage ranking: coarse stage trims to a substantial top-N subset
	// (promoting fresh cold-start videos), then MMR fine rank adds diversity.
	coarseN := limit * 3
	if coarseN > len(pool) {
		coarseN = len(pool)
	}
	if coarseN > 0 && coarseN < len(pool) {
		pool = CoarseTrim(pool, coarseN)
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

	// Cold-start: merge recently-published (low/zero engagement) videos into
	// the pool so brand-new content can surface (NFR-REC cold start).
	if fresh, ferr := fs.fetchFreshCandidates(ctx, 40); ferr == nil && len(fresh) > 0 {
		hot = mergeCandidates(fresh, hot)
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

// fetchFreshCandidates returns the newest published videos within the
// cold-start window, for the cold-start insertion into the warm pool.
func (fs *FeedService) fetchFreshCandidates(ctx context.Context, limit int) ([]VideoFeatures, error) {
	var videos []model.Video
	err := fs.DB.WithContext(ctx).
		Where("status = ? AND created_at >= ?", "published", time.Now().Add(-ColdStartWindow)).
		Order("created_at DESC").
		Limit(limit).Find(&videos).Error
	if err != nil {
		return nil, err
	}
	out := make([]VideoFeatures, len(videos))
	for i := range videos {
		out[i] = ExtractFeatures(&videos[i])
	}
	return out, nil
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
