package worker

import (
	"testing"

	"github.com/glebarez/sqlite"
	"github.com/stretchr/testify/require"
	"gorm.io/gorm"

	"minibili/internal/model"
)

func newWorkerTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	require.NoError(t, err)
	require.NoError(t, db.AutoMigrate(&model.VideoBitrate{}))
	return db
}

func TestReplaceVideoBitrates_Sync(t *testing.T) {
	db := newWorkerTestDB(t)

	// First write: master "原始" + 720p/480p/360p
	first := []model.VideoBitrate{
		{VideoID: 7, Label: "原始", Height: 1080, URL: "https://x/source"},
		{VideoID: 7, Label: "720p", Height: 720, URL: "https://x/720"},
		{VideoID: 7, Label: "360p", Height: 360, URL: "https://x/360"},
	}
	require.NoError(t, replaceVideoBitrates(db, 7, first))

	var count int64
	db.Model(&model.VideoBitrate{}).Where("video_id = 7").Count(&count)
	require.Equal(t, int64(3), count)

	// Second write replaces, not appends.
	second := []model.VideoBitrate{
		{VideoID: 7, Label: "原始", Height: 1080, URL: "https://x/source"},
		{VideoID: 7, Label: "480p", Height: 480, URL: "https://x/480"},
	}
	require.NoError(t, replaceVideoBitrates(db, 7, second))

	var rows []model.VideoBitrate
	db.Where("video_id = 7").Order("height DESC").Find(&rows)
	require.Len(t, rows, 2)
	require.Equal(t, "原始", rows[0].Label)
	require.Equal(t, "480p", rows[1].Label)

	// Other videos untouched
	db.Model(&model.VideoBitrate{}).Create(&model.VideoBitrate{VideoID: 8, Label: "原始", URL: "https://x/8"})
	require.NoError(t, replaceVideoBitrates(db, 7, nil))
	var v8 int64
	db.Model(&model.VideoBitrate{}).Where("video_id = 8").Count(&v8)
	require.Equal(t, int64(1), v8, "unrelated video bitrates must survive")
}
