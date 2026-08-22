package handler

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/require"

	"minibili/internal/middleware"
	"minibili/internal/model"
)

func performAppeal(r *gin.Engine, method, path string, body interface{}) (*httptest.ResponseRecorder, map[string]interface{}) {
	var buf bytes.Buffer
	if body != nil {
		_ = json.NewEncoder(&buf).Encode(body)
	}
	req := httptest.NewRequest(method, path, &buf)
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)

	var out map[string]interface{}
	_ = json.Unmarshal(w.Body.Bytes(), &out)
	return w, out
}

func makeUser(t *testing.T, api *API, status string) model.User {
	t.Helper()
	u := model.User{Username: "appeal_tester", PasswordHash: "x", Status: status}
	require.NoError(t, api.DB.Create(&u).Error)
	return u
}

func mockCtx(userID, adminID uint64) gin.HandlerFunc {
	return func(c *gin.Context) {
		if userID > 0 {
			c.Set(middleware.CtxUserIDKey, userID)
		}
		if adminID > 0 {
			c.Set(middleware.CtxAdminIDKey, adminID)
		}
	}
}

func appealRouter(api *API, userID, adminID uint64) *gin.Engine {
	gin.SetMode(gin.TestMode)
	r := gin.New()
	auth := r.Group("/api/v1", mockCtx(userID, 0))
	auth.POST("/appeals", api.PostAppeal)
	auth.GET("/appeals/me", api.ListMyAppeals)
	auth.GET("/appeals/:id", api.GetAppeal)
	admin := r.Group("/api/v1/admin", mockCtx(0, adminID))
	admin.GET("/appeals", api.AdminListAppeals)
	admin.POST("/appeals/:id/handle", api.AdminHandleAppeal)
	return r
}

func TestAppealSubmitAndDedupe(t *testing.T) {
	api, _, _ := newTestAPI(t)
	u := makeUser(t, api, "banned")
	r := appealRouter(api, u.ID, 1)

	w, out := performAppeal(r, "POST", "/api/v1/appeals", gin.H{
		"target_type": "user", "target_id": u.ID, "reason_type": "ban", "content": "我是被误封的",
	})
	require.Equal(t, http.StatusOK, w.Code, w.Body.String())
	require.Equal(t, "pending", out["data"].(map[string]interface{})["status"])

	// Duplicate pending appeal → rejected
	w2, _ := performAppeal(r, "POST", "/api/v1/appeals", gin.H{
		"target_type": "user", "target_id": u.ID, "reason_type": "ban", "content": "再申诉一次",
	})
	require.Equal(t, http.StatusBadRequest, w2.Code, w2.Body.String())
}

func TestAppealOwnershipDenied(t *testing.T) {
	api, _, _ := newTestAPI(t)
	owner := makeUser(t, api, "banned")
	r := appealRouter(api, 99, 1) // different user than owner
	w, _ := performAppeal(r, "POST", "/api/v1/appeals", gin.H{
		"target_type": "user", "target_id": owner.ID, "reason_type": "ban", "content": "越权申诉",
	})
	require.Equal(t, http.StatusForbidden, w.Code, w.Body.String())
}

func TestAppealApproveUnbansUser(t *testing.T) {
	api, _, _ := newTestAPI(t)
	u := makeUser(t, api, "banned")
	r := appealRouter(api, u.ID, 1)

	_, out := performAppeal(r, "POST", "/api/v1/appeals", gin.H{
		"target_type": "user", "target_id": u.ID, "reason_type": "ban", "content": "解封申诉",
	})
	appealID := strconv.FormatUint(uint64(out["data"].(map[string]interface{})["id"].(float64)), 10)

	w, res := performAppeal(r, "POST", "/api/v1/admin/appeals/"+appealID+"/handle", gin.H{
		"action": "approve", "admin_note": "查无实据，予以解封",
	})
	require.Equal(t, http.StatusOK, w.Code, w.Body.String())
	require.Equal(t, "approved", res["data"].(map[string]interface{})["status"])

	var u2 model.User
	require.NoError(t, api.DB.First(&u2, u.ID).Error)
	require.Equal(t, "active", u2.Status)
}

func TestAppealRejectRequiresNote(t *testing.T) {
	api, _, _ := newTestAPI(t)
	u := makeUser(t, api, "banned")
	r := appealRouter(api, u.ID, 1)

	_, out := performAppeal(r, "POST", "/api/v1/appeals", gin.H{
		"target_type": "user", "target_id": u.ID, "reason_type": "ban", "content": "驳回测试",
	})
	appealID := strconv.FormatUint(uint64(out["data"].(map[string]interface{})["id"].(float64)), 10)

	// Missing note → 400
	w, _ := performAppeal(r, "POST", "/api/v1/admin/appeals/"+appealID+"/handle", gin.H{"action": "reject"})
	require.Equal(t, http.StatusBadRequest, w.Code, w.Body.String())

	// With note → rejected
	w2, res := performAppeal(r, "POST", "/api/v1/admin/appeals/"+appealID+"/handle", gin.H{
		"action": "reject", "admin_note": "证据充分，维持处罚",
	})
	require.Equal(t, http.StatusOK, w2.Code, w2.Body.String())
	require.Equal(t, "rejected", res["data"].(map[string]interface{})["status"])

	var u2 model.User
	require.NoError(t, api.DB.First(&u2, u.ID).Error)
	require.Equal(t, "banned", u2.Status)
}

func TestAppealRestoresVideo(t *testing.T) {
	api, _, _ := newTestAPI(t)
	u := makeUser(t, api, "active")
	r := appealRouter(api, u.ID, 1)

	v := model.Video{Title: "违规视频", Status: "published", UserID: u.ID}
	require.NoError(t, api.DB.Create(&v).Error)
	_ = api.moderateVideo(v.ID, "takedown")

	_, out := performAppeal(r, "POST", "/api/v1/appeals", gin.H{
		"target_type": "video", "target_id": v.ID, "reason_type": "takedown", "content": "误下架，申请恢复",
	})
	appealID := strconv.FormatUint(uint64(out["data"].(map[string]interface{})["id"].(float64)), 10)

	w, res := performAppeal(r, "POST", "/api/v1/admin/appeals/"+appealID+"/handle", gin.H{"action": "approve"})
	require.Equal(t, http.StatusOK, w.Code, w.Body.String())
	require.Equal(t, "approved", res["data"].(map[string]interface{})["status"])

	var v2 model.Video
	require.NoError(t, api.DB.First(&v2, v.ID).Error)
	require.Equal(t, "published", v2.Status)
}

func TestListMyAppeals(t *testing.T) {
	api, _, _ := newTestAPI(t)
	u := makeUser(t, api, "banned")

	require.NoError(t, api.DB.Create(&model.Appeal{
		UserID: u.ID, TargetType: "video", TargetID: 5, ReasonType: "takedown",
		Content: "误下架", Status: "pending",
	}).Error)
	// Another user's appeal must NOT leak into my list.
	require.NoError(t, api.DB.Create(&model.Appeal{
		UserID: 999, TargetType: "user", TargetID: 999, ReasonType: "ban",
		Content: "别的用户", Status: "pending",
	}).Error)

	r := appealRouter(api, u.ID, 1)
	w, out := performAppeal(r, "GET", "/api/v1/appeals/me", nil)
	require.Equal(t, http.StatusOK, w.Code, w.Body.String())

	items, ok := out["data"].(map[string]interface{})["items"].([]interface{})
	require.True(t, ok, "items must be an array: %v", out)
	require.Len(t, items, 1)
}
