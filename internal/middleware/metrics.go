package middleware

// Metrics — zero-dependency Prometheus text exposition.
//
// Exposes a lightweight HTTP metrics endpoint in the standard Prometheus
// text format (https://prometheus.io/docs/instrumenting/exposition_formats/),
// so it can be scraped by Prometheus / Grafana / UptimeRobot out of the box.
//
// Emitted series:
//
//	minibili_http_requests_total{method,path,status}      — request counter
//	minibili_http_request_duration_seconds_bucket{le}      — latency histogram
//	minibili_http_request_duration_seconds_sum             — latency sum
//	minibili_http_request_duration_seconds_count           — latency count
//	minibili_rate_limit_rejected_total{path}               — 429 rejections
//	minibili_circuit_breaker_state{name}                   — 0 closed / 1 open / 2 half_open
//	minibili_circuit_breaker_rejected_total{name}          — fast-fail rejections
//	minibili_circuit_breaker_failures_total{name}          — counted failures
//	minibili_circuit_breaker_requests_total{name}          — counted requests
//	minibili_uptime_seconds                                — process uptime
//
// Duration buckets follow the standard Prometheus defaults.

import (
	"fmt"
	"net/http"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
)

// Prometheus duration buckets (seconds).
var promBuckets = []float64{0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10}

// MetricsRegistry accumulates counters and histograms.
type MetricsRegistry struct {
	mu sync.Mutex

	// requests: key = method + "|" + path + "|" + status
	requests map[string]int64
	// durations: key = method + "|" + path ; value = []bucketHits
	durations map[string][]int64
	durSum    map[string]float64
	durCount  map[string]int64

	rateLimitRejected map[string]int64

	// Breaker gauges are read live from the breaker instances.
	breakers map[string]*CircuitBreaker

	started time.Time
}

// NewMetricsRegistry creates an empty registry and starts the clock.
func NewMetricsRegistry() *MetricsRegistry {
	return &MetricsRegistry{
		requests:          make(map[string]int64),
		durations:         make(map[string][]int64),
		durSum:            make(map[string]float64),
		durCount:          make(map[string]int64),
		rateLimitRejected: make(map[string]int64),
		breakers:          make(map[string]*CircuitBreaker),
		started:           time.Now(),
	}
}

