package middleware

import (
	"fmt"
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/redis/go-redis/v9"

	"minibili/internal/errcode"
	"minibili/internal/pkg/resp"
)

// RateLimitConfig holds per-tier rate limit parameters.
type RateLimitConfig struct {
	Enabled        bool
	GuestWindow    time.Duration // sliding window
	GuestMax       int           // max requests per window
	UserWindow     time.Duration
	UserMax        int
	AdminWindow    time.Duration
	AdminMax       int
	SkipPaths      map[string]bool // bypass exact paths
	SkipPrefixes   []string       // bypass path prefixes
}

// DefaultRateLimitConfig returns sensible defaults.
func DefaultRateLimitConfig() RateLimitConfig {
	return RateLimitConfig{
		Enabled:      true,
		GuestWindow:  60 * time.Second,
		GuestMax:     60, // 1 req/s averaged
		UserWindow:   60 * time.Second,
		UserMax:      300, // 5 req/s averaged
		AdminWindow:  60 * time.Second,
		AdminMax:     1000,
		SkipPaths: map[string]bool{
			"/api/v1/health": true,
		},
		SkipPrefixes: []string{
			"/api/v1/ws/",       // WebSocket upgrade
			"/live-hls/",        // HLS streaming
			"/uploads/",         // static files
		},
	}
}

// RateLimiter returns a Gin middleware that enforces per-tier sliding-window
// rate limits using Redis INCR + EXPIRE.
//
// Tiers:
//   - guest:   identified by client IP
//   - user:    identified by user_id from JWT context
//   - admin:   identified by admin_id from JWT context
//
// Each tier has its own window and max. Exceeded requests receive 429.
func RateLimiter(rdb *redis.Client, cfg RateLimitConfig) gin.HandlerFunc {
	if !cfg.Enabled {
		return func(c *gin.Context) { c.Next() }
	}

	return func(c *gin.Context) {
		path := c.Request.URL.Path

		// Skip known paths.
		if cfg.SkipPaths[path] {
			c.Next()
			return
		}
		for _, pfx := range cfg.SkipPrefixes {
			if len(path) >= len(pfx) && path[:len(pfx)] == pfx {
				c.Next()
				return
			}
		}

		// Determine tier and identity.
		var (
			window time.Duration
			max    int
			id     string
		)

		// Try extracting user/admin ID from context (set by auth middlewares).
		if uid, ok := c.Get(CtxUserIDKey); ok {
			// Check if this is an admin route.
			if adminUID, ok2 := c.Get("admin_id"); ok2 {
				window, max = cfg.AdminWindow, cfg.AdminMax
				id = fmt.Sprintf("admin:%d", adminUID)
				_ = uid // user_id may also be set
			} else {
				window, max = cfg.UserWindow, cfg.UserMax
				id = fmt.Sprintf("user:%d", uid)
			}
		} else {
			// Guest — use client IP.
			window, max = cfg.GuestWindow, cfg.GuestMax
			id = "ip:" + c.ClientIP()
		}

		// Sliding-window counter: ratelimit:{id}:{window_sec}
		windowSec := int64(window.Seconds())
		if windowSec < 1 {
			windowSec = 1
		}
		key := fmt.Sprintf("ratelimit:%s:%d", id, windowSec)

		ctx := c.Request.Context()
		count, err := rdb.Incr(ctx, key).Result()
		if err != nil {
			// Redis down — fail open to avoid blocking all traffic.
			c.Next()
			return
		}

		if count == 1 {
			rdb.Expire(ctx, key, window)
		}

		remaining := max - int(count)
		if remaining < 0 {
			remaining = 0
		}

		// Always set rate limit headers.
		c.Header("X-RateLimit-Limit", strconv.Itoa(max))
		c.Header("X-RateLimit-Remaining", strconv.Itoa(remaining))
		ttl, _ := rdb.TTL(ctx, key).Result()
		c.Header("X-RateLimit-Reset", strconv.FormatInt(time.Now().Unix()+int64(ttl.Seconds()), 10))

		if int(count) > max {
			c.Header("Retry-After", strconv.Itoa(int(ttl.Seconds())))
			resp.Err(c, http.StatusTooManyRequests, errcode.CodeRateLimited)
			c.Abort()
			return
		}

		c.Next()
	}
}
