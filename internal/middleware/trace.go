package middleware

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"os"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/logger"
	"minibili/internal/model"
)

// NewTraceID generates a random 16-byte hex trace id.
func NewTraceID() string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	return hex.EncodeToString(b)
}

// NewRequestID generates a random 8-byte hex request id.
func NewRequestID() string {
	b := make([]byte, 8)
	_, _ = rand.Read(b)
	return hex.EncodeToString(b)
}

// TraceWriterOptions configures the async trace-record writer.
type TraceWriterOptions struct {
	// QueueSize is the record buffer capacity. When full, new records are
	// dropped rather than blocking the request path.
	QueueSize int
	// Workers is the number of concurrent batch-flush goroutines.
	Workers int
	// BatchSize is the maximum rows per INSERT.
	BatchSize int
	// FlushInterval bounds how long a record may sit in the buffer.
	FlushInterval time.Duration
	// SkipPaths are exact request paths that are not traced (probes, metrics).
	SkipPaths []string
	// SkipPrefixes are path prefixes that are not traced (static assets).
	SkipPrefixes []string
}

// DefaultTraceWriterOptions returns production-safe defaults.
//
// The previous implementation forked one goroutine per request and issued a
// single-row INSERT each time. Under 50 concurrent requests that produced ~50
// concurrent INSERTs competing for a pool whose idle size was Go's default of
// 2, which is why /api/v1/health showed p50 41ms but p99 9.49s. Batching plus
// a bounded queue removes that amplification entirely.
func DefaultTraceWriterOptions() TraceWriterOptions {
	skip := []string{
		"/api/v1/health",
		"/api/v1/metrics",
		"/favicon.ico",
		"/robots.txt",
	}
	// Ops escape hatch: TRACE_SKIP_PATHS="/a,/b" appends exact paths.
	for _, p := range strings.Split(os.Getenv("TRACE_SKIP_PATHS"), ",") {
		if p = strings.TrimSpace(p); p != "" {
			skip = append(skip, p)
		}
	}
	return TraceWriterOptions{
		QueueSize:     8192,
		Workers:       2,
		BatchSize:     200,
		FlushInterval: 200 * time.Millisecond,
		SkipPaths:     skip,
		SkipPrefixes:  []string{"/assets/", "/static/"},
	}
}

// TraceWriter buffers TraceRecords and flushes them in batches off the request
// path. It is safe for concurrent use and never blocks a request: when the
// buffer is full, records are dropped and counted.
type TraceWriter struct {
	db            *gorm.DB
	ch            chan model.TraceRecord
	done          chan struct{}
	wg            sync.WaitGroup
	closeOnce     sync.Once
	batchSize     int
	flushInterval time.Duration
	skipPaths     map[string]struct{}
	skipPrefixes  []string

	enqueued atomic.Uint64
	dropped  atomic.Uint64
	written  atomic.Uint64
	failed   atomic.Uint64
}

// NewTraceWriter starts a writer with the given options.
func NewTraceWriter(db *gorm.DB, opts TraceWriterOptions) *TraceWriter {
	return newTraceWriter(db, opts, true)
}

// newTraceWriter builds a writer; startWorkers=false is for white-box tests
// that exercise the drop path deterministically.
func newTraceWriter(db *gorm.DB, opts TraceWriterOptions, startWorkers bool) *TraceWriter {
	if opts.QueueSize <= 0 {
		opts.QueueSize = 8192
	}
	if opts.Workers <= 0 {
		opts.Workers = 1
	}
	if opts.BatchSize <= 0 {
		opts.BatchSize = 100
	}
	if opts.FlushInterval <= 0 {
		opts.FlushInterval = 200 * time.Millisecond
	}
	skip := make(map[string]struct{}, len(opts.SkipPaths))
	for _, p := range opts.SkipPaths {
		if p = strings.TrimSpace(p); p != "" {
			skip[p] = struct{}{}
		}
	}
	w := &TraceWriter{
		db:            db,
		ch:            make(chan model.TraceRecord, opts.QueueSize),
		done:          make(chan struct{}),
		batchSize:     opts.BatchSize,
		flushInterval: opts.FlushInterval,
		skipPaths:     skip,
		skipPrefixes:  opts.SkipPrefixes,
	}
	if startWorkers {
		for i := 0; i < opts.Workers; i++ {
			w.wg.Add(1)
			go w.worker()
		}
	}
	return w
}

