package handler

import (
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
