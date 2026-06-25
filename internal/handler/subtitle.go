package handler

import (
	"io"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	e "minibili/internal/errcode"
	"minibili/internal/model"
	"minibili/internal/worker"
)

// ─── Subtitle Management (Module 3) ───

// ListSubtitles returns all subtitle tracks for a video (public).
func (a *API) ListSubtitles(c *gin.Context) {
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var subs []model.Subtitle
	if err := a.DB.Where("video_id = ?", vid).Order("lang ASC").Find(&subs).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": subs})
}

// GetSubtitle returns a specific subtitle's full content.
func (a *API) GetSubtitle(c *gin.Context) {
	sid, err := strconv.ParseUint(c.Param("subtitleId"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var sub model.Subtitle
	if err := a.DB.First(&sub, sid).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"code": e.CodeNotFound, "msg": e.GetMsg(e.CodeNotFound), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": sub})
}

// UploadSubtitle lets a video uploader add a subtitle track.
func (a *API) UploadSubtitle(c *gin.Context) {
	uid := c.MustGet("user_id").(uint64)
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	// Verify uploader ownership
	var v model.Video
	if err := a.DB.First(&v, vid).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"code": e.CodeNotFound, "msg": e.GetMsg(e.CodeNotFound), "data": nil})
		return
	}
	if v.UserID != uid {
		c.JSON(http.StatusForbidden, gin.H{"code": e.CodeForbidden, "msg": e.GetMsg(e.CodeForbidden), "data": nil})
		return
	}

	lang := c.PostForm("lang")
	if lang == "" {
		lang = "zh"
	}
	title := c.PostForm("title")
	format := c.PostForm("format")
	if format == "" {
		format = "vtt"
	}

	// Read subtitle content from form file or text field
	var content string
	file, fh, err := c.Request.FormFile("file")
	if err == nil && fh != nil {
		b, err := io.ReadAll(file)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
			return
		}
		content = string(b)
		file.Close()
	} else {
		// fallback: read from text field
		content = c.PostForm("content")
	}
	if content == "" {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}

	sub := model.Subtitle{
		VideoID: vid,
		Lang:    lang,
		Title:   title,
		Content: content,
		Format:  format,
	}
	if err := a.DB.Create(&sub).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	a.Log.Info("subtitle uploaded", zap.Uint64("video_id", vid), zap.Uint64("subtitle_id", sub.ID), zap.String("lang", lang))
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": sub})
}

// DeleteSubtitle removes a subtitle (uploader or admin).
func (a *API) DeleteSubtitle(c *gin.Context) {
	uid := c.MustGet("user_id").(uint64)
	sid, err := strconv.ParseUint(c.Param("subtitleId"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var sub model.Subtitle
	if err := a.DB.First(&sub, sid).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"code": e.CodeNotFound, "msg": e.GetMsg(e.CodeNotFound), "data": nil})
		return
	}
	// Verify ownership via video
	var v model.Video
	if err := a.DB.First(&v, sub.VideoID).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"code": e.CodeNotFound, "msg": e.GetMsg(e.CodeNotFound), "data": nil})
		return
	}
	if v.UserID != uid {
		c.JSON(http.StatusForbidden, gin.H{"code": e.CodeForbidden, "msg": e.GetMsg(e.CodeForbidden), "data": nil})
		return
	}
	if err := a.DB.Delete(&sub).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": nil})
}

// ─── Admin Subtitle Management ───

// AdminListSubtitles lists all subtitles across the platform.
func (a *API) AdminListSubtitles(c *gin.Context) {
	var subs []model.Subtitle
	query := a.DB.Order("created_at DESC")
	if vid := c.Query("video_id"); vid != "" {
		query = query.Where("video_id = ?", vid)
	}
	if lang := c.Query("lang"); lang != "" {
		query = query.Where("lang = ?", lang)
	}
	if err := query.Find(&subs).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": subs})
}

// AdminDeleteSubtitle force-deletes a subtitle.
func (a *API) AdminDeleteSubtitle(c *gin.Context) {
	sid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	if err := a.DB.Delete(&model.Subtitle{}, sid).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": nil})
}

// ─── ASR (Automatic Speech Recognition) ───

// RequestSubtitleASR creates a subtitle placeholder and queues it for ASR processing.
// POST /api/v1/videos/:id/subtitles/asr
func (a *API) RequestSubtitleASR(c *gin.Context) {
	uid := c.MustGet("user_id").(uint64)
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}

	// Verify uploader ownership
	var v model.Video
	if err := a.DB.First(&v, vid).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"code": e.CodeNotFound, "msg": e.GetMsg(e.CodeNotFound), "data": nil})
		return
	}
	if v.UserID != uid {
		c.JSON(http.StatusForbidden, gin.H{"code": e.CodeForbidden, "msg": e.GetMsg(e.CodeForbidden), "data": nil})
		return
	}

	if v.Status != "published" {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": "视频尚未发布，无法发起自动转写", "data": nil})
		return
	}

	var req worker.NewSubtitleASRRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": "参数错误: " + err.Error(), "data": nil})
		return
	}
	if req.Lang == "" {
		req.Lang = "zh"
	}
	if req.Title == "" {
		req.Title = "自动转写"
	}

	sub, err := worker.RequestASR(a.DB, vid, req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": "创建转写任务失败", "data": nil})
		return
	}

	a.Log.Info("ASR subtitle requested",
		zap.Uint64("video_id", vid),
		zap.Uint64("subtitle_id", sub.ID),
		zap.String("lang", req.Lang),
	)

	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": "自动转写任务已创建，处理完成后字幕将自动出现", "data": sub})
}
