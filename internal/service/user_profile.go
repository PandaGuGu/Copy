// Package service — user profile and segmentation for recommendation re-ranking.
//
// Builds lightweight profiles from view/like/fav/coin/search history,
// computes zone/tag affinity, adaptive lambda, and user segment.
// All profiles are cached in Redis (TTL 7d). New users fall back to defaults.
package service

import (
	"context"
	"encoding/json"
	"fmt"
	"math"
	"time"

	"github.com/redis/go-redis/v9"
	"gorm.io/gorm"

	"minibili/internal/model"
)

// ─── Profile model ────────────────────────────────────────────

// UserProfile captures content preferences for one user.
// Cached in Redis as JSON under key "user:profile:{uid}".
type UserProfile struct {
	UserID   uint64             `json:"uid"`
	Version  int64              `json:"version"` // last build timestamp
	Segment  string             `json:"segment"`
	Lambda   float64            `json:"lambda"`
	ZoneAff  map[string]float64 `json:"zone_aff"`
	TagAff   map[string]float64 `json:"tag_aff"`
	DurationPref string         `json:"duration_pref"` // short|medium|long|mixed
}

// ─── Segments ────────────────────────────────────────────────

const (
	SegAnime   = "seg_anime"   // zone_affinity[动画] > 0.5
	SegGame    = "seg_game"    // zone_affinity[游戏] > 0.4
	SegTech    = "seg_tech"    // zone_affinity[科技] > 0.3
	SegLife    = "seg_life"    // zone_affinity[生活] > 0.3
	SegMix     = "seg_mix"     // no dominant zone
)

// SegmentZoneThreshold stores the minimum affinity for each segment.
var SegmentZoneThreshold = map[string]float64{
	SegAnime: 0.5,
	SegGame:  0.4,
	SegTech:  0.3,
	SegLife:  0.3,
}

// SegmentZones maps segments to primary zones of interest.
var SegmentZones = map[string][]string{
	SegAnime: {"动画"},
	SegGame:  {"游戏"},
	SegTech:  {"科技"},
	SegLife:  {"生活"},
}

// ─── Redis keys ──────────────────────────────────────────────

const (
	ProfileKeyPrefix = "user:profile:"
	ProfileTTL       = 7 * 24 * time.Hour
)

func profileKey(uid uint64) string {
	return fmt.Sprintf("%s%d", ProfileKeyPrefix, uid)
}

// ─── Service ─────────────────────────────────────────────────

// UserProfileService builds and serves user profiles.
type UserProfileService struct {
	DB    *gorm.DB
	Redis *redis.Client
}

// GetProfile returns the cached or freshly-built profile for uid.
func (s *UserProfileService) GetProfile(ctx context.Context, uid uint64) *UserProfile {
	// Try Redis cache first.
	p := s.cacheGet(ctx, uid)
	if p != nil {
		return p
	}
	// Build from history.
	p = s.Build(ctx, uid)
	if p != nil {
		s.cacheSet(ctx, p)
	}
	return p
}

// Build constructs a fresh profile from user history tables.
// Returns nil if uid == 0 or user has no history (unauthenticated / new user).
func (s *UserProfileService) Build(ctx context.Context, uid uint64) *UserProfile {
	if uid == 0 {
		return nil
	}

	p := &UserProfile{
		UserID:  uid,
		Version: time.Now().Unix(),
		Segment: SegMix,
		Lambda:  DefaultLambda,
	}

	// Collect video IDs the user has interacted with.
	videoIDs := s.collectVideoIDs(ctx, uid)
	if len(videoIDs) == 0 {
		return nil
	}

	var videos []model.Video
	_ = s.DB.WithContext(ctx).Where("id IN ?", videoIDs).Find(&videos).Error
	if len(videos) == 0 {
		return nil
	}

	// Compute affinities.
	p.ZoneAff = s.zoneAffinity(videos)
	p.TagAff = s.tagAffinity(videos)
	p.DurationPref = s.durationPreference(videos)
	p.Lambda = s.adaptiveLambda(p.ZoneAff, p.DurationPref)
	p.Segment = segmentUser(p.ZoneAff)

	return p
}

// ─── Affinity computation ────────────────────────────────────

