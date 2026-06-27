import adminHttp from "@/utils/adminHttp";

export function adminListRoles() { return adminHttp.get("/api/v1/admin/rbac/roles"); }
export function adminCreateRole(payload) { return adminHttp.post("/api/v1/admin/rbac/roles", payload); }
export function adminUpdateRole(id, payload) { return adminHttp.put(`/api/v1/admin/rbac/roles/${id}`, payload); }
export function adminDeleteRole(id) { return adminHttp.delete(`/api/v1/admin/rbac/roles/${id}`); }
export function adminGetRole(id) { return adminHttp.get(`/api/v1/admin/rbac/roles/${id}`); }
export function adminAssignPermissions(roleId, permissions) { return adminHttp.post(`/api/v1/admin/rbac/roles/${roleId}/permissions`, { permissions }); }
export function adminListAdmins(params) { return adminHttp.get("/api/v1/admin/rbac/admins", { params }); }
export function adminCreateAdmin(payload) { return adminHttp.post("/api/v1/admin/rbac/admins", payload); }
export function adminAssignRole(adminId, roleId) { return adminHttp.post(`/api/v1/admin/rbac/admins/${adminId}/role`, { role_id: roleId }); }
export function adminListAuditLogs(params) { return adminHttp.get("/api/v1/admin/rbac/audit-logs", { params }); }
export function adminListApprovals(params) { return adminHttp.get("/api/v1/admin/rbac/approval-flows", { params }); }
export function adminApproveFlow(id) { return adminHttp.post(`/api/v1/admin/rbac/approval-flows/${id}/approve`); }
export function adminRejectFlow(id) { return adminHttp.post(`/api/v1/admin/rbac/approval-flows/${id}/reject`); }
export function adminGetMyPermissions() { return adminHttp.get("/api/v1/admin/rbac/me/permissions"); }
