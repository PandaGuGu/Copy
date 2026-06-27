package handler

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
	"go.uber.org/zap"

	"minibili/internal/model"
)

var wsUpgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

// ServeDanmaku upgrades to WebSocket (F6, S-011).
// 已发布稿件：无 token 也可连接，用于实时弹幕与「正在看」计数；非空但非法 token 仍返回 auth_failed。
func (a *API) ServeDanmaku(c *gin.Context) {
	videoID, _ := strconv.ParseUint(c.Query("video_id"), 10, 64)
	if videoID == 0 {
		c.Status(http.StatusBadRequest)
		return
	}
	token := strings.TrimSpace(c.Query("token"))
	var v model.Video
	if err := a.DB.First(&v, videoID).Error; err != nil {
		c.Status(http.StatusNotFound)
		return
	}
	if token != "" {
		uid, _, err := a.JWT.ParseAccess(token)
		if err != nil {
			conn, errUp := wsUpgrader.Upgrade(c.Writer, c.Request, nil)
			if errUp == nil && conn != nil {
				_ = conn.WriteJSON(gin.H{"type": "auth_failed", "msg": "Token 无效或已过期"})
				_ = conn.Close()
			}
			return
		}
		if v.Status != "published" && v.UserID != uid {
			c.Status(http.StatusNotFound)
			return
		}
	} else {
		if v.Status != "published" {
			c.Status(http.StatusNotFound)
			return
		}
	}
	conn, err := wsUpgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		return
	}
	defer func() {
		a.Hub.Leave(videoID, conn)
		a.pushWatchingCount(videoID)
		_ = conn.Close()
	}()
	a.Hub.Join(videoID, conn)
	a.pushWatchingCount(videoID)

	var hist []model.Danmaku
	_ = a.DB.Where("video_id = ?", videoID).Order("id DESC").Limit(200).Find(&hist).Error
	items := make([]gin.H, 0, len(hist))
	for i := len(hist) - 1; i >= 0; i-- {
		d := hist[i]
		var u model.User
		_ = a.DB.First(&u, d.UserID).Error
		items = append(items, gin.H{
			"id":         d.ID,
			"content":    d.Content,
			"color":      strings.ToUpper(strings.TrimSpace(d.Color)),
			"type":       d.Type,
			"font_size":  danmakuFontSizeField(d),
			"video_time": d.VideoTime,
			"user":       model.DisplayUsername(&u),
			"created_at": d.CreatedAt.Format("2006-01-02 15:04:05"),
		})
	}
	watching := a.Hub.RoomSize(videoID)
	_ = conn.WriteJSON(gin.H{"type": "history", "items": items, "watching_count": watching})

	for {
		_ = conn.SetReadDeadline(time.Now().Add(120 * time.Second))
		if _, _, err := conn.ReadMessage(); err != nil {
			break
		}
	}
}

func (a *API) pushWatchingCount(videoID uint64) {
	if a.Hub == nil {
		return
	}
	n := a.Hub.RoomSize(videoID)
	payload := gin.H{"type": "watching", "count": n}
	if a.DanmakuRelay != nil {
		if err := a.DanmakuRelay.Publish(context.Background(), videoID, payload); err != nil {
			a.Log.Error("danmaku relay publish watching", zap.Error(err))
		}
		return
	}
	a.Hub.BroadcastJSON(videoID, payload)
}

// ──────────────────────────────────────────────
// Live Chat WebSocket with audience tracking
// ──────────────────────────────────────────────

type liveAudienceTracker struct {
	mu    sync.Mutex
	rooms map[uint64]map[string]int
}

var globalAudience = &liveAudienceTracker{
	rooms: make(map[uint64]map[string]int),
}

func (l *liveAudienceTracker) join(roomID uint64, username string) int {
	l.mu.Lock()
	defer l.mu.Unlock()
	if l.rooms[roomID] == nil {
		l.rooms[roomID] = make(map[string]int)
	}
	l.rooms[roomID][username]++
	return len(l.rooms[roomID])
}

func (l *liveAudienceTracker) leave(roomID uint64, username string) int {
	l.mu.Lock()
	defer l.mu.Unlock()
	if m := l.rooms[roomID]; m != nil {
		m[username]--
		if m[username] <= 0 {
			delete(m, username)
		}
		if len(m) == 0 {
			delete(l.rooms, roomID)
			return 0
		}
		return len(m)
	}
	return 0
}

func (l *liveAudienceTracker) list(roomID uint64) []string {
	l.mu.Lock()
	defer l.mu.Unlock()
	if m := l.rooms[roomID]; m != nil {
		list := make([]string, 0, len(m))
		for name := range m {
			list = append(list, name)
		}
		return list
	}
	return nil
}

