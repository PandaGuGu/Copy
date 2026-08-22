package worker

import (
	"context"
	"os"
	"path/filepath"
	"time"

	"github.com/redis/go-redis/v9"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/config"
	"minibili/internal/model"
	"minibili/internal/service"
)

// StartScheduler runs periodic background tasks with TaskLog tracking.
// These are lightweight cron-like jobs that don't depend on RabbitMQ.
func StartScheduler(ctx context.Context, cfg *config.C, db *gorm.DB, rdb *redis.Client, log *zap.Logger) {
	log.Info("scheduler started")

	// ── 30min: health self-check ──
	go runEvery(ctx, 30*time.Minute, func() {
		scheduleHealthCheck(db, rdb, log)
	})

	// ── 1h: user & content stats ──
	go runEvery(ctx, 1*time.Hour, func() {
		scheduleUserStats(db, log)
	})

	// ── 24h: temp file cleanup ──
	go runEvery(ctx, 24*time.Hour, func() {
		scheduleCleanupTemp(cfg, db, log)
	})

	// ── 24h: ItemCF offline similarity recompute (SPEC F17, NFR-REC-2) ──
	go runEvery(ctx, 24*time.Hour, func() {
		scheduleItemCF(db, log)
	})

	// ── 1min: time-driven state transitions (ADR-018) ──
	// ① scheduled publish consumer (PublishAt due → draft → transcode queue)
	// ② ticket SLA deadline escalation (sla_deadline overdue → escalate/notify)
	// ③ expired ban auto-unban (via state machine)
	go runEvery(ctx, 1*time.Minute, func() {
		scheduleScheduledPublishes(db, log)
		scheduleSLAEscalation(db, log)
		scheduleAutoUnban(db, log)
		scheduleExpireUserCapabilities(db, log)
	})
}

func runEvery(ctx context.Context, interval time.Duration, fn func()) {
	fn() // run once at startup
	t := time.NewTicker(interval)
	defer t.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-t.C:
			fn()
		}
	}
}

// ── health_check ──

func scheduleHealthCheck(db *gorm.DB, rdb *redis.Client, log *zap.Logger) {
	now := time.Now()
	task := model.TaskLog{TaskType: "health_check", Status: "running", StartedAt: &now}
	db.Create(&task)

	var errMsg string
	if sqlDB, err := db.DB(); err != nil {
		errMsg = "db handle: " + err.Error()
	} else if err := sqlDB.Ping(); err != nil {
		errMsg = "db ping: " + err.Error()
	} else if rdb == nil {
		errMsg = "redis not configured"
	} else if err := rdb.Ping(context.Background()).Err(); err != nil {
		errMsg = "redis ping: " + err.Error()
	}

	stmt := map[string]interface{}{"finished_at": time.Now()}
	if errMsg != "" {
		stmt["status"] = "failed"
		stmt["error_msg"] = errMsg
		log.Warn("health_check failed", zap.String("error", errMsg))
	} else {
		stmt["status"] = "success"
	}
	db.Model(&model.TaskLog{}).Where("id = ?", task.ID).Updates(stmt)
}

// ── user_stats ──

func scheduleUserStats(db *gorm.DB, log *zap.Logger) {
	now := time.Now()
	task := model.TaskLog{TaskType: "user_stats", Status: "running", StartedAt: &now}
	db.Create(&task)

	var totalUsers, totalVideos, totalComments int64
	hasErr := false
	if err := db.Model(&model.User{}).Count(&totalUsers).Error; err != nil {
		log.Warn("user_stats count users", zap.Error(err))
		hasErr = true
	}
	if err := db.Model(&model.Video{}).Count(&totalVideos).Error; err != nil {
		log.Warn("user_stats count videos", zap.Error(err))
		hasErr = true
	}
	if err := db.Model(&model.Comment{}).Count(&totalComments).Error; err != nil {
		log.Warn("user_stats count comments", zap.Error(err))
		hasErr = true
	}

	stmt := map[string]interface{}{"finished_at": time.Now()}
	if hasErr {
		stmt["status"] = "failed"
		stmt["error_msg"] = "partial count failure"
	} else {
		stmt["status"] = "success"
	}
	db.Model(&model.TaskLog{}).Where("id = ?", task.ID).Updates(stmt)

	log.Info("user_stats collected",
		zap.Int64("users", totalUsers),
		zap.Int64("videos", totalVideos),
		zap.Int64("comments", totalComments),
	)
}

// ── itemcf ──

// scheduleItemCF runs the ItemCF offline similarity recompute,
// tracked in task_logs. Runs at startup and every 24h (see StartScheduler).
func scheduleItemCF(db *gorm.DB, log *zap.Logger) {
	now := time.Now()
	task := model.TaskLog{TaskType: "itemcf", Status: "running", StartedAt: &now}
	db.Create(&task)

	pairs, err := service.ComputeItemCF(db, log)

	stmt := map[string]interface{}{"finished_at": time.Now()}
	if err != nil {
		stmt["status"] = "failed"
		stmt["error_msg"] = err.Error()
		log.Error("itemcf task failed", zap.Error(err))
	} else {
		stmt["status"] = "success"
		log.Info("itemcf task success", zap.Int64("pairs", pairs))
	}
	db.Model(&model.TaskLog{}).Where("id = ?", task.ID).Updates(stmt)
}

// ── cleanup_temp ──

