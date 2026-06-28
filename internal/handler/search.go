package handler

import (
	"context"
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
	"minibili/internal/search"
)

// SearchAll implements GET /api/v1/search for the bilibili-vue search page.
func (a *API) SearchAll(c *gin.Context) {
	keyword := strings.TrimSpace(c.Query("keyword"))
	if err := search.ValidateKeyword(keyword); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var viewer uint64
	if uid, ok := middleware.UserID(c); ok {
		viewer = uid
	}
	if a.SearchHot != nil {
		recCtx, recCancel := context.WithTimeout(c.Request.Context(), 500*time.Millisecond)
		if err := a.SearchHot.Record(recCtx, viewer, c.ClientIP(), keyword); err != nil {
			a.Log.Warn("record search hot", zap.Error(err), zap.String("keyword", keyword))
		}
		recCancel()
	}
	if a.ES == nil || !a.ES.Enabled() {
		// MySQL fallback: multi-field search with filters (order/duration/zone).
		limit, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
		if limit <= 0 || limit > 50 {
			limit = 20
		}
		order := c.DefaultQuery("order", "default")
		duration := c.DefaultQuery("duration", "all")
		zone := strings.TrimSpace(c.Query("zone"))

		seen := make(map[uint64]bool)
		var videos []model.Video
		ctx := c.Request.Context()

		// 1. Exact ID / BV match (respects zone + duration filters).
		id := parseBVOrNumeric(keyword)
		if id > 0 {
			var v model.Video
			q := a.DB.WithContext(ctx).Where("id = ? AND status = ?", id, "published")
			if zone != "" {
				q = q.Where("zone = ? OR zone LIKE ?", zone, zone+"-%")
			}
			switch duration {
			case "lt10":
				q = q.Where("duration_sec < 600")
			case "m10_30":
				q = q.Where("duration_sec >= 600 AND duration_sec <= 1800")
			case "m30_60":
				q = q.Where("duration_sec > 1800 AND duration_sec <= 3600")
			case "gt60":
				q = q.Where("duration_sec > 3600")
			}
			if q.First(&v).Error == nil {
				videos = append(videos, v)
				seen[id] = true
			}
		}

		// 2. Multi-field LIKE with sorting and filters.
		like := fmt.Sprintf("%%%s%%", keyword)
		query := a.DB.WithContext(ctx).
			Where("status = ? AND (title LIKE ? OR description LIKE ? OR tags_json LIKE ?)",
				"published", like, like, like)

		// Zone filter.
		if zone != "" {
			query = query.Where("zone = ? OR zone LIKE ?", zone, zone+"-%")
		}

		// Duration filter.
		switch duration {
		case "lt10":
			query = query.Where("duration_sec < 600")
		case "m10_30":
			query = query.Where("duration_sec >= 600 AND duration_sec <= 1800")
		case "m30_60":
			query = query.Where("duration_sec > 1800 AND duration_sec <= 3600")
		case "gt60":
			query = query.Where("duration_sec > 3600")
		}

		// Sort order.
		switch order {
		case "click":
			query = query.Order("play_count DESC, created_at DESC")
		case "pubdate":
			query = query.Order("created_at DESC, play_count DESC")
		case "dm":
			query = query.Order("danmaku_count DESC, created_at DESC")
		case "fav":
			query = query.Order("fav_count DESC, created_at DESC")
		default: // "default" / 综合排序
			query = query.Order("play_count DESC, created_at DESC")
		}

		var more []model.Video
		query.Limit(limit).Find(&more)
		for i := range more {
			if !seen[more[i].ID] {
				videos = append(videos, more[i])
				seen[more[i].ID] = true
			}
		}

		// Trim.
		if len(videos) > limit {
			videos = videos[:limit]
		}

		out := &search.AllResult{
			Result: search.SearchResultBuckets{
				Video:    make([]search.VideoHit, 0, len(videos)),
				Article:  []search.ArticleHit{},
				BiliUser: []search.UserHit{},
			},
		}
		for _, v := range videos {
			out.Result.Video = append(out.Result.Video, search.VideoHit{
				Aid:         v.ID,
				Title:       v.Title,
				Pic:         v.CoverURL,
				Description: v.Description,
				Play:        v.PlayCount,
				VideoReview: v.DanmakuCount,
				Author:      "",
				Mid:         v.UserID,
				Duration:    fmtDuration(v.DurationSec),
				Pubdate:     v.CreatedAt.Unix(),
			})
		}
		if len(out.Result.Video) == 0 {
			out.SearchStatus = "empty"
		} else {
			out.SearchStatus = "ok"
		}
		resp.OK(c, out)
		return
	}
	highlight := c.Query("highlight") == "1" || strings.EqualFold(c.Query("highlight"), "true")
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	searchType := strings.TrimSpace(c.DefaultQuery("type", "all"))
	sort := strings.TrimSpace(c.Query("sort"))
	videoFilter := search.ParseVideoFilter(
		c.DefaultQuery("order", c.Query("video_order")),
		c.DefaultQuery("duration", ""),
		c.DefaultQuery("zone", ""),
	)

	ctx, cancel := context.WithTimeout(c.Request.Context(), 10*time.Second)
	defer cancel()
	out, err := a.ES.SearchAll(ctx, search.SearchParams{
		Keyword:   keyword,
		Highlight: highlight,
		Page:      page,
		PageSize:  pageSize,
		Type:      searchType,
		Sort:      sort,
		Video:     videoFilter,
	})
	if err != nil {
		a.Log.Error("search all", zap.Error(err), zap.String("keyword", keyword))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	if len(out.Result.BiliUser) > 0 {
		out.Result.BiliUser = search.EnrichUserHits(a.DB, viewer, out.Result.BiliUser)
	}
	if len(out.Result.Video) > 0 && viewer > 0 {
		ids := make([]uint64, 0, len(out.Result.Video))
		for _, v := range out.Result.Video {
			if v.Aid > 0 {
				ids = append(ids, v.Aid)
			}
		}
		later := watchLaterByViewer(a.DB, viewer, ids)
		for i := range out.Result.Video {
			out.Result.Video[i].InWatchLater = later[out.Result.Video[i].Aid]
		}
	}
	if out.SearchStatus == "" {
		if searchResultEmpty(out) {
			out.SearchStatus = "empty"
		} else {
			out.SearchStatus = "ok"
		}
	}
	resp.OK(c, out)
}

func searchResultEmpty(out *search.AllResult) bool {
	if out == nil {
		return true
	}
	r := out.Result
	return len(r.Video) == 0 &&
		len(r.Article) == 0 &&
		len(r.BiliUser) == 0 &&
		len(r.MediaBangumi) == 0 &&
		len(r.MediaFt) == 0 &&
		len(r.Live) == 0 &&
		len(r.Topic) == 0 &&
		len(r.Photo) == 0
}

func emptySearchResult() *search.AllResult {
	return &search.AllResult{
		Result: search.SearchResultBuckets{
			Video:        []search.VideoHit{},
			Article:      []search.ArticleHit{},
			BiliUser:     []search.UserHit{},
			MediaBangumi: []any{},
			MediaFt:      []any{},
			Live:         []any{},
			Topic:        []any{},
			Photo:        []any{},
		},
		TopTlist:     search.TopTlist{},
		SearchStatus: "empty",
	}
}

func fmtDuration(secs float64) string {
	m := int(secs) / 60
	s := int(secs) % 60
	return fmt.Sprintf("%d:%02d", m, s)
}

func parseBVOrNumeric(s string) uint64 {
	s = strings.TrimSpace(s)
	if s == "" {
		return 0
	}
	// Strip "BV"/"bv"/"AV"/"av" prefix.
	if len(s) > 2 && (s[:2] == "BV" || s[:2] == "bv" || s[:2] == "AV" || s[:2] == "av") {
		s = s[2:]
	}
	n, err := strconv.ParseUint(s, 10, 64)
	if err != nil {
		return 0
	}
	return n
}
