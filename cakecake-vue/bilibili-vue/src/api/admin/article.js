import adminHttp from "@/utils/adminHttp";

export function adminListArticles(params = {}) { return adminHttp.get("/api/v1/admin/articles", { params }); }
export function adminGetArticle(id) { return adminHttp.get(`/api/v1/admin/articles/${id}`); }
export function adminApproveArticle(id) { return adminHttp.post(`/api/v1/admin/articles/${id}/approve`); }
export function adminRejectArticle(id, reason) { return adminHttp.post(`/api/v1/admin/articles/${id}/reject`, { reason }); }
export function adminDeleteArticle(id) { return adminHttp.post(`/api/v1/admin/articles/${id}/delete`); }
