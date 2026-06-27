import adminHttp from "@/utils/adminHttp";

export function adminListUsers(params) { return adminHttp.get("/api/v1/admin/users", { params }); }
export function adminGetUser(id) { return adminHttp.get(`/api/v1/admin/users/${id}`); }
export function adminBanUser(id, reason) { return adminHttp.post(`/api/v1/admin/users/${id}/ban`, { reason }); }
export function adminUnbanUser(id) { return adminHttp.post(`/api/v1/admin/users/${id}/unban`); }
export function adminDeleteUser(id) { return adminHttp.post(`/api/v1/admin/users/${id}/delete`); }
export function adminGetUserViolations(uid) { return adminHttp.get(`/api/v1/admin/users/${uid}/violations`); }