// RegisterBreaker attaches a breaker so its state is exported.
func (m *MetricsRegistry) RegisterBreaker(name string, br *CircuitBreaker) {
	if br == nil {
		return
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	m.breakers[name] = br
}

// RecordRequest counts one request with its duration and status.
func (m *MetricsRegistry) RecordRequest(method, path string, status int, dur time.Duration) {
	sk := method + "|" + path + "|" + strconv.Itoa(status)
	dk := method + "|" + path
	sec := dur.Seconds()

	m.mu.Lock()
	defer m.mu.Unlock()

	m.requests[sk]++

	hits, ok := m.durations[dk]
	if !ok {
		hits = make([]int64, len(promBuckets))
		m.durations[dk] = hits
	}
	for i, b := range promBuckets {
		if sec <= b {
			hits[i]++
		}
	}
	m.durSum[dk] += sec
	m.durCount[dk]++
}

// RecordRateLimitRejected counts a 429 rejection.
func (m *MetricsRegistry) RecordRateLimitRejected(path string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.rateLimitRejected[path]++
}

// Render writes the full Prometheus text exposition.
func (m *MetricsRegistry) Render(w http.ResponseWriter) {
	m.mu.Lock()
	defer m.mu.Unlock()

	var b strings.Builder
	b.WriteString("# HELP minibili_http_requests_total HTTP requests by method, path and status.\n")
	b.WriteString("# TYPE minibili_http_requests_total counter\n")

	keys := make([]string, 0, len(m.requests))
	for k := range m.requests {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	for _, k := range keys {
		parts := strings.Split(k, "|")
		fmt.Fprintf(&b, "minibili_http_requests_total{method=%q,path=%q,status=%q} %d\n",
			parts[0], parts[1], parts[2], m.requests[k])
	}

	b.WriteString("# HELP minibili_http_request_duration_seconds HTTP request latency histogram.\n")
	b.WriteString("# TYPE minibili_http_request_duration_seconds histogram\n")
	dkeys := make([]string, 0, len(m.durations))
	for k := range m.durations {
		dkeys = append(dkeys, k)
	}
	sort.Strings(dkeys)
	for _, k := range dkeys {
		parts := strings.Split(k, "|")
		hits := m.durations[k]
		for i, h := range hits {
			le := strconv.FormatFloat(promBuckets[i], 'f', -1, 64)
			fmt.Fprintf(&b, "minibili_http_request_duration_seconds_bucket{method=%q,path=%q,le=%q} %d\n",
				parts[0], parts[1], le, h)
		}
		fmt.Fprintf(&b, "minibili_http_request_duration_seconds_bucket{method=%q,path=%q,le=\"+Inf\"} %d\n",
			parts[0], parts[1], m.durCount[k])
		fmt.Fprintf(&b, "minibili_http_request_duration_seconds_sum{method=%q,path=%q} %g\n",
			parts[0], parts[1], m.durSum[k])
		fmt.Fprintf(&b, "minibili_http_request_duration_seconds_count{method=%q,path=%q} %d\n",
			parts[0], parts[1], m.durCount[k])
	}

	b.WriteString("# HELP minibili_rate_limit_rejected_total Requests rejected by the rate limiter.\n")
	b.WriteString("# TYPE minibili_rate_limit_rejected_total counter\n")
	rl := make([]string, 0, len(m.rateLimitRejected))
	for k := range m.rateLimitRejected {
		rl = append(rl, k)
	}
	sort.Strings(rl)
	for _, k := range rl {
		fmt.Fprintf(&b, "minibili_rate_limit_rejected_total{path=%q} %d\n", k, m.rateLimitRejected[k])
	}

	b.WriteString("# HELP minibili_circuit_breaker_state Circuit breaker state: 0 closed, 1 open, 2 half_open.\n")
	b.WriteString("# TYPE minibili_circuit_breaker_state gauge\n")
	bn := make([]string, 0, len(m.breakers))
	for k := range m.breakers {
		bn = append(bn, k)
	}
	sort.Strings(bn)
	for _, k := range bn {
		br := m.breakers[k]
		total, fails, rejects := br.Metrics()
		state := br.StateName()
		stateVal := 0
		if state == "open" {
			stateVal = 1
		} else if state == "half_open" {
			stateVal = 2
		}
		fmt.Fprintf(&b, "minibili_circuit_breaker_state{name=%q} %d\n", k, stateVal)
		fmt.Fprintf(&b, "# HELP minibili_circuit_breaker_requests_total Requests seen by breaker %s.\n", k)
		fmt.Fprintf(&b, "# TYPE minibili_circuit_breaker_requests_total counter\n")
		fmt.Fprintf(&b, "minibili_circuit_breaker_requests_total{name=%q} %d\n", k, total)
		fmt.Fprintf(&b, "# HELP minibili_circuit_breaker_failures_total Failures seen by breaker %s.\n", k)
		fmt.Fprintf(&b, "# TYPE minibili_circuit_breaker_failures_total counter\n")
		fmt.Fprintf(&b, "minibili_circuit_breaker_failures_total{name=%q} %d\n", k, fails)
		fmt.Fprintf(&b, "# HELP minibili_circuit_breaker_rejected_total Fast-fail rejections by breaker %s.\n", k)
		fmt.Fprintf(&b, "# TYPE minibili_circuit_breaker_rejected_total counter\n")
		fmt.Fprintf(&b, "minibili_circuit_breaker_rejected_total{name=%q} %d\n", k, rejects)
	}

	fmt.Fprintf(&b, "# HELP minibili_uptime_seconds Process uptime.\n")
	fmt.Fprintf(&b, "# TYPE minibili_uptime_seconds gauge\n")
	fmt.Fprintf(&b, "minibili_uptime_seconds %d\n", int64(time.Since(m.started).Seconds()))

	w.Header().Set("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
	_, _ = w.Write([]byte(b.String()))
}

// MetricsMiddleware records request metrics into the registry.
func MetricsMiddleware(reg *MetricsRegistry) gin.HandlerFunc {
	return func(c *gin.Context) {
		if reg == nil {
			c.Next()
			return
		}
		start := time.Now()
		c.Next()
		status := c.Writer.Status()
		reg.RecordRequest(c.Request.Method, c.Request.URL.Path, status, time.Since(start))
	}
}

// MetricsHandler serves the Prometheus text exposition at /api/v1/metrics.
func MetricsHandler(reg *MetricsRegistry) gin.HandlerFunc {
	return func(c *gin.Context) {
		if reg == nil {
			c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": "metrics disabled"})
			return
		}
		reg.Render(c.Writer)
	}
}
