package ffmpeg

import (
	"testing"
)

func TestABRLadder_1080Source(t *testing.T) {
	ladder := ABRLadder(1080)
	if len(ladder) != 3 {
		t.Fatalf("expected 3 renditions for 1080p, got %d", len(ladder))
	}
	// Order from highest to lowest, strictly smaller than source.
	want := []string{"720p", "480p", "360p"}
	for i, v := range ladder {
		if v.Label != want[i] {
			t.Errorf("ladder[%d].Label = %q, want %q", i, v.Label, want[i])
		}
		if v.Height >= 1080 {
			t.Errorf("ladder[%d].Height %d not smaller than source 1080", i, v.Height)
		}
	}
}

func TestABRLadder_720Source(t *testing.T) {
	ladder := ABRLadder(720)
	if len(ladder) != 2 {
		t.Fatalf("expected 2 renditions for 720p (480p,360p), got %d", len(ladder))
	}
	if ladder[0].Label != "480p" || ladder[1].Label != "360p" {
		t.Errorf("unexpected ladder for 720p: %+v", ladder)
	}
}

func TestABRLadder_SmallOrUnknown(t *testing.T) {
	if got := ABRLadder(360); len(got) != 0 {
		t.Errorf("360p source should have no downscale ladder, got %+v", got)
	}
	if got := ABRLadder(0); got != nil {
		t.Errorf("unknown source height should yield nil, got %v", got)
	}
	if got := ABRLadder(-1); got != nil {
		t.Errorf("negative source height should yield nil, got %v", got)
	}
}

func TestABRLadder_StrictMonotonic(t *testing.T) {
	ladder := ABRLadder(2160)
	for i := 1; i < len(ladder); i++ {
		if ladder[i].Height >= ladder[i-1].Height {
			t.Fatalf("ladder not strictly decreasing at %d: %+v", i, ladder)
		}
	}
}
