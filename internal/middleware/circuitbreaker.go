// CircuitBreaker is a per-instance, zero-dependency circuit breaker.
//
// States:
//
//	closed     — requests pass through; failures/total counted in a sliding window.
//	             When error rate >= FailureRate and total >= MinRequests → open.
//	open       — all requests fail fast with 503 for OpenTimeout, protecting
//	             downstream dependencies (DB / Redis / ES / LLM).
//	half-open  — allow up to HalfOpenMax probe requests. All succeed → closed;
//	             any fails → open again (full retry protection).
//
// Thread-safe: state transitions guarded by a mutex; hot-path counters are
// plain ints inside the lock (window reset check is cheap).
package middleware

import (
	"net/http"
	"sync"
	"sync/atomic"
	"time"

	"github.com/gin-gonic/gin"

	"minibili/internal/errcode"
	"minibili/internal/pkg/resp"
)

const (
	cbClosed int32 = iota
	cbOpen
	cbHalfOpen
)

// BreakerConfig holds circuit breaker tuning parameters.
type BreakerConfig struct {
	Enabled      bool          // master switch (default false = pass-through)
	FailureRate  float64       // open when error rate exceeds this (0.0-1.0)
	MinRequests  int           // minimum requests in window before evaluation
	Window       time.Duration // sliding window length
	OpenTimeout  time.Duration // how long to stay open before half-open
	HalfOpenMax  int           // probe requests allowed in half-open
	SkipPrefixes []string      // path prefixes never counted (e.g. /metrics)
}

// DefaultBreakerConfig returns production-reasonable defaults.
func DefaultBreakerConfig() BreakerConfig {
	return BreakerConfig{
		Enabled:      false,
		FailureRate:  0.5,
		MinRequests:  20,
		Window:       10 * time.Second,
		OpenTimeout:  10 * time.Second,
		HalfOpenMax:  5,
		SkipPrefixes: []string{"/api/v1/metrics"},
	}
}

// CircuitBreaker is a named breaker instance.
type CircuitBreaker struct {
	name string
	cfg  BreakerConfig

	mu       sync.Mutex
	state    int32
	winStart time.Time
	total    int
	fails    int
	openedAt time.Time
	probes   int
	probeOK  int
	probeBad int

	// Exposed gauges for metrics endpoint (atomic, lock-free reads).
	statState  atomic.Int32 // mirrors state for lock-free read
	statTotal  atomic.Int64
	statFails  atomic.Int64
	statReject atomic.Int64 // fast-fail rejections while open
}

// NewCircuitBreaker creates a named breaker.
func NewCircuitBreaker(name string, cfg BreakerConfig) *CircuitBreaker {
	b := &CircuitBreaker{name: name, cfg: cfg, winStart: time.Now()}
	b.statState.Store(cbClosed)
	return b
}

// StateName returns a human-readable state: closed / open / half_open.
func (b *CircuitBreaker) StateName() string {
	switch b.statState.Load() {
	case cbOpen:
		return "open"
	case cbHalfOpen:
		return "half_open"
	default:
		return "closed"
	}
}

// Metrics returns counters for the /metrics endpoint.
func (b *CircuitBreaker) Metrics() (total, fails, rejects int64) {
	return b.statTotal.Load(), b.statFails.Load(), b.statReject.Load()
}

// Allow reports whether a request may proceed. Call Success/Failure after.
func (b *CircuitBreaker) Allow() bool {
	b.mu.Lock()
	defer b.mu.Unlock()

	switch atomic.LoadInt32(&b.state) {
	case cbOpen:
		if time.Since(b.openedAt) >= b.cfg.OpenTimeout {
			// Transition to half-open; first probe passes.
			b.setState(cbHalfOpen)
			b.probes = 1
			return true
		}
		b.statReject.Add(1)
		return false
	case cbHalfOpen:
		if b.probes < b.cfg.HalfOpenMax {
			b.probes++
			return true
		}
		b.statReject.Add(1)
		return false
	default: // closed
		b.rollWindowLocked()
		return true
	}
}

// Success records a successful request.
func (b *CircuitBreaker) Success() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.statTotal.Add(1)

	switch atomic.LoadInt32(&b.state) {
	case cbHalfOpen:
		b.probeOK++
		if b.probeOK >= b.cfg.HalfOpenMax {
			b.closeLocked()
		}
	case cbOpen:
		// ignore success while open (shouldn't happen; probes pass via Allow)
	default:
		b.rollWindowLocked()
		b.total++
	}
}

// Failure records a failed request (5xx or downstream error).
func (b *CircuitBreaker) Failure() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.statTotal.Add(1)
	b.statFails.Add(1)

	switch atomic.LoadInt32(&b.state) {
	case cbHalfOpen:
		b.probeBad++
		// Any probe failure → back to open, full cooldown.
		b.setState(cbOpen)
		b.openedAt = time.Now()
	case cbOpen:
		// already open
	default:
		b.rollWindowLocked()
		b.total++
		b.fails++
		if b.total >= b.cfg.MinRequests {
			rate := float64(b.fails) / float64(b.total)
			if rate >= b.cfg.FailureRate {
				b.setState(cbOpen)
				b.openedAt = time.Now()
			}
		}
	}
}

// Reset forces the breaker back to closed (admin/debug endpoint).
func (b *CircuitBreaker) Reset() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.closeLocked()
}

// rollWindowLocked resets the window counters when the window has elapsed.
// Caller must hold b.mu.
func (b *CircuitBreaker) rollWindowLocked() {
	if time.Since(b.winStart) >= b.cfg.Window {
		b.winStart = time.Now()
		b.total = 0
		b.fails = 0
	}
}

func (b *CircuitBreaker) closeLocked() {
	b.setState(cbClosed)
	b.winStart = time.Now()
	b.total = 0
	b.fails = 0
	b.probes = 0
	b.probeOK = 0
	b.probeBad = 0
}

func (b *CircuitBreaker) setState(s int32) {
	atomic.StoreInt32(&b.state, s)
	b.statState.Store(s)
}

// shouldSkip checks whether a path bypasses the breaker.
func (b *CircuitBreaker) shouldSkip(path string) bool {
	for _, pfx := range b.cfg.SkipPrefixes {
		if len(path) >= len(pfx) && path[:len(pfx)] == pfx {
			return true
		}
	}
	return false
}

// CircuitBreakerMiddleware wraps a named breaker as a Gin middleware.
// 5xx responses count as failures; 429/4xx count as success (client errors).
func CircuitBreakerMiddleware(br *CircuitBreaker) gin.HandlerFunc {
	return func(c *gin.Context) {
		if br == nil || !br.cfg.Enabled {
			c.Next()
			return
		}
		path := c.Request.URL.Path
		if br.shouldSkip(path) {
			c.Next()
			return
		}

		if !br.Allow() {
			c.Header("X-Circuit-Breaker", "open")
			resp.Err(c, http.StatusServiceUnavailable, errcode.CodeServiceUnavailable)
			c.Abort()
			return
		}

		c.Next()

		status := c.Writer.Status()
		if status >= 500 {
			br.Failure()
		} else {
			br.Success()
		}
	}
}
