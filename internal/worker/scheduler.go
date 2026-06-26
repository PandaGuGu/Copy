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
