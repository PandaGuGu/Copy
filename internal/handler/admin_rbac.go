package handler

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ──────────────────────────────────────────────
// Module 23: RBAC & Audit
// ──────────────────────────────────────────────

// AdminListAdmins GET /admin/rbac/admins — list all admins with their assigned roles
func (a *API) AdminListAdmins(c *gin.Context) {
	var admins []model.Admin
	if err := a.DB.Select("id, username, display_name, status, last_login_at, created_at").Order("id ASC").Find(&admins).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(admins))
	for _, adm := range admins {
		h := gin.H{
			"id": adm.ID, "username": adm.Username,
			"nickname": adm.DisplayName, "display_name": adm.DisplayName,
			"status": adm.Status, "created_at": adm.CreatedAt,
		}
		if adm.LastLoginAt != nil { h["last_login_at"] = adm.LastLoginAt }
		// Look up assigned role
		var assign model.AdminRoleAssignment
		if err := a.DB.Where("admin_id = ?", adm.ID).First(&assign).Error; err == nil {
			var role model.AdminRole
			if err := a.DB.First(&role, assign.RoleID).Error; err == nil {
				h["role"] = gin.H{
					"id": role.ID, "name": role.Name,
					"description": role.Description,
				}
				h["role_id"] = role.ID
			}
		}
		items = append(items, h)
	}
	resp.OK(c, gin.H{"items": items, "total": len(items)})
}

// AdminGetMyPermissions GET /admin/rbac/me/permissions — current admin's permission codes
func (a *API) AdminGetMyPermissions(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var codes []string
	a.DB.Raw(`SELECT DISTINCT ap.code
		FROM admin_role_assignments ara
		JOIN role_permissions rp ON rp.role_id = ara.role_id
		JOIN admin_permissions ap ON ap.id = rp.permission_id
		WHERE ara.admin_id = ?`, adminID).Scan(&codes)
	resp.OK(c, gin.H{"permissions": codes})
}

// AdminListRoles GET /admin/rbac/roles — list roles
func (a *API) AdminListRoles(c *gin.Context) {
	var roles []model.AdminRole
	if err := a.DB.Order("id ASC").Find(&roles).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(roles))
	for i := range roles {
		var permCount int64
		a.DB.Model(&model.RolePermission{}).Where("role_id = ?", roles[i].ID).Count(&permCount)
		var adminCount int64
		a.DB.Model(&model.AdminRoleAssignment{}).Where("role_id = ?", roles[i].ID).Count(&adminCount)
		items = append(items, gin.H{
			"id":             roles[i].ID,
			"name":           roles[i].Name,
			"description":    roles[i].Description,
			"permission_count": permCount,
			"admin_count":    adminCount,
			"created_at":     roles[i].CreatedAt,
			"updated_at":     roles[i].UpdatedAt,
		})
	}
	resp.OK(c, gin.H{"items": items})
}

type roleReq struct {
	Name        string `json:"name"`
	Description string `json:"description"`
}

// AdminCreateRole POST /admin/rbac/roles — create role (body: name, description)
func (a *API) AdminCreateRole(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var req roleReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	if req.Name == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	role := model.AdminRole{
		Name:        req.Name,
		Description: req.Description,
	}
	if err := a.DB.Create(&role).Error; err != nil {
		a.Log.Error("create role", zap.Error(err), zap.String("name", role.Name))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "create_role", "role", role.ID, `{"name":"`+role.Name+`"}`)
	resp.OK(c, gin.H{
		"id":          role.ID,
		"name":        role.Name,
		"description": role.Description,
		"created_at":  role.CreatedAt,
	})
}

