package handler

import (
	"net/http"
	"sort"

	"github.com/gin-gonic/gin"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// maxFeaturedRooms caps the featured set (1 big + 5 small, but allow headroom).
const maxFeaturedRooms = 12

// featuredRoomRow joins a featured entry with its live room, ordered by sort_order.
func (a *API) featuredRooms(includeIDs []uint64, onlyLive bool) ([]gin.H, error) {
	var feats []model.LiveFeaturedRoom
	q := a.DB
	if len(includeIDs) == 0 {
		q = a.DB.Order("sort_order ASC, id ASC")
	} else {
		q = a.DB.Where("room_id IN ?", includeIDs)
	}
	if err := q.Find(&feats).Error; err != nil {
		return nil, err
	}
	sort.SliceStable(feats, func(i, j int) bool {
		return feats[i].SortOrder < feats[j].SortOrder
	})

	byID := make(map[uint64]*model.LiveRoom, len(feats))
	ids := make([]uint64, 0, len(feats))
	for i := range feats {
		ids = append(ids, feats[i].RoomID)
		byID[feats[i].RoomID] = nil
	}
	if len(ids) > 0 {
		var rooms []model.LiveRoom
		if err := a.DB.Where("id IN ?", ids).Find(&rooms).Error; err != nil {
			return nil, err
		}
		for i := range rooms {
			byID[rooms[i].ID] = &rooms[i]
		}
	}

	out := make([]gin.H, 0, len(feats))
	for _, f := range feats {
		r := byID[f.RoomID]
		if r == nil {
			continue
		}
		if onlyLive && r.Status != "live" {
			continue
		}
		out = append(out, gin.H{
			"room_id":      r.ID,
			"title":        r.Title,
			"cover_url":    r.CoverURL,
			"user_id":      r.UserID,
			"avatar_url":   r.AvatarURL,
			"viewer_count": r.ViewerCount,
			"status":       r.Status,
			"sort_order":   f.SortOrder,
		})
	}
	return out, nil
}

// AdminListLiveFeatured GET /admin/live/featured — list the pinned rooms (+ info).
func (a *API) AdminListLiveFeatured(c *gin.Context) {
	rows, err := a.featuredRooms(nil, false)
	if err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, gin.H{"items": rows})
}

// AdminSetLiveFeatured POST /admin/live/featured — replace the ordered pin set.
// Body: {"room_ids":[3,7,1,...]} in display order (index 0 = big slot).
func (a *API) AdminSetLiveFeatured(c *gin.Context) {
	var req struct {
		RoomIDs []uint64 `json:"room_ids"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if len(req.RoomIDs) > maxFeaturedRooms {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	// Validate dedupe + existence.
	seen := make(map[uint64]bool, len(req.RoomIDs))
	for _, id := range req.RoomIDs {
		if id == 0 {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		if seen[id] {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
		seen[id] = true
		var exists int64
		if err := a.DB.Model(&model.LiveRoom{}).Where("id = ?", id).Count(&exists).Error; err != nil || exists == 0 {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
	}

	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Where("1 = 1").Delete(&model.LiveFeaturedRoom{}).Error; err != nil {
			return err
		}
		for i, id := range req.RoomIDs {
			f := model.LiveFeaturedRoom{RoomID: id, SortOrder: i}
			if err := tx.Create(&f).Error; err != nil {
				return err
			}
		}
		return nil
	}); err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{"count": len(req.RoomIDs)})
}

// GetLiveFeatured GET /api/v1/live/featured — public featured live rooms (live only).
func (a *API) GetLiveFeatured(c *gin.Context) {
	rows, err := a.featuredRooms(nil, true)
	if err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	resp.OK(c, rows)
}
