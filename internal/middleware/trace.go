package middleware

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
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

// TraceRecordMiddleware records every HTTP request as a TraceRecord.
// Must be registered AFTER auth middleware so user_id is available.
func TraceRecordMiddleware(db *gorm.DB) gin.HandlerFunc {
	return func(c *gin.Context) {
		start := time.Now()

		// Generate or read trace / request id
		traceID := c.GetHeader("X-Trace-ID")
		if traceID == "" {
			traceID = NewTraceID()
		}
		requestID := c.GetHeader("X-Request-ID")
		if requestID == "" {
			requestID = NewRequestID()
		}

		// Store in context for downstream use
		c.Set("trace_id", traceID)
		c.Set("request_id", requestID)
		c.Writer.Header().Set("X-Trace-ID", traceID)
		c.Writer.Header().Set("X-Request-ID", requestID)

		// Process request
		c.Next()

		// Record after response
		duration := time.Since(start).Milliseconds()
		rec := model.TraceRecord{
			TraceID:    traceID,
			RequestID:  requestID,
			Path:       c.Request.URL.Path,
			Method:     c.Request.Method,
			Status:     c.Writer.Status(),
			DurationMs: duration,
			CreatedAt:  time.Now(),
		}

		// Attach user_id if available
		if uid, exists := c.Get(CtxUserIDKey); exists {
			if v, ok := uid.(uint64); ok {
				rec.UserID = &v
			}
		}

		// Attach error message from context if any
		if errs := c.Errors.Last(); errs != nil {
			rec.ErrorMsg = truncateStr(errs.Err.Error(), 1000)
		}

		// Write asynchronously to not block the response
		go func(r model.TraceRecord) {
			if err := db.Create(&r).Error; err != nil {
				logger.L.Warn("trace record create failed",
					zap.Error(err),
					zap.String("trace_id", r.TraceID),
					zap.String("path", r.Path),
				)
			}
		}(rec)
	}
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
