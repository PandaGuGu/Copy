import adminHttp from "@/utils/adminHttp";

export function adminListUsers(params) { return adminHttp.get("/api/v1/admin/users", { params }); }
export function adminGetUser(id) { return adminHttp.get(`/api/v1/admin/users/${id}`); }
export function adminBanUser(id, reason) { return adminHttp.post(`/api/v1/admin/users/${id}/ban`, { reason }); }
export function adminUnbanUser(id) { return adminHttp.post(`/api/v1/admin/users/${id}/unban`); }
export function adminDeleteUser(id) { return adminHttp.post(`/api/v1/admin/users/${id}/delete`); }
export function adminGetUserViolations(uid) { return adminHttp.get(`/api/v1/admin/users/${uid}/violations`); }
export function adminListUserCapabilities(uid) { return adminHttp.get(`/api/v1/admin/users/${uid}/capabilities`); }
export function adminRestrictUserCapability(uid, payload) { return adminHttp.post(`/api/v1/admin/users/${uid}/capabilities`, payload); }
export function adminRestoreUserCapability(uid, capability) { return adminHttp.delete(`/api/v1/admin/users/${uid}/capabilities/${capability}`); }
export function adminListCapReasonTemplates() { return adminHttp.get("/api/v1/admin/usercap/templates"); }
export function adminAddCapReasonTemplate(content) { return adminHttp.post("/api/v1/admin/usercap/templates", { content }); }
export function adminUpdateCapReasonTemplate(id, content) { return adminHttp.put(`/api/v1/admin/usercap/templates/${id}`, { content }); }
export function adminDeleteCapReasonTemplate(id) { return adminHttp.delete(`/api/v1/admin/usercap/templates/${id}`); }
