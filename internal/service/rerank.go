// Package service — diversity re-ranking algorithms.
//
// MMR (Maximal Marginal Relevance):
//
//	MMR = argmax [ λ × relevance(i) − (1−λ) × max similarity(i, j) ]
//	         i∈C\S                              j∈S
//
// where C = candidate pool, S = already-selected set, λ ∈ [0,1].
// Higher λ → more relevance, lower λ → more diversity.
package service

import (
	"encoding/json"
	"math"
	"strings"
	"time"

	"minibili/internal/model"
)

// ─── Constants ───────────────────────────────────────────────

const (
	DefaultLambda = 0.7

	// Score weights (relevance).
	WeightPlay     = 1.0
	WeightLike     = 10.0
	WeightCoin     = 20.0
	WeightFav      = 5.0
	WeightDanmaku  = 3.0

	// Time decay: e^(-λ * days), λ = 0.01.
	// 7d → 0.93x, 30d → 0.74x.
	TimeDecayRate = 0.01

	// Similarity weights.
	SimWeightTag     = 0.5
	SimWeightZone    = 0.3
	SimWeightCreator = 0.2
)

// ─── Feature extraction ──────────────────────────────────────

// VideoFeatures holds pre-extracted features for one candidate.
type VideoFeatures struct {
	Video   *model.Video
	Tags    []string
	ZoneParent string
}

// ExtractFeatures parses TagsJSON and zone from a video.
func ExtractFeatures(v *model.Video) VideoFeatures {
	f := VideoFeatures{Video: v, ZoneParent: zoneParent(v.Zone)}
	if v.TagsJSON != "" {
		json.Unmarshal([]byte(v.TagsJSON), &f.Tags)
	}
	return f
}

func zoneParent(zone string) string {
	if idx := strings.IndexByte(zone, '-'); idx > 0 {
		return zone[:idx]
	}
	return zone
}

// ─── Relevance scoring ───────────────────────────────────────

// RelevanceScore computes a weighted quality score with time decay.
//
//	score = (play×1 + like×10 + coin×20 + fav×5 + danmaku×3) × e^(-λ·days)
func RelevanceScore(v *model.Video) float64 {
	base := float64(v.PlayCount)*WeightPlay +
		float64(v.LikeCount)*WeightLike +
		float64(v.CoinCount)*WeightCoin +
		float64(v.FavCount)*WeightFav +
		float64(v.DanmakuCount)*WeightDanmaku
	days := time.Since(v.CreatedAt).Hours() / 24
	decay := math.Exp(-TimeDecayRate * days)
	return base * decay
}

// ─── Similarity ──────────────────────────────────────────────

// TagSimilarity returns Jaccard similarity between two tag sets.
// Returns 0 if either set is empty.
func TagSimilarity(a, b []string) float64 {
	if len(a) == 0 || len(b) == 0 {
		return 0
	}
	intersection := 0
	setA := make(map[string]struct{}, len(a))
	for _, t := range a {
		setA[t] = struct{}{}
	}
	for _, t := range b {
		if _, ok := setA[t]; ok {
			intersection++
		}
	}
	union := len(setA) + len(b) - intersection
	if union == 0 {
		return 0
	}
	return float64(intersection) / float64(union)
}

// ZoneSimilarity returns 0.3 if both videos share the same zone parent.
func ZoneSimilarity(za, zb string) float64 {
	if za == zb && za != "" {
		return 0.3
	}
	return 0
}

// CreatorPenalty returns 0.5 if both videos are by the same creator.
func CreatorPenalty(uidA, uidB uint64) float64 {
	if uidA == uidB && uidA != 0 {
		return 0.5
	}
	return 0
}

// ItemSimilarity computes combined similarity between two videos.
//
//	sim = 0.5 × TagJaccard + 0.3 × ZoneMatch + 0.2 × CreatorCheck
func ItemSimilarity(a, b *VideoFeatures) float64 {
	return SimWeightTag*TagSimilarity(a.Tags, b.Tags) +
		SimWeightZone*ZoneSimilarity(a.ZoneParent, b.ZoneParent) +
		SimWeightCreator*CreatorPenalty(a.Video.UserID, b.Video.UserID)
}

// ─── MMR ─────────────────────────────────────────────────────

// MMR selects up to k items from pool via Maximal Marginal Relevance.
//
// λ controls the relevance/diversity trade-off (0 = pure diversity, 1 = pure relevance).
// Returns indices into the pool in selection order.
func MMR(pool []VideoFeatures, k int, lambda float64) []int {
	if k <= 0 || len(pool) == 0 {
		return nil
	}
	if len(pool) <= k {
		result := make([]int, len(pool))
		for i := range result {
			result[i] = i
		}
		return result
	}

	selected := make([]int, 0, k)
	remained := make(map[int]bool, len(pool))
	for i := range pool {
		remained[i] = true
	}

	// Pre-compute relevance scores.
	scores := make([]float64, len(pool))
	for i, f := range pool {
		scores[i] = RelevanceScore(f.Video)
	}

	// Greedy selection.
	for len(selected) < k {
		bestIdx := -1
		bestScore := math.Inf(-1)
		var maxSim float64

		for idx := range remained {
			// Compute max similarity to already-selected items.
			maxSim = 0
			for _, s := range selected {
				sim := ItemSimilarity(&pool[idx], &pool[s])
				if sim > maxSim {
					maxSim = sim
				}
			}
			mmr := lambda*scores[idx] - (1-lambda)*maxSim
			if mmr > bestScore {
				bestScore = mmr
				bestIdx = idx
			}
		}
		if bestIdx < 0 {
			break
		}
		selected = append(selected, bestIdx)
		delete(remained, bestIdx)
	}
	return selected
}

// MMRVideos is a convenience wrapper that returns []*model.Video.
func MMRVideos(pool []VideoFeatures, k int, lambda float64) []*model.Video {
	idxs := MMR(pool, k, lambda)
	result := make([]*model.Video, len(idxs))
	for i, idx := range idxs {
		result[i] = pool[idx].Video
	}
	return result
}

// ─── DPP (optional, for future use) ──────────────────────────

// DPP selects up to k items via Determinantal Point Process (greedy MAP).
//
// Kernel: L_ii = quality_i²,  L_ij = quality_i × quality_j × similarity(i,j)
// Greedy step: j = argmax L_jj − Σ_{i∈S} (L_ij² / L_ii)
//             = argmax q_j² × (1 − Σ_{i∈S} sim(i,j)²)
func DPP(pool []VideoFeatures, k int) []int {
	if k <= 0 || len(pool) == 0 {
		return nil
	}
	if len(pool) <= k {
		result := make([]int, len(pool))
		for i := range result {
			result[i] = i
		}
		return result
	}

	selected := make([]int, 0, k)
	remained := make(map[int]bool, len(pool))
	for i := range pool {
		remained[i] = true
	}

	qualities := make([]float64, len(pool))
	for i, f := range pool {
		qualities[i] = RelevanceScore(f.Video)
	}

	for len(selected) < k {
		bestIdx := -1
		bestGain := math.Inf(-1)

		for idx := range remained {
			q := qualities[idx]
			gain := q * q // L_jj

			// Subtract L_ij² / L_ii = q_j² × sim(i,j)²
			for _, s := range selected {
				sim := ItemSimilarity(&pool[idx], &pool[s])
				gain -= q * q * sim * sim
			}

			if gain > bestGain {
				bestGain = gain
				bestIdx = idx
			}
		}
		if bestIdx < 0 {
			break
		}
		selected = append(selected, bestIdx)
		delete(remained, bestIdx)
	}
	return selected
}
