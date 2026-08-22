package service

import (
	"testing"
)

func TestColdStartEligible(t *testing.T) {
	fresh := newVideo(1, 100, 0, 0, 0, "游戏", "", 0)           // fresh, no engagement
	freshEngaged := newVideo(2, 100, 10, 0, 0, "游戏", "", 0)   // fresh but has plays
	old := newVideo(3, 100, 0, 0, 0, "游戏", "", 10)            // 10 days old, no engagement
	oldEngaged := newVideo(4, 100, 100, 10, 0, "游戏", "", 100) // old and engaged

	if !ColdStartEligible(fresh) {
		t.Error("fresh 0-engagement video should be cold-start eligible")
	}
	if ColdStartEligible(freshEngaged) {
		t.Error("fresh video with plays should NOT be cold-start eligible")
	}
	if ColdStartEligible(old) {
		t.Error("old 0-engagement video should NOT be cold-start eligible")
	}
	if ColdStartEligible(oldEngaged) {
		t.Error("old engaged video should NOT be cold-start eligible")
	}
	if ColdStartEligible(nil) {
		t.Error("nil video should not be cold-start eligible")
	}
}

func TestEffectiveRelevance_ColdStartFloor(t *testing.T) {
	fresh := newVideo(1, 100, 0, 0, 0, "游戏", "", 0)
	if got := EffectiveRelevance(fresh); got != ColdStartPlayFloor {
		t.Errorf("fresh cold-start relevance = %f, want floor %f", got, ColdStartPlayFloor)
	}

	// Old 0-engagement video has no floor → relevance decays to ~0.
	old := newVideo(2, 100, 0, 0, 0, "游戏", "", 30)
	if got := EffectiveRelevance(old); got != 0 {
		t.Errorf("old 0-engagement relevance = %f, want 0", got)
	}

	// Fresh video with heavy engagement is above the floor (uses raw relevance).
	hot := newVideo(3, 100, 5000, 300, 100, "游戏", "", 0)
	if got := EffectiveRelevance(hot); got <= ColdStartPlayFloor {
		t.Errorf("hot fresh relevance = %f, want > floor %f", got, ColdStartPlayFloor)
	}
}

func TestCoarseRank_FreshLift(t *testing.T) {
	oldPopular := newVideo(1, 100, 100000, 500, 100, "游戏", `["实况"]`, 10) // high relevance
	fresh := newVideo(2, 100, 0, 0, 0, "生活", `["vlog"]`, 0)              // cold-start floor
	oldLow := newVideo(3, 100, 5, 0, 0, "游戏", `["实况"]`, 10)              // low relevance

	pool := features(oldPopular, fresh, oldLow)
	top2 := CoarseRank(pool, 2)
	if len(top2) != 2 {
		t.Fatalf("CoarseRank top2 = %v, want 2 items", top2)
	}

	// The fresh video must survive the coarse stage even though it has 0 plays.
	freshKept := false
	for _, idx := range top2 {
		if pool[idx].Video.ID == fresh.ID {
			freshKept = true
		}
	}
	if !freshKept {
		t.Errorf("fresh cold-start video excluded from coarse top-2: %v", top2)
	}

	// And it ranks below the popular video but above the low-engagement one.
	if top2[0] != 0 {
		t.Errorf("old popular should rank first (idx=0), got %v", top2)
	}
}

func TestCoarseTrim_ReducesPool(t *testing.T) {
	old := newVideo(1, 100, 100000, 0, 0, "游戏", "", 10)
	fresh := newVideo(2, 100, 0, 0, 0, "游戏", "", 0)
	pool := features(old, fresh)

	trimmed := CoarseTrim(pool, 1)
	if len(trimmed) != 1 {
		t.Fatalf("CoarseTrim top1 len = %d, want 1", len(trimmed))
	}
	if trimmed[0].Video.ID != old.ID {
		t.Errorf("top-1 by coarse relevance should be the popular old video")
	}

	// topN >= len(pool) → returns the pool as-is.
	if got := CoarseTrim(pool, 5); len(got) != len(pool) {
		t.Errorf("CoarseTrim(topN>=len) changed size: %d", len(got))
	}
}
