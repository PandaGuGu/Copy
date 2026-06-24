package handler

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	e "minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
)

// ─── Video Chapters (Module 2: Player Advanced) ───

// AdminListVideoChapters returns all chapter markers for a video.
func (a *API) AdminListVideoChapters(c *gin.Context) {
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var chapters []model.VideoChapter
	if err := a.DB.Where("video_id = ?", vid).Order("time_sec ASC").Find(&chapters).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": chapters})
}

// AdminCreateVideoChapter adds a chapter point to a video.
func (a *API) AdminCreateVideoChapter(c *gin.Context) {
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var req struct {
		Title   string  `json:"title" binding:"required"`
		TimeSec float64 `json:"time_sec" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	ch := model.VideoChapter{VideoID: vid, Title: req.Title, TimeSec: req.TimeSec}
	if err := a.DB.Create(&ch).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	a.Log.Info("admin created video chapter", zap.Uint64("video_id", vid), zap.Uint64("chapter_id", ch.ID))
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": ch})
}

// AdminDeleteVideoChapter removes a chapter from a video.
func (a *API) AdminDeleteVideoChapter(c *gin.Context) {
	vid, _ := strconv.ParseUint(c.Param("id"), 10, 64)
	chID, err := strconv.ParseUint(c.Param("chapterId"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	if err := a.DB.Where("id = ? AND video_id = ?", chID, vid).Delete(&model.VideoChapter{}).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": nil})
}

// ─── Video Bitrates (Module 2: Multi-bitrate) ───

// AdminListVideoBitrates returns all bitrate variants for a video.
func (a *API) AdminListVideoBitrates(c *gin.Context) {
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var bitrates []model.VideoBitrate
	if err := a.DB.Where("video_id = ?", vid).Order("kbps ASC").Find(&bitrates).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": bitrates})
}

// AdminCreateVideoBitrate adds a bitrate variant.
func (a *API) AdminCreateVideoBitrate(c *gin.Context) {
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var req struct {
		Label  string `json:"label" binding:"required"`
		Width  int    `json:"width" binding:"required"`
		Height int    `json:"height" binding:"required"`
		Kbps   int    `json:"kbps" binding:"required"`
		URL    string `json:"url" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	br := model.VideoBitrate{VideoID: vid, Label: req.Label, Width: req.Width, Height: req.Height, Kbps: req.Kbps, URL: req.URL}
	if err := a.DB.Create(&br).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": br})
}

// AdminDeleteVideoBitrate removes a bitrate variant.
func (a *API) AdminDeleteVideoBitrate(c *gin.Context) {
	vid, _ := strconv.ParseUint(c.Param("id"), 10, 64)
	brID, err := strconv.ParseUint(c.Param("bitrateId"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	if err := a.DB.Where("id = ? AND video_id = ?", brID, vid).Delete(&model.VideoBitrate{}).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": nil})
}

// ─── Public endpoints for chapters/bitrates ───

// ListVideoChapters returns chapters for public video player.
func (a *API) ListVideoChapters(c *gin.Context) {
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var chapters []model.VideoChapter
	if err := a.DB.Where("video_id = ?", vid).Order("time_sec ASC").Find(&chapters).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": chapters})
}

// ListVideoBitrates returns bitrates for public video player.
func (a *API) ListVideoBitrates(c *gin.Context) {
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var bitrates []model.VideoBitrate
	if err := a.DB.Where("video_id = ?", vid).Order("kbps ASC").Find(&bitrates).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": bitrates})
}

// ─── Creator-managed chapters (auth, ownership check) ───

// CreatorCreateChapter lets the video uploader add a chapter.
func (a *API) CreatorCreateChapter(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		c.JSON(http.StatusUnauthorized, gin.H{"code": e.CodeUnauthorized, "msg": e.GetMsg(e.CodeUnauthorized), "data": nil})
		return
	}
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	var v model.Video
	if err := a.DB.First(&v, vid).Error; err != nil || v.UserID != uid {
		c.JSON(http.StatusForbidden, gin.H{"code": e.CodeForbidden, "msg": e.GetMsg(e.CodeForbidden), "data": nil})
		return
	}
	var req struct {
		Title   string  `json:"title" binding:"required"`
		TimeSec float64 `json:"time_sec" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	ch := model.VideoChapter{VideoID: vid, Title: req.Title, TimeSec: req.TimeSec}
	if err := a.DB.Create(&ch).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": ch})
}

// CreatorDeleteChapter lets the video uploader remove a chapter.
func (a *API) CreatorDeleteChapter(c *gin.Context) {
	uid, ok := middleware.UserID(c)
	if !ok {
		c.JSON(http.StatusUnauthorized, gin.H{"code": e.CodeUnauthorized, "msg": e.GetMsg(e.CodeUnauthorized), "data": nil})
		return
	}
	vid, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	chID, err := strconv.ParseUint(c.Param("chapterId"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": e.CodeParamError, "msg": e.GetMsg(e.CodeParamError), "data": nil})
		return
	}
	// Verify ownership via video
	var v model.Video
	if err := a.DB.First(&v, vid).Error; err != nil || v.UserID != uid {
		c.JSON(http.StatusForbidden, gin.H{"code": e.CodeForbidden, "msg": e.GetMsg(e.CodeForbidden), "data": nil})
		return
	}
	if err := a.DB.Where("id = ? AND video_id = ?", chID, vid).Delete(&model.VideoChapter{}).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": e.CodeInternalError, "msg": e.GetMsg(e.CodeInternalError), "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": e.CodeSuccess, "msg": e.GetMsg(e.CodeSuccess), "data": nil})
}
