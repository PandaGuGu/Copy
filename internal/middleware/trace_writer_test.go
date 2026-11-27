package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/glebarez/sqlite"
	"gorm.io/gorm"

	"minibili/internal/model"
)

func newTraceTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	if err != nil {
		t.Fatalf("open sqlite: %v", err)
	}
	sqlDB, err := db.DB()
	if err != nil {
		t.Fatalf("db handle: %v", err)
	}
	// :memory: is per-connection; keep a single connection so migrations and
	// queries hit the same database.
	sqlDB.SetMaxOpenConns(1)
	if err := db.AutoMigrate(&model.TraceRecord{}); err != nil {
		t.Fatalf("migrate: %v", err)
	}
	return db
}

// Probe and metrics endpoints must not generate trace rows: they run at high
// frequency and their only job is to be cheap.
func TestTraceWriter_SkipsHealthAndMetrics(t *testing.T) {
	gin.SetMode(gin.TestMode)
	db := newTraceTestDB(t)

	w := NewTraceWriter(db, TraceWriterOptions{
		QueueSize:     16,
		Workers:       1,
		BatchSize:     4,
		FlushInterval: 10 * time.Millisecond,
		SkipPaths:     []string{"/api/v1/health", "/api/v1/metrics"},
	})
	defer w.Close()

	r := gin.New()
	r.Use(w.Middleware())
	r.GET("/api/v1/health", func(c *gin.Context) { c.Status(http.StatusOK) })
	r.GET("/api/v1/metrics", func(c *gin.Context) { c.Status(http.StatusOK) })
	r.GET("/api/v1/videos", func(c *gin.Context) { c.Status(http.StatusOK) })

	for _, p := range []string{"/api/v1/health", "/api/v1/metrics"} {
		rec := httptest.NewRecorder()
		r.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, p, nil))
		if rec.Code != http.StatusOK {
			t.Fatalf("%s: status = %d", p, rec.Code)
		}
		if got := rec.Header().Get("X-Trace-ID"); got != "" {
			t.Fatalf("%s: skipped path should not set X-Trace-ID, got %q", p, got)
		}
	}

	rec := httptest.NewRecorder()
	r.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/api/v1/videos", nil))
	if rec.Header().Get("X-Trace-ID") == "" {
		t.Fatal("traced path should set X-Trace-ID")
	}

	w.Close()

	var n int64
	if err := db.Model(&model.TraceRecord{}).Count(&n).Error; err != nil {
		t.Fatalf("count: %v", err)
	}
	if n != 1 {
		t.Fatalf("expected exactly 1 trace row (videos only), got %d", n)
	}
	enq, dropped, written, failed := w.Stats()
	if enq != 1 || dropped != 0 || written != 1 || failed != 0 {
		t.Fatalf("stats = enq:%d drop:%d written:%d failed:%d", enq, dropped, written, failed)
	}
}

// Every enqueued record must land in the database after Close, across multiple
// batch boundaries.
func TestTraceWriter_FlushesAllBatchesOnClose(t *testing.T) {
	gin.SetMode(gin.TestMode)
	db := newTraceTestDB(t)

	w := NewTraceWriter(db, TraceWriterOptions{
		QueueSize:     128,
		Workers:       2,
		BatchSize:     5,
		FlushInterval: 50 * time.Millisecond,
		SkipPaths:     nil,
	})

	r := gin.New()
	r.Use(w.Middleware())
	r.GET("/api/v1/ping", func(c *gin.Context) { c.Status(http.StatusOK) })

	const requests = 37
	for i := 0; i < requests; i++ {
		rec := httptest.NewRecorder()
		r.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/api/v1/ping", nil))
	}

	w.Close()

	var n int64
	if err := db.Model(&model.TraceRecord{}).Count(&n).Error; err != nil {
		t.Fatalf("count: %v", err)
	}
	if n != requests {
		t.Fatalf("expected %d rows after Close, got %d", requests, n)
	}
}

// A full buffer must drop rather than block the request path.
func TestTraceWriter_DropsWhenQueueFull(t *testing.T) {
	db := newTraceTestDB(t)

	// No workers: nothing drains the buffer, so the second enqueue overflows.
	w := newTraceWriter(db, TraceWriterOptions{QueueSize: 1}, false)

	if !w.Enqueue(model.TraceRecord{TraceID: "a"}) {
		t.Fatal("first enqueue should succeed")
	}
	if w.Enqueue(model.TraceRecord{TraceID: "b"}) {
		t.Fatal("second enqueue should be dropped when queue is full")
	}
	if _, dropped, _, _ := w.Stats(); dropped != 1 {
		t.Fatalf("dropped = %d, want 1", dropped)
	}
}

func TestTraceWriter_NilSafe(t *testing.T) {
	var w *TraceWriter
	if w.Enqueue(model.TraceRecord{}) {
		t.Fatal("nil writer must not accept records")
	}
	w.Close()
	if _, _, _, _ = w.Stats(); false {
		t.Fatal("unreachable")
	}
}
