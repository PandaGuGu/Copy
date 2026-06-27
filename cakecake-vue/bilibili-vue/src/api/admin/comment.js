import adminHttp from "@/utils/adminHttp";

export function adminListComments(params) { return adminHttp.get("/api/v1/admin/comments", { params }); }
export function adminGetComment(id, type) { return adminHttp.get(`/api/v1/admin/comments/${id}`, { params: { type } }); }
export function adminDeleteComment(id, type) { return adminHttp.post(`/api/v1/admin/comments/${id}/delete`, null, { params: { type } }); }
