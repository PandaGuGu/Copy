import adminHttp from "@/utils/adminHttp";

export function adminListDynamics(params = {}) { return adminHttp.get("/api/v1/admin/dynamics", { params }); }
export function adminGetDynamic(id) { return adminHttp.get(`/api/v1/admin/dynamics/${id}`); }
export function adminDeleteDynamic(id) { return adminHttp.post(`/api/v1/admin/dynamics/${id}/delete`); }
