package handler

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"minibili/internal/errcode"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// zoneRow is a single zone with its video count.
type zoneRow struct {
	Name       string `json:"name"`
	VideoCount int64  `json:"video_count"`
}

// ListZones returns all video zones (from videos.zone) with counts, ordered by count DESC.
// GET /api/v1/zones
func (a *API) ListZones(c *gin.Context) {
	var rows []struct {
		Zone string
		Cnt  int64
	}
	if err := a.DB.Model(&model.Video{}).
		Select("zone, COUNT(*) AS cnt").
		Where("zone <> '' AND zone IS NOT NULL").
		Group("zone").
		Order("cnt DESC").
		Scan(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	out := make([]zoneRow, 0, len(rows))
	for _, r := range rows {
		out = append(out, zoneRow{Name: r.Zone, VideoCount: r.Cnt})
	}
	resp.OK(c, gin.H{"items": out, "total": len(out)})
}
