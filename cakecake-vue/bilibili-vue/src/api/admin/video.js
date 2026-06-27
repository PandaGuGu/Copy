import adminHttp from "@/utils/adminHttp";

export function adminListVideos(params = {}) { return adminHttp.get("/api/v1/admin/videos", { params }); }
export function adminGetVideo(id) { return adminHttp.get(`/api/v1/admin/videos/${id}`); }
export function adminApproveVideo(id) { return adminHttp.post(`/api/v1/admin/videos/${id}/approve`); }
export function adminRejectVideo(id, reason) { return adminHttp.post(`/api/v1/admin/videos/${id}/reject`, { reason }); }
export function adminDeleteVideo(id) { return adminHttp.post(`/api/v1/admin/videos/${id}/delete`); }
export function adminBatchApproveVideos() { return adminHttp.post("/api/v1/admin/videos/batch-approve"); }
