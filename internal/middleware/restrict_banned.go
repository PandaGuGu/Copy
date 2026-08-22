package middleware

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/model"
)

// bannedAllowedPath reports whether a banned (restricted) user may call this
// request. Restricted users can only reach the appeal flow and read their own
// account status — the "被限制权力的用户可恢复部分/全部权力" limited-login path.
func bannedAllowed(r *http.Request) bool {
	p := r.URL.Path
	// Any appeal endpoint (submit / list-mine / detail).
	if strings.HasPrefix(p, "/api/v1/appeals") {
		return true
	}
	// Read own account status (shows ban reason/expiry for the restricted screen).
	if r.Method == http.MethodGet && p == "/api/v1/users/me" {
		return true
	}
	return false
}

// RestrictBanned confines banned users to a restricted session: they may log
// in but every authenticated API call outside the allow-list is denied. Applied
// on the authenticated /api/v1 group after JWTAuth.
func RestrictBanned(db *gorm.DB) gin.HandlerFunc {
	return func(c *gin.Context) {
		uid, ok := UserID(c)
		if !ok {
			c.Next()
			return
		}
		var u model.User
		if err := db.Select("status").Where("id = ?", uid).First(&u).Error; err != nil || u.Status != "banned" {
			c.Next()
			return
		}
		if !bannedAllowed(c.Request) {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
				"code": errcode.CodeAccountBanned,
				"msg":  "账号已被封禁，仅可进行申诉或查看封禁状态",
			})
			return
		}
		c.Next()
	}
}
