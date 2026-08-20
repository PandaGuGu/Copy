package data

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"

	"gorm.io/gorm"
)

// Versioned migration runner (P0-3, ADR-002 revisit trigger).
//
// Strategy — "ammunition ready, not hard switch":
//   - AutoMigrate remains the primary path (S-002, contains ~20 idempotent
//     backfills). Default behaviour is unchanged.
//   - When DB_MIGRATE_TOOL=golang-migrate (or =sql), the server runs the
//     versioned migrations in ./migrations at startup, AFTER AutoMigrate,
//     so both layers stay consistent.
//   - Migration files follow golang-migrate naming: NNNNNN_name.up.sql /
//     NNNNNN_name.down.sql. A schema_migrations table tracks applied versions,
//     compatible with golang-migrate's own bookkeeping table.
//
// Switching fully to golang-migrate later (dropping AutoMigrate) is a
// documented follow-up; the files here are directly consumable by the
// golang-migrate CLI/library without conversion.

// migrationDir resolves the migrations directory (project root/migrations).
// Overridable via MIGRATIONS_DIR for tests or non-standard layouts.
func migrationDir() string {
	if d := os.Getenv("MIGRATIONS_DIR"); d != "" {
		return d
	}
	// Working dir is the project root (cmd/mini-bili runs from root).
	return "migrations"
}

// RunVersionedMigrations applies pending *.up.sql migrations in order.
// No-op when the tool is not enabled or no migration files exist.
func RunVersionedMigrations(db *gorm.DB, enabled bool) error {
	if !enabled {
		return nil
	}

	// Create bookkeeping table if missing.
	if err := db.Exec(`CREATE TABLE IF NOT EXISTS schema_migrations (
		version bigint NOT NULL PRIMARY KEY,
		dirty boolean NOT NULL DEFAULT false
	) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci`).Error; err != nil {
		return fmt.Errorf("create schema_migrations: %w", err)
	}

	dir := migrationDir()
	entries, err := filepath.Glob(filepath.Join(dir, "*.up.sql"))
	if err != nil {
		return fmt.Errorf("list migrations: %w", err)
	}
	sort.Strings(entries)
	if len(entries) == 0 {
		return nil
	}

	for _, name := range entries {
		// Parse version from "000001_name.up.sql".
		base := filepath.Base(name)
		versionStr := strings.SplitN(base, "_", 2)[0]
		version, err := strconv.ParseInt(versionStr, 10, 64)
		if err != nil || version == 0 {
			continue
		}

		var cnt int64
		if err := db.Raw("SELECT COUNT(*) FROM schema_migrations WHERE version = ?", version).Scan(&cnt).Error; err != nil {
			return err
		}
		if cnt > 0 {
			continue // already applied
		}

		sqlBytes, err := os.ReadFile(name)
		if err != nil {
			return fmt.Errorf("read %s: %w", name, err)
		}

		tx := db.Begin()
		if err := tx.Exec(string(sqlBytes)).Error; err != nil {
			tx.Rollback()
			return fmt.Errorf("apply %s: %w", name, err)
		}
		if err := tx.Exec("INSERT INTO schema_migrations (version, dirty) VALUES (?, false)", version).Error; err != nil {
			tx.Rollback()
			return fmt.Errorf("record %s: %w", name, err)
		}
		if err := tx.Commit().Error; err != nil {
			return fmt.Errorf("commit %s: %w", name, err)
		}
	}
	return nil
}

// MigrateToolEnabled reports whether versioned migrations are enabled via env.
func MigrateToolEnabled() bool {
	tool := strings.ToLower(os.Getenv("DB_MIGRATE_TOOL"))
	return tool == "golang-migrate" || tool == "sql"
}
