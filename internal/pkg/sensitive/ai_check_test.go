package sensitive

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func newTestChecker(review ReviewFunc) *AIChecker {
	return NewAIChecker(true, 2*time.Second, nil, review)
}

// 启用条件：enabled=true 且 review 非 nil。
func TestAICheckerEnabled(t *testing.T) {
	ck := NewAIChecker(true, time.Second, nil, func(ctx context.Context, text string) (bool, error) {
		return false, nil
	})
	assert.True(t, ck.Enabled())

	// review 为 nil → 自动禁用
	ck2 := NewAIChecker(true, time.Second, nil, nil)
	assert.False(t, ck2.Enabled())

	// enabled=false → 禁用
	ck3 := NewAIChecker(false, time.Second, nil, func(ctx context.Context, text string) (bool, error) {
		return false, nil
	})
	assert.False(t, ck3.Enabled())
}

// LLM 判定违规 → 拦截。
func TestAICheckerBlocks(t *testing.T) {
	ck := newTestChecker(func(ctx context.Context, text string) (bool, error) {
		return true, nil
	})
	blocked, err := ck.Review(context.Background(), "违规内容")
	assert.NoError(t, err)
	assert.True(t, blocked)
}

// LLM 判定通过 → 放行。
func TestAICheckerAllows(t *testing.T) {
	ck := newTestChecker(func(ctx context.Context, text string) (bool, error) {
		return false, nil
	})
	blocked, err := ck.Review(context.Background(), "正常内容")
	assert.NoError(t, err)
	assert.False(t, blocked)
}

// fail-open：LLM 报错 → 放行（由词表硬门禁兜底）+ 返回 err 供日志。
func TestAICheckerFailOpenOnError(t *testing.T) {
	ck := newTestChecker(func(ctx context.Context, text string) (bool, error) {
		return false, errors.New("llm timeout")
	})
	blocked, err := ck.Review(context.Background(), "内容")
	assert.Error(t, err)
	assert.False(t, blocked, "LLM 故障必须放行（fail-open），否则全站 UGC 会被打挂")
}

// 禁用状态直接放行，不调 review。
func TestAICheckerDisabledPasses(t *testing.T) {
	called := false
	ck := NewAIChecker(false, time.Second, nil, func(ctx context.Context, text string) (bool, error) {
		called = true
		return true, nil
	})
	blocked, err := ck.Review(context.Background(), "内容")
	assert.NoError(t, err)
	assert.False(t, blocked)
	assert.False(t, called, "禁用时不应调用 review")
}

// 运行时开关 SetEnabled。
func TestAICheckerSetEnabled(t *testing.T) {
	ck := newTestChecker(func(ctx context.Context, text string) (bool, error) {
		return true, nil
	})
	ck.SetEnabled(false)
	assert.False(t, ck.Enabled())

	ck.SetEnabled(true)
	assert.True(t, ck.Enabled())
	blocked, _ := ck.Review(context.Background(), "内容")
	assert.True(t, blocked)
}
