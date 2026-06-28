package handler

import (
	"fmt"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ─── Special Pages (Module 9) ───

// AdminListSpecialPages returns all special pages.
func (a *API) AdminListSpecialPages(c *gin.Context) {
	var pages []model.SpecialPage
	if err := a.DB.Order("updated_at DESC").Find(&pages).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "查询失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": pages})
}

// AdminCreateSpecialPage creates a new special page.
func (a *API) AdminCreateSpecialPage(c *gin.Context) {
	var req model.SpecialPage
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "参数错误", "data": nil})
		return
	}
	if req.Title == "" || req.Slug == "" {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "标题和标识不能为空", "data": nil})
		return
	}
	if req.Status == "" {
		req.Status = "draft"
	}
	if err := a.DB.Create(&req).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "创建失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": req})
}

// AdminUpdateSpecialPage updates a special page.
func (a *API) AdminUpdateSpecialPage(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "参数错误", "data": nil})
		return
	}
	var page model.SpecialPage
	if err := a.DB.First(&page, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"code": 404, "msg": "不存在", "data": nil})
		return
	}
	var req model.SpecialPage
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "参数错误", "data": nil})
		return
	}
	updates := map[string]interface{}{
		"title":       req.Title,
		"slug":        req.Slug,
		"cover_url":   req.CoverURL,
		"description": req.Description,
		"blocks":      req.Blocks,
		"status":      req.Status,
	}
	if err := a.DB.Model(&page).Updates(updates).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "更新失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": page})
}

// AdminDeleteSpecialPage deletes a special page.
func (a *API) AdminDeleteSpecialPage(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "参数错误", "data": nil})
		return
	}
	if err := a.DB.Delete(&model.SpecialPage{}, id).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "删除失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": nil})
}

// ─── Campaigns ───

// AdminListCampaigns returns all campaigns.
func (a *API) AdminListCampaigns(c *gin.Context) {
	var list []model.Campaign
	if err := a.DB.Order("updated_at DESC").Find(&list).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "查询失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": list})
}

// AdminCreateCampaign creates a new campaign.
func (a *API) AdminCreateCampaign(c *gin.Context) {
	var req model.Campaign
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "参数错误", "data": nil})
		return
	}
	if req.Title == "" || req.Slug == "" {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "标题和标识不能为空", "data": nil})
		return
	}
	if req.Status == "" {
		req.Status = "draft"
	}
	if err := a.DB.Create(&req).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "创建失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": req})
}

// AdminUpdateCampaign updates a campaign.
func (a *API) AdminUpdateCampaign(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "参数错误", "data": nil})
		return
	}
	var cg model.Campaign
	if err := a.DB.First(&cg, id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"code": 404, "msg": "不存在", "data": nil})
		return
	}
	var req model.Campaign
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "参数错误", "data": nil})
		return
	}
	updates := map[string]interface{}{
		"title":       req.Title,
		"slug":        req.Slug,
		"cover_url":   req.CoverURL,
		"description": req.Description,
		"rules":       req.Rules,
		"rewards":     req.Rewards,
		"start_time":  req.StartTime,
		"end_time":    req.EndTime,
		"status":      req.Status,
	}
	if err := a.DB.Model(&cg).Updates(updates).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "更新失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": cg})
}

// AdminDeleteCampaign deletes a campaign.
func (a *API) AdminDeleteCampaign(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"code": 400, "msg": "参数错误", "data": nil})
		return
	}
	if err := a.DB.Delete(&model.Campaign{}, id).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "删除失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": nil})
}

// ─── Public endpoints ───

// GetPublicSpecialPage returns a published special page by slug.
func (a *API) GetPublicSpecialPage(c *gin.Context) {
	slug := c.Param("slug")
	var page model.SpecialPage
	if err := a.DB.Where("slug = ? AND status = ?", slug, "published").First(&page).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"code": 404, "msg": "页面不存在", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": page})
}

// ListPublicSpecialPages returns all published special pages.
func (a *API) ListPublicSpecialPages(c *gin.Context) {
	var pages []model.SpecialPage
	if err := a.DB.Where("status = ?", "published").Order("updated_at DESC").Find(&pages).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"code": 500, "msg": "查询失败", "data": nil})
		return
	}
	c.JSON(http.StatusOK, gin.H{"code": 0, "msg": "ok", "data": pages})
}

// AdminUploadSpecialCover POST /api/v1/admin/specials/upload-cover — multipart field "image".
func (a *API) AdminUploadSpecialCover(c *gin.Context) {
	if err := c.Request.ParseMultipartForm(12 << 20); err != nil {
		resp.Err(c, http.StatusBadRequest, 400)
		return
	}
	fh, err := c.FormFile("image")
	if err != nil {
		resp.Err(c, http.StatusBadRequest, 400)
		return
	}
	key := fmt.Sprintf("special-covers/%s.jpg", uuid.NewString())
	url, code := a.uploadBannerImageToOSS(fh, key)
	if code != 0 {
		resp.Err(c, http.StatusBadRequest, code)
		return
	}
	resp.OK(c, gin.H{"cover_url": url})
}
