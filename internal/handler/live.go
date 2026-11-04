package handler

import (
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"go.uber.org/zap"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/coverval"
	"minibili/internal/pkg/resp"
)

// ──────────────────────────────────────────────
// Public: Live Room List
// ──────────────────────────────────────────────

// ListLiveRooms GET /api/v1/live/rooms — list live rooms (filter: status=live)
func (a *API) ListLiveRooms(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 60 {
		pageSize = 20
	}

	q := a.DB.Model(&model.LiveRoom{})
	if s := strings.TrimSpace(c.Query("status")); s != "" {
		q = q.Where("status = ?", s)
	}

	var total int64
	q.Count(&total)

	var rooms []model.LiveRoom
	offset := (page - 1) * pageSize
	if err := q.Order("viewer_count DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rooms).Error; err != nil {
		a.Log.Error("list live rooms", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{
		"rooms": rooms,
		"total": total,
	})
}

// GetLiveRoom GET /api/v1/live/room/:id
func (a *API) GetLiveRoom(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var room model.LiveRoom
	if err := a.DB.First(&room, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	// Populate avatar from user if empty
	if strings.TrimSpace(room.AvatarURL) == "" {
		var u model.User
		if err := a.DB.Select("avatar_url").First(&u, room.UserID).Error; err == nil {
			room.AvatarURL = strings.TrimSpace(u.AvatarURL)
		}
	}

	// Record live view history when a logged-in user opens the live room page.
	// This is the primary entry point (more reliable than the WebSocket path).
	authHeader := c.GetHeader("Authorization")
	if strings.HasPrefix(authHeader, "Bearer ") {
		token := strings.TrimPrefix(authHeader, "Bearer ")
		if uid, _, err := a.JWT.ParseAccess(token); err == nil && uid > 0 {
			a.RecordLiveViewHistory(uid, id, "web")
		}
	}

	resp.OK(c, room)
}

// ──────────────────────────────────────────────
// Auth: Create / Update Live Room
// ──────────────────────────────────────────────

// ──────────────────────────────────────────────
// Auth: Get or Create My Live Room
// ──────────────────────────────────────────────

// GetOrCreateMyLiveRoom GET /api/v1/live/room/my — returns existing room or auto-creates one
func (a *API) GetOrCreateMyLiveRoom(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	// Try to find existing room for this user
	var room model.LiveRoom
	err := a.DB.Where("user_id = ?", userID).Order("id DESC").First(&room).Error
	if err == nil {
		// Populate avatar from user if empty
		if strings.TrimSpace(room.AvatarURL) == "" {
			var u model.User
			if err := a.DB.Select("avatar_url").First(&u, userID).Error; err == nil {
				room.AvatarURL = strings.TrimSpace(u.AvatarURL)
			}
		}
		resp.OK(c, room)
		return
	}

	// Auto-create a room with default title
	var user model.User
	if err := a.DB.Select("username, avatar_url").First(&user, userID).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	streamKey := strings.ReplaceAll(uuid.New().String(), "-", "")[:32]
	room = model.LiveRoom{
		UserID:    userID,
		Title:     "未命名直播间",
		StreamKey: streamKey,
		Status:    "idle",
		HostName:  user.Username,
		AvatarURL: strings.TrimSpace(user.AvatarURL),
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}
	if err := a.DB.Create(&room).Error; err != nil {
		a.Log.Error("auto create live room", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, room)
}

// CreateLiveRoom POST /api/v1/live/room/create
func (a *API) CreateLiveRoom(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	var body struct {
		Title    string `json:"title"`
		CoverURL string `json:"cover_url"`
	}
	if err := c.ShouldBindJSON(&body); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	title := strings.TrimSpace(body.Title)
	if title == "" || len(title) > 60 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeTitleInvalid)
		return
	}

	// Look up username for host_name
	var user model.User
	if err := a.DB.Select("username").First(&user, userID).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	streamKey := strings.ReplaceAll(uuid.New().String(), "-", "")[:32]

	room := model.LiveRoom{
		UserID:    userID,
		Title:     title,
		CoverURL:  strings.TrimSpace(body.CoverURL),
		StreamKey: streamKey,
		Status:    "idle",
		HostName:  user.Username,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}

	if err := a.DB.Create(&room).Error; err != nil {
		a.Log.Error("create live room", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, room)
}

// UpdateLiveRoom PUT /api/v1/live/room/:id
func (a *API) UpdateLiveRoom(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var room model.LiveRoom
	if err := a.DB.First(&room, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if room.UserID != userID {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	var body struct {
		Title    *string `json:"title"`
		CoverURL *string `json:"cover_url"`
	}
	if err := c.ShouldBindJSON(&body); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	updates := map[string]interface{}{"updated_at": time.Now()}
	if body.Title != nil {
		t := strings.TrimSpace(*body.Title)
		if t == "" || len(t) > 60 {
			resp.Err(c, http.StatusBadRequest, errcode.CodeTitleInvalid)
			return
		}
		updates["title"] = t
	}
	if body.CoverURL != nil {
		updates["cover_url"] = strings.TrimSpace(*body.CoverURL)
	}

	if err := a.DB.Model(&room).Updates(updates).Error; err != nil {
		a.Log.Error("update live room", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	if v, ok := updates["title"]; ok {
		room.Title = v.(string)
	}
	if v, ok := updates["cover_url"]; ok {
		room.CoverURL = v.(string)
	}
	resp.OK(c, room)
}

// RegenerateStreamKey POST /api/v1/live/room/:id/regenerate-key
func (a *API) RegenerateStreamKey(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var room model.LiveRoom
	if err := a.DB.First(&room, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if room.UserID != userID {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	newKey := strings.ReplaceAll(uuid.New().String(), "-", "")[:32]
	if err := a.DB.Model(&room).Updates(map[string]interface{}{
		"stream_key": newKey,
		"updated_at": time.Now(),
	}).Error; err != nil {
		a.Log.Error("regenerate stream key", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	room.StreamKey = newKey
	resp.OK(c, room)
}

// StartLiveRoom POST /api/v1/live/room/:id/start — called by SRS on_publish callback
func (a *API) StartLiveRoom(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// For SRS callback, we check the stream key in the callback handler.
	// This public endpoint just marks a room as live.
	now := time.Now()
	if err := a.DB.Model(&model.LiveRoom{}).Where("id = ?", id).
		Updates(map[string]interface{}{
			"status":     "live",
			"started_at": now,
			"updated_at": now,
		}).Error; err != nil {
		a.Log.Error("start live room", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"status": "live"})
}

// EndLiveRoom POST /api/v1/live/room/:id/end
func (a *API) EndLiveRoom(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	now := time.Now()
	if err := a.DB.Model(&model.LiveRoom{}).Where("id = ?", id).
		Updates(map[string]interface{}{
			"status":     "ended",
			"ended_at":   now,
			"updated_at": now,
		}).Error; err != nil {
		a.Log.Error("end live room", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"status": "ended"})
}

// ──────────────────────────────────────────────
// SRS Callbacks (verify stream_key on publish)
// ──────────────────────────────────────────────

// srsHookStreamKey extracts the stream name from an SRS http_hooks request.
//
// SRS 5 posts a JSON body (Content-Type: application/json):
//
//	{"action":"on_publish","client_id":"...","vhost":"__defaultVhost__",
//	 "app":"live","stream":"<stream_key>","param":"", ...}
//
// Older node-media-server style configs use query/form fields instead, so all
// three shapes (query / form / JSON) are accepted.
func srsHookStreamKey(c *gin.Context) string {
	keys := []string{"stream", "name", "stream_key"}
	for _, k := range keys {
		if v := strings.TrimSpace(c.Query(k)); v != "" {
			return v
		}
	}
	for _, k := range keys {
		if v := strings.TrimSpace(c.PostForm(k)); v != "" {
			return v
		}
	}
	var body struct {
		StreamKey string `json:"stream_key"`
		Stream    string `json:"stream"`
		Name      string `json:"name"`
	}
	if err := c.ShouldBindJSON(&body); err == nil {
		for _, v := range []string{body.Stream, body.Name, body.StreamKey} {
			if s := strings.TrimSpace(v); s != "" {
				return s
			}
		}
	}
	return ""
}

// SRSOnPublish POST /api/v1/live/callback/on_publish — node-media-server calls this when stream starts
func (a *API) SRSOnPublish(c *gin.Context) {
	room, ok := a.liveRoomByStreamKey(srsHookStreamKey(c))
	if !ok {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if room.Status == "banned" {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	now := time.Now()
	a.DB.Model(&room).Updates(map[string]interface{}{
		"status":     "live",
		"started_at": now,
		"updated_at": now,
	})
	resp.OK(c, gin.H{"code": 0, "room_id": room.ID, "status": "live"})
}

// liveRoomByStreamKey 按 SRS 上报的流名找直播间。
//
// SRS 转码出来的多码率流是「<stream_key>_<engine>」（如 `<key>_sd`），它们同样会走
// on_publish 回调。这些变体不是独立的房间，去掉后缀再按原始 key 查一次即可，
// 否则 SRS 收到 404 会直接拒绝发布，转码产物就出不来（直播清晰度也就没法切）。
func (a *API) liveRoomByStreamKey(streamKey string) (model.LiveRoom, bool) {
	var room model.LiveRoom
	if streamKey == "" {
		return room, false
	}
	if err := a.DB.Where("stream_key = ?", streamKey).First(&room).Error; err == nil {
		return room, true
	}
	if i := strings.LastIndex(streamKey, "_"); i > 0 {
		base := streamKey[:i]
		if err := a.DB.Where("stream_key = ?", base).First(&room).Error; err == nil {
			return room, true
		}
	}
	return room, false
}

// SRSOnDone POST /api/v1/live/callback/on_done — node-media-server calls when stream ends
func (a *API) SRSOnDone(c *gin.Context) {
	streamKey := srsHookStreamKey(c)
	if streamKey == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	a.DB.Model(&model.LiveRoom{}).Where("stream_key = ?", streamKey).
		Updates(map[string]interface{}{
			"status":     "ended",
			"ended_at":   now,
			"updated_at": now,
		})
	resp.OK(c, gin.H{"code": 0})
}

// UploadLiveCover POST /api/v1/live/room/:id/cover — upload cover image from local
func (a *API) UploadLiveCover(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var room model.LiveRoom
	if err := a.DB.First(&room, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if room.UserID != userID {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	if err := c.Request.ParseMultipartForm(6 << 20); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	fh, err := c.FormFile("cover")
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if code := coverval.ValidateCoverHeader(fh); code != 0 {
		resp.Err(c, http.StatusBadRequest, code)
		return
	}

	if a.OSS == nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	if err := os.MkdirAll(a.Cfg.TempUploadDir, 0o755); err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	tmp := filepath.Join(a.Cfg.TempUploadDir, uuid.NewString()+filepath.Ext(fh.Filename))
	if err := saveUploadedFile(fh, tmp); err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	defer os.Remove(tmp)

	ext := strings.TrimPrefix(strings.ToLower(filepath.Ext(fh.Filename)), ".")
	if ext == "jpeg" {
		ext = "jpg"
	}
	key := fmt.Sprintf("live-covers/%d.%s", id, ext)
	if err := a.OSS.UploadFile(key, tmp); err != nil {
		a.Log.Error("oss live cover upload", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	url := a.Cfg.OSSObjectURL(key)

	if err := a.DB.Model(&room).Updates(map[string]interface{}{
		"cover_url":  url,
		"updated_at": time.Now(),
	}).Error; err != nil {
		a.Log.Error("live cover url save", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"cover_url": url})
}
