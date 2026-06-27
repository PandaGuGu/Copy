import adminHttp from "@/utils/adminHttp";

export function adminListReports(params) { return adminHttp.get("/api/v1/admin/reports", { params }); }
export function adminHandleReport(id, payload) { return adminHttp.post(`/api/v1/admin/reports/${id}/handle`, payload); }
export function adminBatchHandleReports(payload) { return adminHttp.post("/api/v1/admin/reports/batch", payload); }
export function adminDeleteReport(id) { return adminHttp.delete(`/api/v1/admin/reports/${id}`); }
