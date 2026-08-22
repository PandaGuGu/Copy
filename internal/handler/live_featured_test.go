package handler

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/require"

	"minibili/internal/model"
)

func TestLiveFeaturedFlow(t *testing.T) {
	api, _, _ := newTestAPI(t)

	rooms := []model.LiveRoom{
		{UserID: 1, Title: "游戏直播A", StreamKey: "srv-a", Status: "live", ViewerCount: 100},
		{UserID: 2, Title: "音乐直播B", StreamKey: "srv-b", Status: "live", ViewerCount: 50},
		{UserID: 3, Title: "已下播C", StreamKey: "srv-c", Status: "idle", ViewerCount: 0},
	}
	for i := range rooms {
		require.NoError(t, api.DB.Create(&rooms[i]).Error)
	}

	// Minimal engine: public featured + admin setter.
	gin.SetMode(gin.TestMode)
	r := gin.New()
	pub := r.Group("/api/v1")
	pub.GET("/live/featured", api.GetLiveFeatured)
	adm := r.Group("/api/v1/admin", mockCtx(0, 1))
	adm.POST("/live/featured", api.AdminSetLiveFeatured)

	// Nothing pinned yet.
	w := httptest.NewRecorder()
	r.ServeHTTP(w, httptest.NewRequest("GET", "/api/v1/live/featured", nil))
	require.Equal(t, http.StatusOK, w.Code, w.Body.String())
	require.Equal(t, 0, len(getDataArray(w.Body.Bytes())))

	// Admin pins [A, B, C] in order (C is idle).
	buf, _ := json.Marshal(map[string]interface{}{"room_ids": []uint64{rooms[0].ID, rooms[1].ID, rooms[2].ID}})
	req := httptest.NewRequest("POST", "/api/v1/admin/live/featured", bytes.NewReader(buf))
	req.Header.Set("Content-Type", "application/json")
	w = httptest.NewRecorder()
	r.ServeHTTP(w, req)
	require.Equal(t, http.StatusOK, w.Code, w.Body.String())

	// Public featured returns only live rooms, in pin order.
	w = httptest.NewRecorder()
	r.ServeHTTP(w, httptest.NewRequest("GET", "/api/v1/live/featured", nil))
	require.Equal(t, http.StatusOK, w.Code, w.Body.String())
	items := getDataArray(w.Body.Bytes())
	require.Len(t, items, 2)
	require.Equal(t, float64(rooms[0].ID), items[0].(map[string]interface{})["room_id"])
	require.Equal(t, float64(rooms[1].ID), items[1].(map[string]interface{})["room_id"])
}

func getDataArray(b []byte) []interface{} {
	var env struct {
		Data []interface{} `json:"data"`
	}
	_ = json.Unmarshal(b, &env)
	return env.Data
}
