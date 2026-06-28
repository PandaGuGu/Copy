package service

import (
	"math"
	"testing"
	"time"

	"minibili/internal/model"
)

// ─── Helpers ──────────────────────────────────────────────────

func newVideo(id, uid uint64, play, like, coin uint64, zone string, tagsJSON string, daysAgo int) *model.Video {
	return &model.Video{
		ID:      id,
		UserID:  uid,
		Title:   "test",
		Status:  "published",
		PlayCount:  play,
		LikeCount:  like,
		CoinCount:  coin,
		FavCount:   0,
		DanmakuCount: 0,
		Zone:     zone,
		TagsJSON: tagsJSON,
		CreatedAt: time.Now().Add(-time.Duration(daysAgo) * 24 * time.Hour),
	}
}

func features(vs ...*model.Video) []VideoFeatures {
	out := make([]VideoFeatures, len(vs))
	for i, v := range vs {
		out[i] = ExtractFeatures(v)
	}
	return out
}

// ─── Tests ────────────────────────────────────────────────────

func TestRelevanceScore(t *testing.T) {
	v := newVideo(1, 100, 1000, 100, 50, "游戏", `["实况","RPG"]`, 0)
	s := RelevanceScore(v)
	// play×1 + like×10 + coin×20 = 1000 + 1000 + 1000 = 3000
	expected := float64(3000)
	if math.Abs(s-expected) > 0.01 {
		t.Errorf("relevance = %f, want %f", s, expected)
	}
}

func TestRelevanceScoreTimeDecay(t *testing.T) {
	v0 := newVideo(1, 100, 1000, 100, 50, "游戏", "", 0)
	v30 := newVideo(2, 100, 1000, 100, 50, "游戏", "", 30)
	v30.CreatedAt = time.Now().Add(-30 * 24 * time.Hour) // 30 days ago

	s0 := RelevanceScore(v0)
	s30 := RelevanceScore(v30)

	if s30 >= s0 {
		t.Errorf("30-day-old video score %f should be less than fresh %f", s30, s0)
	}
	// decay = e^(-0.01 × 30) ≈ 0.74
	ratio := s30 / s0
	if ratio < 0.7 || ratio > 0.78 {
		t.Errorf("decay ratio = %f, want ~0.74", ratio)
	}
}

func TestTagSimilarity_Jaccard(t *testing.T) {
	sim := TagSimilarity(
		[]string{"实况", "RPG", "单机"},
		[]string{"实况", "RPG", "手游"},
	)
	// intersection=2 (实况,RPG), union=4 (实况,RPG,单机,手游)
	expected := 2.0 / 4.0
	if math.Abs(sim-expected) > 0.001 {
		t.Errorf("Jaccard = %f, want %f", sim, expected)
	}
}

func TestTagSimilarity_Empty(t *testing.T) {
	if TagSimilarity(nil, []string{"a"}) != 0 {
		t.Error("empty tag set should give 0 similarity")
	}
	if TagSimilarity([]string{"a"}, nil) != 0 {
		t.Error("empty tag set should give 0 similarity")
	}
}

func TestTagSimilarity_Disjoint(t *testing.T) {
	sim := TagSimilarity([]string{"a", "b"}, []string{"c", "d"})
	if sim != 0 {
		t.Errorf("disjoint = %f, want 0", sim)
	}
}

func TestZoneSimilarity(t *testing.T) {
	if ZoneSimilarity("游戏", "游戏") != 0.3 {
		t.Error("same zone parent should be 0.3")
	}
	if ZoneSimilarity("游戏", "科技") != 0 {
		t.Error("different zone should be 0")
	}
	if ZoneSimilarity("", "") != 0 {
		t.Error("empty zones should be 0")
	}
}

func TestCreatorPenalty(t *testing.T) {
	if CreatorPenalty(100, 100) != 0.5 {
		t.Error("same creator should be 0.5")
	}
	if CreatorPenalty(100, 200) != 0 {
		t.Error("different creator should be 0")
	}
	if CreatorPenalty(0, 0) != 0 {
		t.Error("zero user id should be 0")
	}
}

func TestItemSimilarity_Identical(t *testing.T) {
	v := newVideo(1, 100, 1000, 10, 5, "游戏", `["实况","RPG"]`, 0)
	f := features(v, v)
	sim := ItemSimilarity(&f[0], &f[1])
	// Tag=1.0×0.5=0.5, Zone=0.3×0.3=0.09, Creator=0.5×0.2=0.1 → 0.69
	expected := 0.5*1.0 + 0.3*0.3 + 0.2*0.5
	if math.Abs(sim-expected) > 0.001 {
		t.Errorf("identical similarity = %f, want %f", sim, expected)
	}
}

func TestItemSimilarity_Different(t *testing.T) {
	v1 := newVideo(1, 100, 1000, 10, 5, "游戏", `["实况"]`, 0)
	v2 := newVideo(2, 200, 500, 5, 3, "科技", `["教程"]`, 0)
	f := features(v1, v2)
	sim := ItemSimilarity(&f[0], &f[1])
	if sim != 0 {
		t.Errorf("completely different videos should have 0 similarity, got %f", sim)
	}
}

