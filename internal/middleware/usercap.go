package middleware

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/redis/go-redis/v9"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/model"
	"minibili/internal/pkg/usercap"
)

const (
	usercapCacheTTL    = 30 * time.Second
	usercapCachePrefix = "usercap:"
)

// UserCapabilityRestrict enforces capability-level restrictions. When a request
// consumes a capability the user is actively restricted from, it is denied with
// CodeCapabilityRestricted. Disabled unless enabled=true (灰度开关 USERCAP_ENABLED).
func UserCapabilityRestrict(db *gorm.DB, rdb *redis.Client, enabled bool) gin.HandlerFunc {
	if !enabled || db == nil {
		return func(c *gin.Context) { c.Next() }
	}
	return func(c *gin.Context) {
		capName, ok := usercap.CapabilityOf(c.Request.Method, c.Request.URL.Path)
		if !ok {
			c.Next()
			return
		}
		uid, ok := UserID(c)
		if !ok {
			c.Next()
			return
		}
		if !restrictedCaps(c.Request.Context(), db, rdb, uid)[capName] {
			c.Next()
			return
		}
		c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
			"code": errcode.CodeCapabilityRestricted,
			"msg":  errcode.GetMsg(errcode.CodeCapabilityRestricted),
			"data": gin.H{
				"capability": capName,
				"label":      usercap.Label(capName),
				"reason":     capabilityReason(db, uid, capName),
			},
		})
	}
}

// restrictedCaps returns the set of capability names currently active-restricted
// for the user, backed by a short-TTL Redis cache to avoid a per-request DB hit.
func restrictedCaps(ctx context.Context, db *gorm.DB, rdb *redis.Client, uid uint64) map[string]bool {
	key := fmt.Sprintf("%s%d", usercapCachePrefix, uid)
	if rdb != nil {
		if b, err := rdb.Get(ctx, key).Bytes(); err == nil {
			var arr []string
			if json.Unmarshal(b, &arr) == nil {
				return toSet(arr)
			}
		}
	}

	var rows []model.UserCapabilityRestriction
	_ = db.WithContext(ctx).
		Where("user_id = ? AND status = 'active' AND (expires_at IS NULL OR expires_at > ?)", uid, time.Now()).
		Find(&rows).Error
	arr := make([]string, 0, len(rows))
	for _, r := range rows {
		arr = append(arr, r.Capability)
	}

	if rdb != nil {
		if b, err := json.Marshal(arr); err == nil {
			rdb.Set(ctx, key, b, usercapCacheTTL)
		}
	}
	return toSet(arr)
}

func toSet(arr []string) map[string]bool {
	m := make(map[string]bool, len(arr))
	for _, s := range arr {
		m[s] = true
	}
	return m
}

func capabilityReason(db *gorm.DB, uid uint64, capability string) string {
	var r model.UserCapabilityRestriction
	if err := db.Where("user_id = ? AND capability = ? AND status = 'active'", uid, capability).First(&r).Error; err == nil {
		return r.Reason
	}
	return ""
}

// InvalidateUserCapCache clears the user's capability cache after an admin change.
func InvalidateUserCapCache(rdb *redis.Client, uid uint64) {
	if rdb == nil {
		return
	}
	rdb.Del(context.Background(), fmt.Sprintf("%s%d", usercapCachePrefix, uid))
}
