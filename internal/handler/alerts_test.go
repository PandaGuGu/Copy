package handler

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"

	"minibili/internal/model"
)

func TestEvalEnabledAlertRules_FiresAndPersists(t *testing.T) {
	api, _, _ := newTestAPI(t)

	// Simulate queue backlog (pending/retrying tasks) so queue_depth > 0.
	require.NoError(t, api.DB.Create(&model.TaskLog{TaskType: "transcode", TargetID: 1, Status: "pending"}).Error)
	require.NoError(t, api.DB.Create(&model.TaskLog{TaskType: "transcode", TargetID: 2, Status: "retrying"}).Error)

	// One enabled rule that will fire; one disabled rule that would fire if enabled.
	require.NoError(t, api.DB.Create(&model.AlertRule{
		Name: "queue-backlog", Metric: "queue_depth", Threshold: 0, Operator: ">",
		DurationSec: 0, Channel: "log", Enabled: true,
	}).Error)
	disabled := model.AlertRule{
		Name: "disabled-rule", Metric: "queue_depth", Threshold: 0, Operator: ">",
		DurationSec: 0, Channel: "log", Enabled: true,
	}
	require.NoError(t, api.DB.Create(&disabled).Error)
	// Explicitly flip Enabled off (GORM skips zero-value bool on create).
	require.NoError(t, api.DB.Model(&model.AlertRule{}).Where("id = ?", disabled.ID).Update("enabled", false).Error)

	firedHooks := 0
	checked, fired := api.evalEnabledAlertRules(context.Background(), func(model.AlertRule, float64) {
		firedHooks++
	})

	require.Equal(t, 1, checked, "only the enabled rule should be checked")
	require.Equal(t, 1, fired)
	require.Equal(t, 1, firedHooks, "onFire must be called once")

	// One firing AlertRecord persisted.
	var recs int64
	api.DB.Model(&model.AlertRecord{}).Where("status = ?", "firing").Count(&recs)
	require.Equal(t, int64(1), recs)
}

func TestEvalEnabledAlertRules_DurationGate(t *testing.T) {
	api, _, _ := newTestAPI(t)
	require.NoError(t, api.DB.Create(&model.TaskLog{TaskType: "transcode", TargetID: 1, Status: "pending"}).Error)

	// DurationSec>0 with no prior record → placeholder only, no fire.
	require.NoError(t, api.DB.Create(&model.AlertRule{
		Name: "sustained", Metric: "queue_depth", Threshold: 0, Operator: ">",
		DurationSec: 600, Channel: "log", Enabled: true,
	}).Error)

	checked, fired := api.evalEnabledAlertRules(context.Background(), nil)
	require.Equal(t, 1, checked)
	require.Equal(t, 0, fired, "first in-window evaluation must not fire (duration gate)")

	// The placeholder record is present.
	var recs int64
	api.DB.Model(&model.AlertRecord{}).Where("status = 'firing'").Count(&recs)
	require.Equal(t, int64(1), recs)
}

func TestEvaluateCond(t *testing.T) {
	cases := []struct {
		val, thr float64
		op       string
		expected bool
	}{
		{5, 3, ">", true},
		{3, 5, ">", false},
		{3, 3, ">=", true},
		{3, 3, "==", true},
		{3, 3, "eq", true},
		{2, 3, "<", true},
		{2, 3, "lt", true},
		{3, 3, "<=", true},
	}
	for _, c := range cases {
		if got := evaluateCond(c.val, c.op, c.thr); got != c.expected {
			t.Errorf("evaluateCond(%.1f %s %.1f) = %v, want %v", c.val, c.op, c.thr, got, c.expected)
		}
	}
}
