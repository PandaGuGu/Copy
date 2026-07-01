package data

import (
	"fmt"
	"os"
	"strings"

	"go.uber.org/zap"
	"gorm.io/driver/mysql"
	"gorm.io/gorm"
	gormlogger "gorm.io/gorm/logger"
)

// NewDB opens MySQL and optionally runs AutoMigrate (Skill S-002).
// AutoMigrate is automatically skipped when APP_ENV is not "development".
func NewDB(dsn string, lg *zap.Logger, appEnv string) (*gorm.DB, error) {
	if dsn == "" {
		return nil, fmt.Errorf("MYSQL_DSN is empty")
	}
	db, err := gorm.Open(mysql.Open(dsn), &gorm.Config{
		Logger: gormlogger.Default.LogMode(gormlogger.Warn),
	})
	if err != nil {
		return nil, err
	}

	// SAFETY: AutoMigrate modifies schema automatically — only safe for local
	// development. In staging/production, use explicit versioned migrations
	// (e.g. golang-migrate / goose) instead.
	// Set FORCE_AUTO_MIGRATE=true to run it anyway (e.g. for free-tier deploys).
	if strings.ToLower(strings.TrimSpace(appEnv)) == "development" ||
		strings.ToLower(strings.TrimSpace(os.Getenv("FORCE_AUTO_MIGRATE"))) == "true" {
		if err := AutoMigrateAll(db, lg); err != nil {
			return nil, err
		}
	} else {
		lg.Info("skipping AutoMigrate: APP_ENV is not development; use explicit migrations for this environment",
			zap.String("app_env", appEnv),
		)
	}
	return db, nil
}