func TestMMR_Basic(t *testing.T) {
	// 3 videos with similar relevance but different diversity profiles:
	// v1, v2: same zone/creator/tags (high similarity to each other)
	// v3: different zone/creator/tags (fully diverse)
	v1 := newVideo(1, 100, 1000, 200, 50, "游戏", `["实况","RPG"]`, 0)
	v2 := newVideo(2, 100, 1050, 220, 55, "游戏", `["实况","动作"]`, 0) // slightly higher relevance
	v3 := newVideo(3, 200, 1000, 200, 50,  "科技", `["教程","编程"]`, 0) // same relevance, different

	pool := features(v1, v2, v3)

	// λ=0.7: first pick v2 (highest), second should be v3 (diverse, equal relevance to v1 but no penalty)
	result := MMR(pool, 2, 0.7)
	if len(result) != 2 {
		t.Fatalf("MMR returned %d items, want 2", len(result))
	}
	// First pick should be v2 (highest relevance)
	if result[0] != 1 {
		t.Errorf("first pick should be v2 (idx=1), got idx=%d", result[0])
	}
	// Second pick should be v3 — identical relevance to v1 but no diversity penalty
	if result[1] != 2 {
		t.Errorf("second pick should be v3 (idx=2) for diversity, got idx=%d (v1 MMR=%.1f, v3 MMR=%.1f)",
			result[1],
			0.7*RelevanceScore(pool[0].Video)-0.3*ItemSimilarity(&pool[0], &pool[1]),
			0.7*RelevanceScore(pool[2].Video)-0.3*ItemSimilarity(&pool[2], &pool[1]),
		)
	}
}

func TestMMR_KEqualsN(t *testing.T) {
	v := []*model.Video{
		newVideo(1, 100, 100, 10, 5, "游戏", "", 0),
		newVideo(2, 200, 200, 20, 5, "科技", "", 0),
	}
	pool := features(v...)
	result := MMR(pool, 2, 0.7)
	if len(result) != 2 {
		t.Fatalf("want %d, got %d", 2, len(result))
	}
}

func TestMMR_AllHighRelevance(t *testing.T) {
	// All 6 videos from the same zone/creator — diversity should spread them
	videos := make([]*model.Video, 6)
	for i := range videos {
		videos[i] = newVideo(
			uint64(i+1), 100,
			uint64(1000+i*100), uint64(10+i), 5,
			"游戏", `["实况"]`, 0,
		)
	}
	pool := features(videos...)
	result := MMR(pool, 3, 0.7)
	if len(result) != 3 {
		t.Fatalf("want 3, got %d", len(result))
	}
	// All returned indices must be valid
	for _, idx := range result {
		if idx < 0 || idx >= len(pool) {
			t.Errorf("invalid index %d", idx)
		}
	}
}

func TestMMR_LambdaZero(t *testing.T) {
	// λ=0 → pure diversity
	v1 := newVideo(1, 100, 3000, 500, 50, "游戏", `["实况","RPG"]`, 0)     // highest relevance
	v2 := newVideo(2, 100, 2000, 400, 50, "游戏", `["实况","动作"]`, 0)     // similar
	v3 := newVideo(3, 200, 500,  10,  5,  "科技", `["教程","编程"]`, 0)     // different

	pool := features(v1, v2, v3)
	result := MMR(pool, 2, 0)
	if len(result) != 2 {
		t.Fatalf("want 2, got %d", len(result))
	}
	// With λ=0, first pick is still v1 (base score highest), second must be v3 (most diverse)
	if result[1] != 2 {
		t.Errorf("pure diversity should pick v3 as second, got idx=%d", result[1])
	}
}

func TestExtractFeatures(t *testing.T) {
	v := &model.Video{
		ID:       1,
		Zone:     "生活-日常",
		TagsJSON: `["vlog","美食"]`,
	}
	f := ExtractFeatures(v)
	if f.ZoneParent != "生活" {
		t.Errorf("zone parent = %q, want %q", f.ZoneParent, "生活")
	}
	if len(f.Tags) != 2 || f.Tags[0] != "vlog" || f.Tags[1] != "美食" {
		t.Errorf("tags = %v, want [vlog, 美食]", f.Tags)
	}
}

func TestExtractFeatures_NoDash(t *testing.T) {
	v := &model.Video{Zone: "动画"}
	f := ExtractFeatures(v)
	if f.ZoneParent != "动画" {
		t.Errorf("zone parent without dash = %q, want %q", f.ZoneParent, "动画")
	}
}

func TestDPP_Basic(t *testing.T) {
	v1 := newVideo(1, 100, 1000, 200, 50, "游戏", `["实况","RPG"]`, 0)
	v2 := newVideo(2, 100, 2000, 400, 100, "游戏", `["实况","动作"]`, 0)
	v3 := newVideo(3, 200, 500,  50,  20,  "科技", `["教程","编程"]`, 0)

	pool := features(v1, v2, v3)
	result := DPP(pool, 2)
	if len(result) != 2 {
		t.Fatalf("DPP returned %d items, want 2", len(result))
	}
	// DPP should also pick v2 first (highest quality), then v3 for diversity
	if result[0] != 1 {
		t.Errorf("DPP first pick should be v2 (idx=1), got idx=%d", result[0])
	}
}

func TestDPP_AllSame(t *testing.T) {
	// All identical — DPP should still return them (no penalty if sim=0 between identical items? No, sim is high)
	// Actually identical items will have high similarity, so DPP will penalize them.
	videos := make([]*model.Video, 5)
	for i := range videos {
		videos[i] = newVideo(
			uint64(i+1), 100,
			uint64(1000+i*10), 10, 5,
			"游戏", `["实况"]`, 0,
		)
	}
	pool := features(videos...)
	result := DPP(pool, 3)
	if len(result) != 3 {
		t.Fatalf("want 3, got %d", len(result))
	}
	for _, idx := range result {
		if idx < 0 || idx >= len(pool) {
			t.Errorf("invalid index %d", idx)
		}
	}
}

func TestMMRVideos_Wrapper(t *testing.T) {
	v1 := newVideo(1, 100, 100, 10, 5, "游戏", "", 0)
	pool := features(v1)
	result := MMRVideos(pool, 1, 0.7)
	if len(result) != 1 || result[0].ID != 1 {
		t.Errorf("MMRVideos failed")
	}
}