// ServeLiveChat handles real-time chat in live rooms.
// Query params: room_id (required), token (optional JWT for username)
func (a *API) ServeLiveChat(c *gin.Context) {
	roomID, _ := strconv.ParseUint(c.Query("room_id"), 10, 64)
	if roomID == 0 {
		c.Status(http.StatusBadRequest)
		return
	}

	var room model.LiveRoom
	if err := a.DB.First(&room, roomID).Error; err != nil {
		c.Status(http.StatusNotFound)
		return
	}

	username := "游客"
	var userID uint64
	token := strings.TrimSpace(c.Query("token"))
	if token != "" {
		if uid, _, err := a.JWT.ParseAccess(token); err == nil {
			userID = uid
			var u model.User
			if err := a.DB.Select("username").First(&u, uid).Error; err == nil {
				username = model.DisplayUsername(&u)
			}
		}
	}

	conn, err := wsUpgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		return
	}
	defer func() {
		a.Hub.Leave(roomID, conn)
		globalAudience.leave(roomID, username)
		_ = conn.Close()
	}()

	a.Hub.Join(roomID, conn)
	globalAudience.join(roomID, username)

	// Record live view history when a logged-in user enters the live room
	if userID > 0 {
		a.RecordLiveViewHistory(userID, roomID, "web")
	}

	// Send current audience list and user info
	conn.WriteJSON(gin.H{
		"type":         "user_info",
		"username":     username,
		"users":        globalAudience.list(roomID),
		"user_count":   len(globalAudience.list(roomID)),
		"broadcaster":  room.UserID == userID,
	})

	// Broadcast audience update
	a.Hub.BroadcastJSON(roomID, gin.H{
		"type":    "audience",
		"users":   globalAudience.list(roomID),
		"count":   len(globalAudience.list(roomID)),
	})

	a.Hub.BroadcastJSON(roomID, gin.H{
		"type": "system",
		"msg":  username + " 进入了直播间",
	})

	for {
		_ = conn.SetReadDeadline(time.Now().Add(120 * time.Second))
		_, msgBytes, err := conn.ReadMessage()
		if err != nil {
			break
		}

		var msg struct {
			Content string `json:"content"`
			Gift    string `json:"gift"`  // gift type: "rose", "heart", "rocket" etc
			Follow  bool   `json:"follow"` // follow broadcaster
		}
		if err := json.Unmarshal(msgBytes, &msg); err != nil {
			continue
		}

		// Follow action
		if msg.Follow && userID > 0 && room.UserID != userID {
			a.handleLiveFollow(conn, userID, room.UserID, username)
			continue
		}

		// Gift action
		if msg.Gift != "" && userID > 0 {
			a.handleLiveGift(conn, roomID, username, msg.Gift, room.UserID)
			continue
		}

		// Chat message
		content := strings.TrimSpace(msg.Content)
		if content == "" {
			continue
		}
		if len([]rune(content)) > 200 {
			content = string([]rune(content)[:200])
		}

		a.Hub.BroadcastJSON(roomID, gin.H{
			"type":     "message",
			"username": username,
			"content":  content,
		})
	}

	a.Hub.BroadcastJSON(roomID, gin.H{
		"type":    "audience",
		"users":   globalAudience.list(roomID),
		"count":   len(globalAudience.list(roomID)),
		"system":  true,
		"msg":     username + " 离开了直播间",
	})
}

func (a *API) handleLiveFollow(conn *websocket.Conn, followerID, followeeID uint64, username string) {
	var existing model.UserFollow
	err := a.DB.Where("follower_id = ? AND followee_id = ?", followerID, followeeID).First(&existing).Error
	if err == nil {
		conn.WriteJSON(gin.H{"type": "system", "msg": "你已经关注过了"})
		return
	}

	now := time.Now()
	if err := a.DB.Create(&model.UserFollow{
		FollowerID: followerID,
		FolloweeID: followeeID,
		CreatedAt:  now,
	}).Error; err != nil {
		conn.WriteJSON(gin.H{"type": "system", "msg": "关注失败"})
		return
	}

	var u model.User
	a.DB.Select("username").First(&u, followeeID)
	conn.WriteJSON(gin.H{"type": "system", "msg": "已关注 " + model.DisplayUsername(&u)})
}

func (a *API) handleLiveGift(conn *websocket.Conn, roomID uint64, username, gift string, _ uint64) {
	validGifts := map[string]string{
		"rose":   "🌹 玫瑰",
		"heart":  "❤️ 小心心",
		"rocket": "🚀 火箭",
		"star":   "⭐ 星星",
		"cake":   "🍰 蛋糕",
		"flower": "🌸 花束",
	}
	label, ok := validGifts[gift]
	if !ok {
		return
	}

	a.Hub.BroadcastJSON(roomID, gin.H{
		"type":     "gift",
		"username": username,
		"gift":     gift,
		"label":    label,
		"content":  fmt.Sprintf("%s 送出 %s", username, label),
	})
}
