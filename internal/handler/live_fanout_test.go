package handler

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/require"
)

// TestHubFanoutCrossInstance proves the live-room fanout path used by
// hubFanout: publish → Redis Pub/Sub (DanmakuRelay) → subscriber → Hub →
// a connected WebSocket client. This is the mechanism that lets live chat
// cross API replicas (M-直播集群).
func TestHubFanoutCrossInstance(t *testing.T) {
	api, _, _ := newTestAPI(t)

	joinDone := make(chan struct{})
	upgrader := websocket.Upgrader{CheckOrigin: func(r *http.Request) bool { return true }}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			return
		}
		api.Hub.Join(7, conn)
		close(joinDone)
		<-r.Context().Done()
	}))
	defer srv.Close()

	wsURL := "ws" + srv.URL[len("http"):]
	conn, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
	require.NoError(t, err)
	defer conn.Close()
	<-joinDone // ensure server-side joined before publishing

	// Fan out via the relay (Redis) — a second/hmi API pod would receive the same.
	api.hubFanout(context.Background(), 7, gin.H{"type": "message", "username": "alice", "content": "hi"})

	conn.SetReadDeadline(time.Now().Add(5 * time.Second))
	_, data, err := conn.ReadMessage()
	require.NoError(t, err)

	var got map[string]interface{}
	require.NoError(t, json.Unmarshal(data, &got))
	require.Equal(t, "message", got["type"])
	require.Equal(t, "alice", got["username"])
	require.Equal(t, "hi", got["content"])
}
