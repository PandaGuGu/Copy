import adminHttp from "@/utils/adminHttp";

export function adminListAppeals(params) { return adminHttp.get("/api/v1/admin/appeals", { params }); }
export function adminHandleAppeal(id, payload) { return adminHttp.post(`/api/v1/admin/appeals/${id}/handle`, payload); }