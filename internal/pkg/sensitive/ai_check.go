package sensitive

// AIChecker is an optional LLM-based semantic review layer layered on top of
// the keyword Filter. It catches AIGC-era paraphrased / homophone / implicit
// violations that hard substring matching misses (the "漏报" problem).
//
// Design:
//   - The keyword Filter remains the mandatory hard gate (Rule R-BIZ-5).
//   - AIChecker runs only for content that passed the keyword filter.
//   - Fail-open: if the LLM is unreachable / times out / returns junk,
//     the content is ALLOWED through (keyword gate still applies) and the
//     failure is logged for ops visibility. Blocking on LLM outage would
//     take down all UGC, which is worse than occasional miss.
//   - All verdicts and failures are logged via zap with request context.

import (
	"context"
	"sync"
	"time"

	"go.uber.org/zap"
)

// ReviewFunc invokes the configured LLM and returns (blocked, err).
// blocked=true means the LLM judged the content as violating.
type ReviewFunc func(ctx context.Context, text string) (blocked bool, err error)

// AIChecker wraps a ReviewFunc with enable/disable and logging.
type AIChecker struct {
	enabled bool
	timeout time.Duration
	log     *zap.Logger
	review  ReviewFunc

	mu sync.RWMutex
}

// NewAIChecker creates the checker. review may be nil (disabled).
func NewAIChecker(enabled bool, timeout time.Duration, lg *zap.Logger, review ReviewFunc) *AIChecker {
	if lg == nil {
		lg = zap.NewNop()
	}
	return &AIChecker{enabled: enabled && review != nil, timeout: timeout, log: lg, review: review}
}

// Enabled reports whether the AI review layer is active.
func (a *AIChecker) Enabled() bool {
	a.mu.RLock()
	defer a.mu.RUnlock()
	return a.enabled
}

// SetEnabled toggles the layer at runtime (ops switch).
func (a *AIChecker) SetEnabled(v bool) {
	a.mu.Lock()
	defer a.mu.Unlock()
	a.enabled = v && a.review != nil
}

// Review returns true if the content must be blocked.
//
// Fail-open semantics: any error (timeout / HTTP / parse) → (false, err) so
// the caller lets content through and logs the issue.
func (a *AIChecker) Review(ctx context.Context, text string) (bool, error) {
	if !a.Enabled() || a.review == nil {
		return false, nil
	}
	ctx, cancel := context.WithTimeout(ctx, a.timeout)
	defer cancel()

	blocked, err := a.review(ctx, text)
	if err != nil {
		a.log.Warn("ai sensitive review failed (fail-open, keyword gate still applies)",
			zap.String("text_preview", truncate(text, 80)), zap.Error(err))
		return false, err
	}
	if blocked {
		a.log.Info("ai sensitive review blocked",
			zap.String("text_preview", truncate(text, 80)))
	} else {
		a.log.Info("ai sensitive review passed",
			zap.String("text_preview", truncate(text, 80)))
	}
	return blocked, nil
}

// DefaultReviewPrompt is the system prompt for the content-safety judge.
const DefaultReviewPrompt = `你是一名中文社区内容安全审核员。判断用户发表的内容是否包含违规信息，违规类型包括：色情低俗、暴力恐怖、赌博、毒品、违法内容、人身攻击与辱骂、垃圾广告、涉政敏感、引战谣言。
规则：
1. 只根据内容本身判断，不臆测作者意图。
2. 正常讨论、批评、玩梗、科普不属于违规。
3. 谐音、拆字、变体字规避审查的违规表达必须识别为违规。
请只回复 PASS 或 BLOCK，不要输出任何其他内容。`

func truncate(s string, n int) string {
	r := []rune(s)
	if len(r) <= n {
		return s
	}
	return string(r[:n]) + "…"
}
