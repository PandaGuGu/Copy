package handler

import (
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ──────────────────────────────────────────────
// Admin: Live Room Management
// ──────────────────────────────────────────────

// AdminListLiveRooms GET /admin/live/rooms
func (a *API) AdminListLiveRooms(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
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
	if err := q.Order("id DESC").Offset(offset).Limit(pageSize).Find(&rooms).Error; err != nil {
		a.Log.Error("admin list live rooms", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{
		"rooms": rooms,
		"total": total,
	})
}

// AdminBanLiveRoom POST /admin/live/room/:id/ban
func (a *API) AdminBanLiveRoom(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
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

	if err := a.DB.Model(&room).Updates(map[string]interface{}{
		"status":     "banned",
		"updated_at": time.Now(),
	}).Error; err != nil {
		a.Log.Error("ban live room", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Broadcast ban to all viewers in the room
	if a.Hub != nil {
		a.Hub.BroadcastJSON(room.ID, gin.H{
			"type": "admin_ban",
			"msg":  "直播间已被管理员封禁",
		})
	}

	a.recordAudit(c, adminID, "ban", "live_room", id, "Banned live room: "+room.Title)
	resp.OK(c, gin.H{"id": id, "status": "banned"})
}

// AdminUnbanLiveRoom POST /admin/live/room/:id/unban
func (a *API) AdminUnbanLiveRoom(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
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

	if err := a.DB.Model(&room).Updates(map[string]interface{}{
		"status":     "idle",
		"updated_at": time.Now(),
	}).Error; err != nil {
		a.Log.Error("unban live room", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.recordAudit(c, adminID, "unban", "live_room", id, "Unbanned live room: "+room.Title)
	resp.OK(c, gin.H{"id": id, "status": "idle"})
}

// AdminDeleteLiveRoom DELETE /admin/live/room/:id
func (a *API) AdminDeleteLiveRoom(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
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

	if err := a.DB.Delete(&room).Error; err != nil {
		a.Log.Error("delete live room", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.recordAudit(c, adminID, "delete", "live_room", id, "Deleted live room: "+room.Title)
	resp.OK(c, gin.H{"id": id})
}

// AdminGetLiveRoomDetail GET /admin/live/room/:id  — admin detail with stream info
func (a *API) AdminGetLiveRoomDetail(c *gin.Context) {
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
	var u model.User
	_ = a.DB.Select("username").First(&u, room.UserID).Error
	resp.OK(c, gin.H{
		"id":          room.ID,
		"title":       room.Title,
		"host_name":   room.HostName,
		"avatar_url":  room.AvatarURL,
		"cover_url":   room.CoverURL,
		"stream_key":  room.StreamKey,
		"status":      room.Status,
		"viewer_count": room.ViewerCount,
		"user_id":     room.UserID,
		"username":    model.DisplayUsername(&u),
		"started_at":  room.StartedAt,
		"created_at":  room.CreatedAt,
	})
}

// AdminWarnLiveRoom POST /admin/live/room/:id/warn
// Sends a red warning overlay to all viewers for 5 seconds.
func (a *API) AdminWarnLiveRoom(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req struct {
		Reason string `json:"reason"`
	}
	_ = c.ShouldBindJSON(&req)
	reason := strings.TrimSpace(req.Reason)
	if reason == "" {
		reason = "管理员警告：直播内容违规，请立即整改"
	}

	var room model.LiveRoom
	if err := a.DB.First(&room, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	// Broadcast warning to all viewers via WebSocket
	if a.Hub != nil {
		a.Hub.BroadcastJSON(room.ID, gin.H{
			"type":   "admin_warning",
			"reason": reason,
			"msg":    fmt.Sprintf("⚠ 管理员警告：%s", reason),
		})
	}

	a.Log.Info("admin warned live room",
		zap.Uint64("room_id", id),
		zap.Uint64("admin_id", adminID),
		zap.String("reason", reason),
	)
	a.recordAudit(c, adminID, "warn", "live_room", id, "Warned: "+reason)
	resp.OK(c, gin.H{"id": id, "warned": true})
}

// ── Live Warning Templates ──

// AdminListLiveWarnTemplates GET /admin/live/warn-templates
func (a *API) AdminListLiveWarnTemplates(c *gin.Context) {
	var rows []model.LiveWarnTemplate
	if err := a.DB.Order("sort_order ASC, id ASC").Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"templates": rows})
}

// AdminCreateLiveWarnTemplate POST /admin/live/warn-templates
func (a *API) AdminCreateLiveWarnTemplate(c *gin.Context) {
	var req struct {
		Name    string `json:"name"`
		Content string `json:"content"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	req.Content = strings.TrimSpace(req.Content)
	if req.Name == "" || req.Content == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	tmpl := model.LiveWarnTemplate{Name: req.Name, Content: req.Content}
	if err := a.DB.Create(&tmpl).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"id": tmpl.ID})
}

// AdminUpdateLiveWarnTemplate PUT /admin/live/warn-templates/:id
func (a *API) AdminUpdateLiveWarnTemplate(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var tmpl model.LiveWarnTemplate
	if err := a.DB.First(&tmpl, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	var req struct {
		Name    string `json:"name"`
		Content string `json:"content"`
	}
	_ = c.ShouldBindJSON(&req)
	updates := map[string]interface{}{}
	if n := strings.TrimSpace(req.Name); n != "" {
		updates["name"] = n
	}
	if c := strings.TrimSpace(req.Content); c != "" {
		updates["content"] = c
	}
	if len(updates) == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if err := a.DB.Model(&tmpl).Updates(updates).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"id": id})
}

// AdminDeleteLiveWarnTemplate DELETE /admin/live/warn-templates/:id
func (a *API) AdminDeleteLiveWarnTemplate(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var tmpl model.LiveWarnTemplate
	if err := a.DB.First(&tmpl, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if err := a.DB.Delete(&tmpl).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"id": id})
}
