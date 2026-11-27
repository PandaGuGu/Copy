package data

import (
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	"go.uber.org/zap"
	"gorm.io/driver/mysql"
	"gorm.io/gorm"
	gormlogger "gorm.io/gorm/logger"
)

// Connection-pool defaults. Go's sql.DB leaves MaxIdleConns at 2, which forces
// a fresh TCP handshake + MySQL auth for almost every request under bursty
// load — the dominant cause of multi-second p99 tails on otherwise trivial
// endpoints. Keep MaxOpen well below MySQL's default max_connections (151) so
// several app replicas can coexist.
const (
	defaultDBMaxOpen = 50
	defaultDBMaxIdle = 25
)

var (
	defaultDBConnLifetime = 30 * time.Minute
	defaultDBConnIdleTime = 5 * time.Minute
)

func envInt(name string, def int) int {
	if v := strings.TrimSpace(os.Getenv(name)); v != "" {
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			return n
		}
	}
	return def
}

func envDuration(name string, def time.Duration) time.Duration {
	if v := strings.TrimSpace(os.Getenv(name)); v != "" {
		if d, err := time.ParseDuration(v); err == nil && d > 0 {
			return d
		}
	}
	return def
}

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

	// Configure the pool explicitly. Without this, MaxIdleConns stays at 2 and
	// connections are churned on every concurrent burst.
	sqlDB, err := db.DB()
	if err != nil {
		return nil, fmt.Errorf("db handle: %w", err)
	}
	maxOpen := envInt("DB_MAX_OPEN_CONNS", defaultDBMaxOpen)
	maxIdle := envInt("DB_MAX_IDLE_CONNS", defaultDBMaxIdle)
	if maxIdle > maxOpen {
		maxIdle = maxOpen
	}
	lifetime := envDuration("DB_CONN_MAX_LIFETIME", defaultDBConnLifetime)
	idleTime := envDuration("DB_CONN_MAX_IDLE_TIME", defaultDBConnIdleTime)
	sqlDB.SetMaxOpenConns(maxOpen)
	sqlDB.SetMaxIdleConns(maxIdle)
	sqlDB.SetConnMaxLifetime(lifetime)
	sqlDB.SetConnMaxIdleTime(idleTime)
	lg.Info("db pool configured",
		zap.Int("max_open", maxOpen),
		zap.Int("max_idle", maxIdle),
		zap.Duration("conn_max_lifetime", lifetime),
		zap.Duration("conn_max_idle_time", idleTime),
	)

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

	// Versioned migrations (P0-3): runs ./migrations/*.up.sql after AutoMigrate
	// when DB_MIGRATE_TOOL=golang-migrate|sql. Default off — behaviour unchanged.
	if err := RunVersionedMigrations(db, MigrateToolEnabled()); err != nil {
		return nil, fmt.Errorf("versioned migrations: %w", err)
	}
	return db, nil
}
