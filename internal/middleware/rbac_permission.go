package middleware

import (
	"fmt"
	"net/http"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

// RequirePermission returns a middleware that checks if the current admin
// has the specified permission via their assigned role.
func RequirePermission(db *gorm.DB, resource, action string) gin.HandlerFunc {
	return func(c *gin.Context) {
		adminID, ok := AdminID(c)
		if !ok {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"code": 40100, "msg": "未登录"})
			return
		}

		// Check direct assignment: admin → role → role_permissions → permission
		var count int64
		db.WithContext(c.Request.Context()).
			Raw(`SELECT COUNT(*) FROM admin_role_assignments
			JOIN role_permissions ON role_permissions.role_id = admin_role_assignments.role_id
			JOIN admin_permissions ON admin_permissions.id = role_permissions.permission_id
			WHERE admin_role_assignments.admin_id = ? AND admin_permissions.resource = ? AND admin_permissions.action LIKE ?`,
				adminID, resource, "%"+action+"%").Scan(&count)

		// Debug: log permission check result
		c.Header("X-RBAC-Check", "admin:"+fmt.Sprint(adminID)+" resource:"+resource+" action:"+action+" count:"+fmt.Sprint(count))

		if count == 0 {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"code": 40300, "msg": "无操作权限: " + resource + "." + action})
			return
		}
		c.Next()
	}
}
