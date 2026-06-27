import adminHttp from "@/utils/adminHttp";

export function adminListHotSearchOps() { return adminHttp.get("/api/v1/admin/hot-search/ops"); }
export function adminCreateHotSearchOp(payload) { return adminHttp.post("/api/v1/admin/hot-search/ops", payload); }
export function adminUpdateHotSearchOp(id, payload) { return adminHttp.put(`/api/v1/admin/hot-search/ops/${id}`, payload); }
export function adminDeleteHotSearchOp(id) { return adminHttp.delete(`/api/v1/admin/hot-search/ops/${id}`); }
export function adminPreviewHotSearch(limit = 10) { return adminHttp.get("/api/v1/admin/hot-search/preview", { params: { limit } }); }
export function adminHotSearchDashboard(limit = 10, redisLimit = 30) { return adminHttp.get("/api/v1/admin/hot-search/dashboard", { params: { limit, redis_limit: redisLimit } }); }
export function adminRemoveHotSearchRedis(keyword) { return adminHttp.post("/api/v1/admin/hot-search/redis/remove", { keyword }); }
export function adminBoostHotSearchRedis(keyword, delta = 5) { return adminHttp.post("/api/v1/admin/hot-search/redis/boost", { keyword, delta }); }
export function adminQuickHotSearchOp(payload) { return adminHttp.post("/api/v1/admin/hot-search/quick-op", payload); }
export function adminReorderHotSearch(items) { return adminHttp.post("/api/v1/admin/hot-search/reorder", { items }); }
export function adminResetHotSearchDisplayOrder() { return adminHttp.post("/api/v1/admin/hot-search/display-order/reset"); }