// Enqueue adds a record without blocking. Returns false if it was dropped.
func (w *TraceWriter) Enqueue(rec model.TraceRecord) bool {
	if w == nil {
		return false
	}
	select {
	case w.ch <- rec:
		w.enqueued.Add(1)
		return true
	default:
		n := w.dropped.Add(1)
		if n%1000 == 1 && logger.L != nil {
			logger.L.Warn("trace queue full, dropping records",
				zap.Uint64("dropped_total", n))
		}
		return false
	}
}

// Middleware records every HTTP request into the buffer.
func (w *TraceWriter) Middleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		if w == nil || w.shouldSkip(c.Request.URL.Path) {
			c.Next()
			return
		}
		start := time.Now()

		traceID := c.GetHeader("X-Trace-ID")
		if traceID == "" {
			traceID = NewTraceID()
		}
		requestID := c.GetHeader("X-Request-ID")
		if requestID == "" {
			requestID = NewRequestID()
		}

		c.Set("trace_id", traceID)
		c.Set("request_id", requestID)
		c.Writer.Header().Set("X-Trace-ID", traceID)
		c.Writer.Header().Set("X-Request-ID", requestID)

		c.Next()

		rec := model.TraceRecord{
			TraceID:    traceID,
			RequestID:  requestID,
			Path:       c.Request.URL.Path,
			Method:     c.Request.Method,
			Status:     c.Writer.Status(),
			DurationMs: time.Since(start).Milliseconds(),
			CreatedAt:  time.Now(),
		}
		if uid, exists := c.Get(CtxUserIDKey); exists {
			if v, ok := uid.(uint64); ok {
				rec.UserID = &v
			}
		}
		if last := c.Errors.Last(); last != nil {
			rec.ErrorMsg = truncateStr(last.Err.Error(), 1000)
		}
		w.Enqueue(rec)
	}
}

func (w *TraceWriter) shouldSkip(path string) bool {
	if _, ok := w.skipPaths[path]; ok {
		return true
	}
	for _, p := range w.skipPrefixes {
		if p != "" && strings.HasPrefix(path, p) {
			return true
		}
	}
	return false
}

func (w *TraceWriter) worker() {
	defer w.wg.Done()
	batch := make([]model.TraceRecord, 0, w.batchSize)
	ticker := time.NewTicker(w.flushInterval)
	defer ticker.Stop()

	for {
		select {
		case rec := <-w.ch:
			batch = append(batch, rec)
			if len(batch) >= w.batchSize {
				w.flush(batch)
				batch = batch[:0]
			}
		case <-ticker.C:
			if len(batch) > 0 {
				w.flush(batch)
				batch = batch[:0]
			}
		case <-w.done:
			// Drain whatever is still buffered, then exit.
			for {
				select {
				case rec := <-w.ch:
					batch = append(batch, rec)
					if len(batch) >= w.batchSize {
						w.flush(batch)
						batch = batch[:0]
					}
				default:
					if len(batch) > 0 {
						w.flush(batch)
					}
					return
				}
			}
		}
	}
}

func (w *TraceWriter) flush(batch []model.TraceRecord) {
	if len(batch) == 0 {
		return
	}
	if err := w.db.CreateInBatches(batch, len(batch)).Error; err != nil {
		w.failed.Add(uint64(len(batch)))
		if logger.L != nil {
			logger.L.Warn("trace batch insert failed",
				zap.Int("rows", len(batch)),
				zap.Error(err),
			)
		}
		return
	}
	w.written.Add(uint64(len(batch)))
}

// Close stops accepting new records and flushes what is buffered.
// It is idempotent and safe to call from a shutdown path.
func (w *TraceWriter) Close() {
	if w == nil {
		return
	}
	w.closeOnce.Do(func() {
		close(w.done)
		w.wg.Wait()
	})
}

// Stats reports writer counters for observability.
func (w *TraceWriter) Stats() (enqueued, dropped, written, failed uint64) {
	if w == nil {
		return 0, 0, 0, 0
	}
	return w.enqueued.Load(), w.dropped.Load(), w.written.Load(), w.failed.Load()
}

func truncateStr(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen]
}

// UUIDv4 generates a random UUID v4 string (RFC 4122).
func UUIDv4() string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	b[6] = (b[6] & 0x0f) | 0x40
	b[8] = (b[8] & 0x3f) | 0x80
	return fmt.Sprintf("%08x-%04x-%04x-%04x-%012x",
		b[0:4], b[4:6], b[6:8], b[8:10], b[10:16])
}