func (s *UserProfileService) collectVideoIDs(ctx context.Context, uid uint64) []uint64 {
	ids := make(map[uint64]bool)

	// View history.
	var vh []model.VideoViewHistory
	_ = s.DB.WithContext(ctx).Where("user_id = ?", uid).
		Order("viewed_at DESC").Limit(200).Find(&vh).Error
	for _, h := range vh {
		ids[h.VideoID] = true
	}

	// Likes.
	var likes []model.VideoLike
	_ = s.DB.WithContext(ctx).Where("user_id = ?", uid).
		Order("created_at DESC").Limit(100).Find(&likes).Error
	for _, l := range likes {
		ids[l.VideoID] = true
	}

	// Coins.
	var coins []model.VideoCoin
	_ = s.DB.WithContext(ctx).Where("user_id = ?", uid).
		Order("created_at DESC").Limit(100).Find(&coins).Error
	for _, c := range coins {
		ids[c.VideoID] = true
	}

	// Favorites.
	var faves []model.VideoFavorite
	_ = s.DB.WithContext(ctx).Where("user_id = ?", uid).
		Order("created_at DESC").Limit(100).Find(&faves).Error
	for _, f := range faves {
		ids[f.VideoID] = true
	}

	result := make([]uint64, 0, len(ids))
	for id := range ids {
		result = append(result, id)
	}
	return result
}

func (s *UserProfileService) zoneAffinity(videos []model.Video) map[string]float64 {
	zoneCount := make(map[string]int)
	total := 0
	for _, v := range videos {
		zp := zoneParent(v.Zone)
		if zp == "" {
			continue
		}
		zoneCount[zp]++
		total++
	}
	if total == 0 {
		return nil
	}
	aff := make(map[string]float64, len(zoneCount))
	for z, c := range zoneCount {
		aff[z] = float64(c) / float64(total)
	}
	return aff
}

func (s *UserProfileService) tagAffinity(videos []model.Video) map[string]float64 {
	tagCount := make(map[string]int)
	total := 0
	for _, v := range videos {
		feat := ExtractFeatures(&v)
		for _, t := range feat.Tags {
			tagCount[t]++
			total++
		}
	}
	if total == 0 {
		return nil
	}
	aff := make(map[string]float64, len(tagCount))
	for t, c := range tagCount {
		aff[t] = float64(c) / float64(total)
	}
	return aff
}

func (s *UserProfileService) durationPreference(videos []model.Video) string {
	var short, mid, long float64
	for _, v := range videos {
		d := v.DurationSec
		if d <= 120 {
			short++
		} else if d <= 600 {
			mid++
		} else {
			long++
		}
	}
	total := short + mid + long
	if total < 5 {
		return "mixed"
	}
	best := math.Max(math.Max(short/total, mid/total), long/total)
	if best > 0.6 {
		switch {
		case short/total == best:
			return "short"
		case mid/total == best:
			return "medium"
		default:
			return "long"
		}
	}
	return "mixed"
}

func (s *UserProfileService) adaptiveLambda(
	zoneAff map[string]float64,
	durPref string,
) float64 {
	lambda := DefaultLambda

	// Higher repeat-zone ratio → user prefers depth → higher λ.
	maxAff := 0.0
	for _, v := range zoneAff {
		if v > maxAff {
			maxAff = v
		}
	}
	lambda += 0.1 * (maxAff - 0.3)
	if lambda > 0.9 {
		lambda = 0.9
	}
	if lambda < 0.5 {
		lambda = 0.5
	}
	return lambda
}

func segmentUser(zoneAff map[string]float64) string {
	if v := zoneAff["动画"]; v > SegmentZoneThreshold[SegAnime] {
		return SegAnime
	}
	if v := zoneAff["游戏"]; v > SegmentZoneThreshold[SegGame] {
		return SegGame
	}
	if v := zoneAff["科技"]; v > SegmentZoneThreshold[SegTech] {
		return SegTech
	}
	if v := zoneAff["生活"]; v > SegmentZoneThreshold[SegLife] {
		return SegLife
	}
	return SegMix
}

// ─── Redis cache ──────────────────────────────────────────────

func (s *UserProfileService) cacheGet(ctx context.Context, uid uint64) *UserProfile {
	data, err := s.Redis.Get(ctx, profileKey(uid)).Bytes()
	if err != nil {
		return nil
	}
	p := &UserProfile{}
	if json.Unmarshal(data, p) != nil {
		return nil
	}
	return p
}

func (s *UserProfileService) cacheSet(ctx context.Context, p *UserProfile) {
	data, _ := json.Marshal(p)
	s.Redis.Set(ctx, profileKey(p.UserID), data, ProfileTTL)
}