// AdminUpdateRole PUT /admin/rbac/roles/:id — update role
func (a *API) AdminUpdateRole(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var req roleReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var role model.AdminRole
	if err := a.DB.First(&role, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	updates := map[string]interface{}{}
	if strings.TrimSpace(req.Name) != "" {
		updates["name"] = strings.TrimSpace(req.Name)
	}
	updates["description"] = req.Description
	if err := a.DB.Model(&role).Updates(updates).Error; err != nil {
		a.Log.Error("update role", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "update_role", "role", id, "")
	resp.OK(c, gin.H{"id": id, "ok": true})
}

// AdminDeleteRole DELETE /admin/rbac/roles/:id — delete role (super_admin cannot be deleted)
func (a *API) AdminDeleteRole(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var role model.AdminRole
	if err := a.DB.First(&role, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if role.Name == "super_admin" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeForbidden)
		return
	}
	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Where("role_id = ?", id).Delete(&model.RolePermission{}).Error; err != nil {
			return err
		}
		if err := tx.Where("role_id = ?", id).Delete(&model.AdminRoleAssignment{}).Error; err != nil {
			return err
		}
		return tx.Delete(&model.AdminRole{}, id).Error
	}); err != nil {
		a.Log.Error("delete role", zap.Error(err), zap.Uint64("id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "delete_role", "role", id, `{"name":"`+role.Name+`"}`)
	resp.OK(c, gin.H{"id": id, "deleted": true})
}

// AdminListPermissions GET /admin/rbac/permissions — list all permission points
func (a *API) AdminListPermissions(c *gin.Context) {
	var perms []model.AdminPermission
	q := a.DB.Model(&model.AdminPermission{})
	if resource := strings.TrimSpace(c.Query("resource")); resource != "" {
		q = q.Where("resource = ?", resource)
	}
	if err := q.Order("resource ASC, id ASC").Find(&perms).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(perms))
	for i := range perms {
		items = append(items, gin.H{
			"id":       perms[i].ID,
			"code":     perms[i].Code,
			"resource": perms[i].Resource,
			"action":   perms[i].Action,
		})
	}
	resp.OK(c, gin.H{"items": items})
}

type assignPermsReq struct {
	PermissionIDs  []uint64 `json:"permission_ids"`
	PermissionCodes []string `json:"permissions"`
}

// AdminAssignRolePermissions POST /admin/rbac/roles/:id/permissions — assign permissions to role
func (a *API) AdminAssignRolePermissions(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var req assignPermsReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	// Resolve permission codes to IDs if codes are provided
	permIDs := req.PermissionIDs
	if len(req.PermissionCodes) > 0 {
		var perms []model.AdminPermission
		if err := a.DB.Where("code IN ?", req.PermissionCodes).Find(&perms).Error; err != nil {
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
		for _, p := range perms {
			permIDs = append(permIDs, p.ID)
		}
	}
	var role model.AdminRole
	if err := a.DB.First(&role, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		// Replace: wipe existing then insert the new set.
		if err := tx.Where("role_id = ?", id).Delete(&model.RolePermission{}).Error; err != nil {
			return err
		}
		if len(permIDs) == 0 {
			return nil
		}
		// Deduplicate.
		seen := map[uint64]bool{}
		rows := make([]model.RolePermission, 0, len(permIDs))
		for _, pid := range permIDs {
			if pid == 0 || seen[pid] {
				continue
			}
			seen[pid] = true
			rows = append(rows, model.RolePermission{RoleID: id, PermissionID: pid})
		}
		if len(rows) == 0 {
			return nil
		}
		return tx.Create(&rows).Error
	}); err != nil {
		a.Log.Error("assign role permissions", zap.Error(err), zap.Uint64("role_id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.recordAudit(c, adminID, "assign_role_permissions", "role", id, `{"permission_ids_count":`+strconv.Itoa(len(permIDs))+`}`)
	resp.OK(c, gin.H{"id": id, "assigned": len(permIDs)})
}

// AdminGetRolePermissions GET /admin/rbac/roles/:id/permissions — get role's permissions
func (a *API) AdminGetRolePermissions(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var role model.AdminRole
	if err := a.DB.First(&role, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	var perms []model.AdminPermission
	if err := a.DB.
		Joins("JOIN role_permissions ON role_permissions.permission_id = admin_permissions.id").
		Where("role_permissions.role_id = ?", id).
		Order("admin_permissions.resource ASC, admin_permissions.id ASC").
		Find(&perms).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(perms))
	for i := range perms {
		items = append(items, gin.H{
			"id":       perms[i].ID,
			"code":     perms[i].Code,
			"resource": perms[i].Resource,
			"action":   perms[i].Action,
		})
	}
	resp.OK(c, gin.H{
		"role": gin.H{
			"id":          role.ID,
			"name":        role.Name,
			"description": role.Description,
		},
		"permissions": items,
	})
}

type createAdminReq struct {
	Username    string `json:"username"`
	Password    string `json:"password"`
	DisplayName string `json:"display_name"`
	RoleID      uint64 `json:"role_id"`
}

// AdminCreateAdmin POST /admin/rbac/admins — create new admin account
func (a *API) AdminCreateAdmin(c *gin.Context) {
	curAdminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var req createAdminReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Username = strings.TrimSpace(req.Username)
	req.DisplayName = strings.TrimSpace(req.DisplayName)
	if req.Username == "" || req.Password == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	// Check uniqueness
	var exist int64
	a.DB.Model(&model.Admin{}).Where("username = ?", req.Username).Count(&exist)
	if exist > 0 {
		resp.Err(c, http.StatusConflict, errcode.CodeParamError)
		return
	}

	hash, err := bcrypt.GenerateFromPassword([]byte(req.Password), bcrypt.DefaultCost)
	if err != nil {
		a.Log.Error("create admin hash", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	admin := model.Admin{
		Username:     req.Username,
		PasswordHash: string(hash),
		DisplayName:  req.DisplayName,
		Status:       "active",
	}
	if err := a.DB.Create(&admin).Error; err != nil {
		a.Log.Error("create admin", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Assign role if specified
	if req.RoleID > 0 {
		var role model.AdminRole
		if a.DB.First(&role, req.RoleID).Error == nil {
			a.DB.Create(&model.AdminRoleAssignment{AdminID: admin.ID, RoleID: req.RoleID})
		}
	}

	a.recordAudit(c, curAdminID, "create_admin", "admin", admin.ID, `{"username":"`+admin.Username+`"}`)
	resp.OK(c, gin.H{"id": admin.ID, "username": admin.Username, "display_name": admin.DisplayName})
}

type assignAdminRoleReq struct {
	RoleID uint64 `json:"role_id"`
}

// AdminAssignAdminRole POST /admin/rbac/admins/:adminId/role — assign role to admin (body: role_id)
func (a *API) AdminAssignAdminRole(c *gin.Context) {
	curAdminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	adminID, err := strconv.ParseUint(c.Param("adminId"), 10, 64)
	if err != nil || adminID == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var req assignAdminRoleReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.RoleID == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	// Verify target admin and role exist.
	var target model.Admin
	if err := a.DB.First(&target, adminID).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	var role model.AdminRole
	if err := a.DB.First(&role, req.RoleID).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	// Upsert the single assignment (unique admin_id).
	var assign model.AdminRoleAssignment
	if err := a.DB.Where("admin_id = ?", adminID).First(&assign).Error; err == gorm.ErrRecordNotFound {
		assign = model.AdminRoleAssignment{AdminID: adminID, RoleID: req.RoleID}
		if err := a.DB.Create(&assign).Error; err != nil {
			a.Log.Error("assign admin role", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	} else if err != nil {
		a.Log.Error("lookup admin role assignment", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	} else {
		if err := a.DB.Model(&assign).Update("role_id", req.RoleID).Error; err != nil {
			a.Log.Error("update admin role assignment", zap.Error(err))
			resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
			return
		}
	}
	a.recordAudit(c, curAdminID, "assign_admin_role", "admin", adminID, `{"role_id":`+strconv.FormatUint(req.RoleID, 10)+`}`)
	resp.OK(c, gin.H{"admin_id": adminID, "role_id": req.RoleID, "role_name": role.Name})
}

// AdminGetAdminRole GET /admin/rbac/admins/:adminId/role — get admin's role
func (a *API) AdminGetAdminRole(c *gin.Context) {
	adminID, err := strconv.ParseUint(c.Param("adminId"), 10, 64)
	if err != nil || adminID == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var assign model.AdminRoleAssignment
	if err := a.DB.Where("admin_id = ?", adminID).First(&assign).Error; err != nil {
		resp.OK(c, gin.H{"admin_id": adminID, "role_id": nil, "role": nil})
		return
	}
	var role model.AdminRole
	if err := a.DB.First(&role, assign.RoleID).Error; err != nil {
		resp.OK(c, gin.H{"admin_id": adminID, "role_id": assign.RoleID, "role": nil})
		return
	}
	resp.OK(c, gin.H{
		"admin_id": adminID,
		"role_id":  role.ID,
		"role": gin.H{
			"id":          role.ID,
			"name":        role.Name,
			"description": role.Description,
		},
	})
}

// AdminListAuditLogs GET /admin/rbac/audit-logs — list audit logs
// (filter: admin_id, action, resource, time range)
func (a *API) AdminListAuditLogs(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	q := a.DB.Model(&model.AuditLog{})
	if v := strings.TrimSpace(c.Query("admin_id")); v != "" {
		if aid, err := strconv.ParseUint(v, 10, 64); err == nil {
			q = q.Where("admin_id = ?", aid)
		}
	}
	if v := strings.TrimSpace(c.Query("action")); v != "" {
		q = q.Where("action = ?", v)
	}
	if v := strings.TrimSpace(c.Query("resource")); v != "" {
		q = q.Where("resource = ?", v)
	}
	if v := strings.TrimSpace(c.Query("start")); v != "" {
		if t, err := time.Parse(time.RFC3339, v); err == nil {
			q = q.Where("created_at >= ?", t)
		}
	}
	if v := strings.TrimSpace(c.Query("end")); v != "" {
		if t, err := time.Parse(time.RFC3339, v); err == nil {
			q = q.Where("created_at <= ?", t)
		}
	}

	var total int64
	if err := q.Count(&total).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	offset := (page - 1) * pageSize
	var rows []model.AuditLog
	if err := q.Order("created_at DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":         rows[i].ID,
			"admin_id":   rows[i].AdminID,
			"action":     rows[i].Action,
			"resource":   rows[i].Resource,
			"target_id":  rows[i].TargetID,
			"detail":     rows[i].Detail,
			"ip_address": rows[i].IPAddress,
			"created_at": rows[i].CreatedAt,
		})
	}
	totalPages := int((total + int64(pageSize) - 1) / int64(pageSize))
	if totalPages < 1 {
		totalPages = 1
	}
	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}

// AdminGetAuditLog GET /admin/rbac/audit-logs/:id — get audit log detail
func (a *API) AdminGetAuditLog(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var log model.AuditLog
	if err := a.DB.First(&log, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	resp.OK(c, gin.H{
		"id":         log.ID,
		"admin_id":   log.AdminID,
		"action":     log.Action,
		"resource":   log.Resource,
		"target_id":  log.TargetID,
		"detail":     log.Detail,
		"ip_address": log.IPAddress,
		"created_at": log.CreatedAt,
	})
}

// ──────────────────────────────────────────────
// Approval Flows
// ──────────────────────────────────────────────

type approvalFlowReq struct {
	ResourceType string `json:"resource_type"`
	ResourceID   uint64 `json:"resource_id"`
	TotalSteps   int    `json:"total_steps"`
}

// AdminCreateApprovalFlow POST /admin/rbac/approval-flows — create approval flow
func (a *API) AdminCreateApprovalFlow(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	var req approvalFlowReq
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if strings.TrimSpace(req.ResourceType) == "" || req.TotalSteps < 1 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	flow := model.ApprovalFlow{
		ResourceType: strings.TrimSpace(req.ResourceType),
		ResourceID:   req.ResourceID,
		Status:       "pending",
		CurrentStep:  1,
		TotalSteps:   req.TotalSteps,
		RequestorID:  adminID,
	}
	if err := a.DB.Create(&flow).Error; err != nil {
		a.Log.Error("create approval flow", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	// Auto-create the first step with the requesting admin as the initial approver.
	step := model.ApprovalStep{
		FlowID:     flow.ID,
		StepNumber: 1,
		ApproverID: adminID,
		Decision:   "pending",
	}
	_ = a.DB.Create(&step).Error
	a.recordAudit(c, adminID, "create_approval_flow", "approval_flow", flow.ID, `{"resource_type":"`+flow.ResourceType+`","resource_id":`+strconv.FormatUint(flow.ResourceID, 10)+`}`)
	resp.OK(c, gin.H{
		"id":            flow.ID,
		"resource_type": flow.ResourceType,
		"resource_id":   flow.ResourceID,
		"status":        flow.Status,
		"current_step":  flow.CurrentStep,
		"total_steps":   flow.TotalSteps,
		"created_at":    flow.CreatedAt,
	})
}

type approveStepReq struct {
	Comment string `json:"comment"`
}

// AdminApproveFlowStep POST /admin/rbac/approval-flows/:id/approve — approve current step
func (a *API) AdminApproveFlowStep(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var req approveStepReq
	_ = c.ShouldBindJSON(&req)

	var flow model.ApprovalFlow
	if err := a.DB.First(&flow, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if flow.Status != "pending" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		// Mark the current step approved.
		if err := tx.Model(&model.ApprovalStep{}).
			Where("flow_id = ? AND step_number = ?", id, flow.CurrentStep).
			Updates(map[string]interface{}{
				"decision":   "approved",
				"comment":    req.Comment,
				"decided_at": &now,
			}).Error; err != nil {
			return err
		}
		if flow.CurrentStep >= flow.TotalSteps {
			// All steps done → flow approved.
			return tx.Model(&flow).Updates(map[string]interface{}{
				"status":      "approved",
				"current_step": flow.TotalSteps,
			}).Error
		}
		// Advance to next step.
		nextStep := flow.CurrentStep + 1
		nextStepRow := model.ApprovalStep{
			FlowID:     id,
			StepNumber: nextStep,
			ApproverID: adminID,
			Decision:   "pending",
		}
		if err := tx.Create(&nextStepRow).Error; err != nil {
			return err
		}
		return tx.Model(&flow).Updates(map[string]interface{}{
			"current_step": nextStep,
		}).Error
	}); err != nil {
		a.Log.Error("approve flow step", zap.Error(err), zap.Uint64("flow_id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	_ = a.DB.First(&flow, id).Error
	a.recordAudit(c, adminID, "approve_flow_step", "approval_flow", id, `{"step":`+strconv.Itoa(flow.CurrentStep)+`}`)
	resp.OK(c, gin.H{
		"id":           flow.ID,
		"status":       flow.Status,
		"current_step": flow.CurrentStep,
		"total_steps":  flow.TotalSteps,
	})
}

// AdminRejectFlowStep POST /admin/rbac/approval-flows/:id/reject — reject current step
func (a *API) AdminRejectFlowStep(c *gin.Context) {
	adminID, ok := middleware.AdminID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var req approveStepReq
	_ = c.ShouldBindJSON(&req)

	var flow model.ApprovalFlow
	if err := a.DB.First(&flow, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if flow.Status != "pending" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	if err := a.DB.Transaction(func(tx *gorm.DB) error {
		if err := tx.Model(&model.ApprovalStep{}).
			Where("flow_id = ? AND step_number = ?", id, flow.CurrentStep).
			Updates(map[string]interface{}{
				"decision":   "rejected",
				"comment":    req.Comment,
				"decided_at": &now,
			}).Error; err != nil {
			return err
		}
		return tx.Model(&flow).Update("status", "rejected").Error
	}); err != nil {
		a.Log.Error("reject flow step", zap.Error(err), zap.Uint64("flow_id", id))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	_ = a.DB.First(&flow, id).Error
	a.recordAudit(c, adminID, "reject_flow_step", "approval_flow", id, `{"step":`+strconv.Itoa(flow.CurrentStep)+`}`)
	resp.OK(c, gin.H{
		"id":           flow.ID,
		"status":       flow.Status,
		"current_step": flow.CurrentStep,
		"total_steps":  flow.TotalSteps,
	})
}

// AdminListApprovalFlows GET /admin/rbac/approval-flows — list approval flows
func (a *API) AdminListApprovalFlows(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	q := a.DB.Model(&model.ApprovalFlow{})
	if v := strings.TrimSpace(c.Query("status")); v != "" {
		q = q.Where("status = ?", v)
	}
	if v := strings.TrimSpace(c.Query("resource_type")); v != "" {
		q = q.Where("resource_type = ?", v)
	}

	var total int64
	if err := q.Count(&total).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	offset := (page - 1) * pageSize
	var rows []model.ApprovalFlow
	if err := q.Order("created_at DESC, id DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":            rows[i].ID,
			"resource_type": rows[i].ResourceType,
			"resource_id":   rows[i].ResourceID,
			"status":        rows[i].Status,
			"current_step":  rows[i].CurrentStep,
			"total_steps":   rows[i].TotalSteps,
			"requestor_id":  rows[i].RequestorID,
			"created_at":    rows[i].CreatedAt,
			"updated_at":    rows[i].UpdatedAt,
		})
	}
	totalPages := int((total + int64(pageSize) - 1) / int64(pageSize))
	if totalPages < 1 {
		totalPages = 1
	}
	resp.OK(c, gin.H{
		"items":       items,
		"page":        page,
		"page_size":   pageSize,
		"total":       total,
		"total_pages": totalPages,
	})
}