func scheduleCleanupTemp(cfg *config.C, db *gorm.DB, log *zap.Logger) {
	now := time.Now()
	task := model.TaskLog{TaskType: "cleanup_temp", Status: "running", StartedAt: &now}
	db.Create(&task)

	dir := cfg.TempUploadDir
	if dir == "" {
		dir = os.TempDir()
	}

	var cleaned int64
	cutoff := time.Now().Add(-24 * time.Hour)
	entries, err := os.ReadDir(dir)
	if err != nil {
		stmt := map[string]interface{}{"finished_at": time.Now(), "status": "failed", "error_msg": err.Error()}
		db.Model(&model.TaskLog{}).Where("id = ?", task.ID).Updates(stmt)
		log.Warn("cleanup_temp read dir failed", zap.String("dir", dir), zap.Error(err))
		return
	}
	for _, e := range entries {
		path := filepath.Join(dir, e.Name())
		info, err := e.Info()
		if err != nil {
			continue
		}
		if info.ModTime().Before(cutoff) {
			if err := os.RemoveAll(path); err == nil {
				cleaned++
			}
		}
	}

	db.Model(&model.TaskLog{}).Where("id = ?", task.ID).Updates(map[string]interface{}{
		"status": "success", "finished_at": time.Now(),
	})
	log.Info("cleanup_temp completed",
		zap.Int64("files_removed", cleaned),
		zap.String("dir", dir),
	)
}

// ── time-driven state transitions (ADR-018) ──

// scheduleScheduledPublishes consumes due scheduled_publishes rows:
// a video in "draft" whose PublishAt <= now moves to "processing" (draft is
// released to the transcode pipeline). Previously this feature had NO consumer.
func scheduleScheduledPublishes(db *gorm.DB, log *zap.Logger) {
	now := time.Now()
	var due []model.ScheduledPublish
	if err := db.Where("published = ? AND publish_at <= ?", false, now).Find(&due).Error; err != nil {
		log.Warn("scheduled publish scan", zap.Error(err))
		return
	}
	if len(due) == 0 {
		return
	}
	for _, sp := range due {
		var v model.Video
		if err := db.First(&v, sp.VideoID).Error; err != nil {
			log.Warn("scheduled publish video missing", zap.Uint64("video_id", sp.VideoID))
			continue
		}
		if v.Status != "draft" {
			// Already beyond draft (e.g. processing/published) — mark done.
			db.Model(&sp).Update("published", true)
			continue
		}
		// State machine: draft → processing (release to transcode).
		if !service.VideoMachine.Can("draft", "processing") {
			log.Warn("scheduled publish illegal transition", zap.Uint64("video_id", sp.VideoID), zap.String("status", v.Status))
			continue
		}
		if err := db.Model(&v).Updates(map[string]interface{}{
			"status":         "processing",
			"draft_raw_path": "",
		}).Error; err != nil {
			log.Warn("scheduled publish update failed", zap.Uint64("video_id", sp.VideoID), zap.Error(err))
			continue
		}
		db.Model(&sp).Update("published", true)
		log.Info("scheduled publish executed", zap.Uint64("video_id", sp.VideoID), zap.Time("publish_at", sp.PublishAt))
	}
}

// scheduleSLAEscalation escalates tickets whose sla_deadline is overdue.
// Uses the SLA deadline field (not updated_at) — the previous main.go worker
// was decoupled from sla_deadline. Escalation: assigned/processing overdue →
// priority=urgent; no hard auto-close (human review preferred).
func scheduleSLAEscalation(db *gorm.DB, log *zap.Logger) {
	res := db.Model(&model.Ticket{}).
		Where("sla_deadline IS NOT NULL AND sla_deadline < ? AND status IN ('assigned','processing') AND priority != 'urgent'", time.Now()).
		Update("priority", "urgent")
	if res.Error != nil {
		log.Warn("sla escalation", zap.Error(res.Error))
	} else if res.RowsAffected > 0 {
		log.Info("sla escalated tickets to urgent", zap.Int64("count", res.RowsAffected))
	}
}

// scheduleAutoUnban unblocks users whose ban_expires_at has passed,
// via the user state machine (banned → active).
func scheduleAutoUnban(db *gorm.DB, log *zap.Logger) {
	var banned []model.User
	if err := db.Where("status = 'banned' AND ban_expires_at IS NOT NULL AND ban_expires_at <= ?", time.Now()).
		Find(&banned).Error; err != nil {
		log.Warn("auto unban scan", zap.Error(err))
		return
	}
	for _, u := range banned {
		if !service.UserMachine.Can("banned", "active") {
			continue
		}
		if err := db.Model(&u).Updates(map[string]interface{}{
			"status": "active", "banned_at": nil, "banned_reason": "",
		}).Error; err != nil {
			log.Warn("auto unban update", zap.Uint64("user_id", u.ID), zap.Error(err))
			continue
		}
		log.Info("auto unban user", zap.Uint64("user_id", u.ID))
	}
}

// scheduleExpireUserCapabilities lifts capability restrictions whose expires_at
// passed (能力级限制到期自动恢复). Kept with status restored for audit.
func scheduleExpireUserCapabilities(db *gorm.DB, log *zap.Logger) {
	rows, err := db.Where("status = 'active' AND expires_at IS NOT NULL AND expires_at <= ?", time.Now()).
		Find(&model.UserCapabilityRestriction{}).Rows()
	if err != nil {
		log.Warn("expire user capabilities scan", zap.Error(err))
		return
	}
	defer rows.Close()

	for rows.Next() {
		var r model.UserCapabilityRestriction
		if err := db.ScanRows(rows, &r); err != nil {
			continue
		}
		now := time.Now()
		if db.Model(&r).Updates(map[string]interface{}{
			"status": "restored", "restored_at": &now,
		}).Error == nil {
			log.Info("expire user capability",
				zap.Uint64("user_id", r.UserID), zap.String("capability", r.Capability))
		}
	}
}
