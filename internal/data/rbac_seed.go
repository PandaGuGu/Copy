package data

import (
	"errors"
	"strings"

	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
	"gorm.io/gorm/clause"

	"minibili/internal/model"
)

// SeedRBAC ensures baseline roles, permissions, and role-permission mappings exist.
// Safe to call on every startup (upsert semantics).
func SeedRBAC(db *gorm.DB, lg *zap.Logger) {
	// ── 1. Upsert all permissions ──
	perms := []model.AdminPermission{
		// 📊 数据
		{Code: "dashboard:view",   Resource: "dashboard", Action: "view"},
		{Code: "dashboard:export", Resource: "dashboard", Action: "export"},

		// 📢 运营
		{Code: "banner:manage",    Resource: "banner",    Action: "manage"},
		{Code: "hotsearch:manage", Resource: "hotsearch", Action: "manage"},
		{Code: "special:manage",   Resource: "special",   Action: "manage"},
		{Code: "dynamic:manage",   Resource: "dynamic",   Action: "manage"},
		{Code: "subtitle:manage",  Resource: "subtitle",  Action: "manage"},

		// 🛡️ 审核 — also used by RequirePermission middleware
		{Code: "video:approve",    Resource: "video",     Action: "approve"},
		{Code: "article:approve",  Resource: "article",   Action: "approve"},
		{Code: "comment:delete",   Resource: "comment",   Action: "delete"},
		{Code: "report:handle",    Resource: "ticket",    Action: "handle"}, // middleware checks ticket.handle
		{Code: "copyright:handle", Resource: "copyright", Action: "handle"},
		{Code: "risk:manage",      Resource: "risk",      Action: "manage"},

		// 👤 用户
		{Code: "user:ban",         Resource: "user",      Action: "ban"},    // middleware checks user.ban
		{Code: "ticket:handle",    Resource: "ticket",    Action: "handle"}, // also covers ticket ops
		{Code: "cs:manage",        Resource: "cs",        Action: "manage"},

		// 🤖 AI
		{Code: "agent:manage",     Resource: "agent",     Action: "manage"},
		{Code: "llm:manage",       Resource: "setting",   Action: "manage"},

		// ⚙️ 系统
		{Code: "setting:manage",   Resource: "setting",   Action: "manage"},
		{Code: "config:manage",    Resource: "config",    Action: "manage"},
		{Code: "ops:manage",       Resource: "ops",       Action: "manage"},
		{Code: "rbac:manage",      Resource: "rbac",      Action: "manage"},
	}

	permIDMap := make(map[string]uint64, len(perms))
	for i := range perms {
		if err := db.Clauses(clause.OnConflict{
			Columns:   []clause.Column{{Name: "code"}},
			DoUpdates: clause.AssignmentColumns([]string{"resource", "action"}),
		}).Create(&perms[i]).Error; err != nil {
			lg.Error("rbac seed: upsert permission failed", zap.Error(err), zap.String("code", perms[i].Code))
			continue
		}
		// Refetch id after upsert
		var p model.AdminPermission
		if db.Where("code = ?", perms[i].Code).First(&p).Error == nil {
			permIDMap[perms[i].Code] = p.ID
		}
	}

	// ── 2. Upsert roles ──
	roleDefs := []struct {
		Name, Desc string
		Perms      []string // permission codes
	}{
		{
			Name: "super_admin", Desc: "超级管理员",
			Perms: allCodes(perms),
		},
		{
			Name: "content_review", Desc: "内容审核",
			Perms: []string{
				// 审核组 — 全部
				"video:approve", "article:approve", "comment:delete",
				"report:handle", "copyright:handle", "risk:manage",
				// 用户组 — 仅封禁（违规用户）
				"user:ban",
				// 数据组 — 只读
				"dashboard:view",
			},
		},
		{
			Name: "cs_admin", Desc: "客服",
			Perms: []string{
				// 用户组 — 全部
				"user:ban", "ticket:handle", "cs:manage",
				// 数据组 — 只读
				"dashboard:view",
			},
		},
	}

	for _, rd := range roleDefs {
		role := model.AdminRole{Name: rd.Name, Description: rd.Desc}
		if err := db.Where("name = ?", rd.Name).FirstOrCreate(&role).Error; err != nil {
			lg.Error("rbac seed: role create failed", zap.Error(err), zap.String("name", rd.Name))
			continue
		}

		// ── 3. Sync role-permission assignments ──
		// Remove stale and re-insert current set inside a transaction
		err := db.Transaction(func(tx *gorm.DB) error {
			if err := tx.Where("role_id = ?", role.ID).Delete(&model.RolePermission{}).Error; err != nil {
				return err
			}
			for _, code := range rd.Perms {
				pid, ok := permIDMap[code]
				if !ok {
					continue
				}
				rp := model.RolePermission{RoleID: role.ID, PermissionID: pid}
				if err := tx.Create(&rp).Error; err != nil {
					return err
				}
			}
			return nil
		})
		if err != nil {
			lg.Error("rbac seed: sync role permissions failed", zap.Error(err), zap.String("role", rd.Name))
		}
		lg.Info("rbac seed: role ready", zap.String("name", role.Name), zap.Int("permissions", len(rd.Perms)))
	}

	// ── 4. Seed demo admin accounts for each role ──
	seedAdmins := []struct {
		Username, DisplayName, RoleName, Password string
	}{
		{"superadmin", "超级管理员", "super_admin", "admin123"},
		{"content_review", "内容审核员", "content_review", "review123"},
		{"cs_admin", "客服管理员", "cs_admin", "cs123"},
	}
	for _, sa := range seedAdmins {
		var a model.Admin
		err := db.Where("username = ?", sa.Username).First(&a).Error
		if err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
			lg.Error("rbac seed: admin lookup failed", zap.Error(err), zap.String("username", sa.Username))
			continue
		}
		if errors.Is(err, gorm.ErrRecordNotFound) {
			// Create new admin account
			hash, e := bcrypt.GenerateFromPassword([]byte(sa.Password), bcrypt.DefaultCost)
			if e != nil {
				lg.Error("rbac seed: hash failed", zap.Error(e), zap.String("username", sa.Username))
				continue
			}
			a = model.Admin{
				Username:     sa.Username,
				PasswordHash: string(hash),
				DisplayName:  sa.DisplayName,
				Status:       "active",
			}
			if e := db.Create(&a).Error; e != nil {
				lg.Error("rbac seed: create admin failed", zap.Error(e), zap.String("username", sa.Username))
				continue
			}
			lg.Info("rbac seed: admin account created",
				zap.String("username", sa.Username),
				zap.String("password", strings.Repeat("*", len(sa.Password))),
			)
		}
		// Always ensure role assignment (new or existing admin)
		var role model.AdminRole
		if db.Where("name = ?", sa.RoleName).First(&role).Error == nil {
			assign := model.AdminRoleAssignment{AdminID: a.ID, RoleID: role.ID}
			db.Where("admin_id = ?", a.ID).FirstOrCreate(&assign)
			lg.Info("rbac seed: role assigned",
				zap.String("username", sa.Username),
				zap.String("role", sa.RoleName),
				zap.Uint64("adminID", a.ID),
			)
		}
	}

	// ── 5. Ensure admin #1 (the bootstrap admin) gets super_admin role ──
	var count int64
	if db.Model(&model.AdminRoleAssignment{}).Where("admin_id = 1").Count(&count); count == 0 && errors.Is(db.First(&model.AdminRole{}, "name = ?", "super_admin").Error, nil) {
		var superRole model.AdminRole
		db.Where("name = ?", "super_admin").First(&superRole)
		db.Create(&model.AdminRoleAssignment{AdminID: 1, RoleID: superRole.ID})
		lg.Info("rbac seed: assigned super_admin to admin #1")
	}
}

func allCodes(perms []model.AdminPermission) []string {
	codes := make([]string, len(perms))
	for i, p := range perms {
		codes[i] = p.Code
	}
	return codes
}
