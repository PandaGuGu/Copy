package handler

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	"github.com/gin-gonic/gin"
	"github.com/glebarez/sqlite"
	"github.com/stretchr/testify/require"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/config"
	"minibili/internal/data"
	"minibili/internal/middleware"
	"minibili/internal/model"
)

// loadUserCtx sets user/admin id from headers for capability tests.
func loadUserCtx() gin.HandlerFunc {
	return func(c *gin.Context) {
		if v := c.GetHeader("X-Test-Uid"); v != "" {
			if n, _ := strconv.Atoi(v); n > 0 {
				c.Set(middleware.CtxUserIDKey, uint64(n))
			}
		}
		if v := c.GetHeader("X-Test-Aid"); v != "" {
			if n, _ := strconv.Atoi(v); n > 0 {
				c.Set(middleware.CtxAdminIDKey, uint64(n))
			}
		}
		c.Next()
	}
}

func dummyOK(c *gin.Context) { c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok"}) }

func newUsercapTestEnv(t *testing.T) (api *API, r *gin.Engine) {
	t.Helper()
	gin.SetMode(gin.TestMode)

	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	require.NoError(t, err)
	sqlDB, err := db.DB()
	require.NoError(t, err)
	sqlDB.SetMaxOpenConns(1)
	require.NoError(t, data.AutoMigrateAll(db, zap.NewNop()))

	mr, err := miniredis.Run()
	require.NoError(t, err)
	t.Cleanup(mr.Close)

	cfg := &config.C{
		RedisAddr: mr.Addr(), UserCapEnabled: true,
		RedisDial: 5 * time.Second, RedisRead: 3 * time.Second,
		RedisWrite: 3 * time.Second, RedisPoolSize: 10,
	}
	rdb, err := data.NewRedis(cfg)
	require.NoError(t, err)

	api = &API{Dependencies: &Dependencies{
		DB: db, Redis: rdb, Log: zap.NewNop(), Cfg: cfg,
	}}

	r = gin.New()
	g := r.Group("/api/v1", loadUserCtx())
	g.Use(middleware.UserCapabilityRestrict(db, rdb, true))
	g.POST("/videos", dummyOK)              // publish
	g.POST("/videos/:id/comments", dummyOK) // comment

	gadmin := g.Group("/admin", loadUserCtx())
	gadmin.GET("/users/:id/capabilities", api.AdminListUserCapabilities)
	gadmin.POST("/users/:id/capabilities", api.AdminRestrictUserCapability)
	gadmin.DELETE("/users/:id/capabilities/:capability", api.AdminRestoreUserCapability)
	return api, r
}

func perform(r *gin.Engine, method, path string, uid uint64, body interface{}) *httptest.ResponseRecorder {
	var buf bytes.Buffer
	if body != nil {
		_ = json.NewEncoder(&buf).Encode(body)
	}
	req := httptest.NewRequest(method, path, &buf)
	req.Header.Set("Content-Type", "application/json")
	if uid > 0 {
		req.Header.Set("X-Test-Uid", strconv.FormatUint(uid, 10))
		req.Header.Set("X-Test-Aid", strconv.FormatUint(uid, 10))
	}
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)
	return w
}

