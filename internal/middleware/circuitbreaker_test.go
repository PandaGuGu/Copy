package middleware

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func newTestBreaker() *CircuitBreaker {
	return NewCircuitBreaker("test", BreakerConfig{
		Enabled:      true,
		FailureRate:  0.5,
		MinRequests:  3,
		Window:       10 * time.Second,
		OpenTimeout:  50 * time.Millisecond,
		HalfOpenMax:  2,
		SkipPrefixes: []string{"/api/v1/metrics"},
	})
}

// closed → open: 3 failures out of 3 (100% ≥ 50%) → open.
func TestBreakerOpensOnErrorRate(t *testing.T) {
	br := newTestBreaker()

	for i := 0; i < 3; i++ {
		assert.True(t, br.Allow(), "request %d should pass while closed", i)
		br.Failure()
	}

	assert.Equal(t, "open", br.StateName(), "breaker should open after error rate threshold")

	// While open, requests fail fast.
	assert.False(t, br.Allow(), "requests should be rejected while open")
	_, _, rejects := br.Metrics()
	assert.Equal(t, int64(1), rejects)
}

// open → half-open → closed: probes succeed → closed.
func TestBreakerRecoversToClosed(t *testing.T) {
	br := newTestBreaker()

	for i := 0; i < 3; i++ {
		br.Allow()
		br.Failure()
	}
	assert.Equal(t, "open", br.StateName())

	// Wait for OpenTimeout to elapse → half-open probes.
	time.Sleep(60 * time.Millisecond)

	assert.True(t, br.Allow(), "first probe should pass in half-open")
	br.Success()
	assert.Equal(t, "half_open", br.StateName())

	assert.True(t, br.Allow(), "second probe should pass")
	br.Success()

	assert.Equal(t, "closed", br.StateName(), "all probes ok → closed")
}

// half-open probe failure → back to open (full cooldown).
func TestBreakerProbeFailureReopens(t *testing.T) {
	br := newTestBreaker()

	for i := 0; i < 3; i++ {
		br.Allow()
		br.Failure()
	}
	assert.Equal(t, "open", br.StateName())

	time.Sleep(60 * time.Millisecond)

	assert.True(t, br.Allow(), "first probe should pass")
	br.Failure() // probe fails

	assert.Equal(t, "open", br.StateName(), "probe failure → open again")
	assert.False(t, br.Allow(), "still open → reject")
}

// Below MinRequests, breaker must NOT open even with high error rate.
func TestBreakerStaysClosedBelowMinRequests(t *testing.T) {
	br := newTestBreaker()

	// 2 requests (below MinRequests=3), all fail → still closed.
	for i := 0; i < 2; i++ {
		assert.True(t, br.Allow())
		br.Failure()
	}
	assert.Equal(t, "closed", br.StateName())

	// 3rd failure pushes total to MinRequests with 100% error → open.
	assert.True(t, br.Allow())
	br.Failure()
	assert.Equal(t, "open", br.StateName())
}

func TestBreakerReset(t *testing.T) {
	br := newTestBreaker()
	for i := 0; i < 3; i++ {
		br.Allow()
		br.Failure()
	}
	assert.Equal(t, "open", br.StateName())

	br.Reset()
	assert.Equal(t, "closed", br.StateName())
	assert.True(t, br.Allow())
}