func TestCapabilityRestrictAndRestore(t *testing.T) {
	api, r := newUsercapTestEnv(t)

	// Unrestricted user can comment and publish.
	require.Equal(t, http.StatusOK, perform(r, "POST", "/api/v1/videos/1/comments", 5, nil).Code)
	require.Equal(t, http.StatusOK, perform(r, "POST", "/api/v1/videos", 5, nil).Code)

	// Admin restricts user 5's comment capability.
	require.Equal(t, http.StatusOK, perform(r, "POST", "/api/v1/admin/users/5/capabilities", 1,
		gin.H{"capability": "comment", "reason": "刷屏"}).Code)

	// Comment now blocked, publish still allowed.
	require.Equal(t, http.StatusForbidden, perform(r, "POST", "/api/v1/videos/1/comments", 5, nil).Code)
	require.Equal(t, http.StatusOK, perform(r, "POST", "/api/v1/videos", 5, nil).Code)

	// Other users unaffected.
	require.Equal(t, http.StatusOK, perform(r, "POST", "/api/v1/videos/1/comments", 9, nil).Code)

	// Admin restores → comment allowed again.
	require.Equal(t, http.StatusOK, perform(r, "DELETE", "/api/v1/admin/users/5/capabilities/comment", 1, nil).Code)
	require.Equal(t, http.StatusOK, perform(r, "POST", "/api/v1/videos/1/comments", 5, nil).Code)

	// History row kept (status restored).
	var cnt int64
	api.DB.Model(&model.UserCapabilityRestriction{}).
		Where("user_id = ? AND capability = ? AND status = 'restored'", 5, "comment").Count(&cnt)
	require.Equal(t, int64(1), cnt)
}

// TestCapabilityAppealRestores verifies a user can appeal a capability
// restriction and the admin's approve lifts it (部分权力恢复).
func TestCapabilityAppealRestores(t *testing.T) {
	api, r := newUsercapTestEnv(t)
	_ = api // env used for admin routes below

	// Restrict user 5's coin capability directly via DB+model for the appeal flow test.
	require.NoError(t, dataInsertCap(t, api, 5, "coin"))

	// User 5 submits a capability appeal targeting themselves.
	w := perform(r, "POST", "/api/v1/videos/5/comments", 5, nil) // sanity: not coin, allowed
	_ = w
	// We need the appeal endpoints; reuse appealRouter-like with this api.
	ar := gin.New()
	auth := ar.Group("/api/v1", loadUserCtx())
	auth.POST("/appeals", api.PostAppeal)
	grp := ar.Group("/api/v1/admin", loadUserCtx())
	grp.POST("/appeals/:id/handle", api.AdminHandleAppeal)

	var buf bytes.Buffer
	_ = json.NewEncoder(&buf).Encode(gin.H{
		"target_type": "capability", "target_id": 5, "reason_type": "ban",
		"content": "投币功能被误限制", "capability": "coin",
	})
	req := httptest.NewRequest("POST", "/api/v1/appeals", &buf)
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Test-Uid", "5")
	rew := httptest.NewRecorder()
	ar.ServeHTTP(rew, req)
	require.Equal(t, http.StatusOK, rew.Code, rew.Body.String())
	var out struct {
		Data struct {
			ID uint64 `json:"id"`
		} `json:"data"`
	}
	require.NoError(t, json.Unmarshal(rew.Body.Bytes(), &out))
	require.True(t, out.Data.ID > 0)

	// Admin approves → capability restriction lifted.
	buf.Reset()
	_ = json.NewEncoder(&buf).Encode(gin.H{"action": "approve"})
	areq := httptest.NewRequest("POST", "/api/v1/admin/appeals/"+strconv.FormatUint(out.Data.ID, 10)+"/handle", &buf)
	areq.Header.Set("Content-Type", "application/json")
	areq.Header.Set("X-Test-Aid", "1")
	aw := httptest.NewRecorder()
	ar.ServeHTTP(aw, areq)
	require.Equal(t, http.StatusOK, aw.Code, aw.Body.String())

	var cnt int64
	api.DB.Model(&model.UserCapabilityRestriction{}).
		Where("user_id = ? AND capability = ? AND status = 'active'", 5, "coin").Count(&cnt)
	require.Equal(t, int64(0), cnt, "capability restriction should be lifted after approval")
}

func dataInsertCap(t *testing.T, api *API, uid uint64, capability string) error {
	return api.DB.Create(&model.UserCapabilityRestriction{
		UserID: uid, Capability: capability, Reason: "test", Status: "active",
	}).Error
}
