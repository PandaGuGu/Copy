import adminHttp from "@/utils/adminHttp";

export function adminListSpecials() { return adminHttp.get("/api/v1/admin/specials"); }
export function adminCreateSpecial(payload) { return adminHttp.post("/api/v1/admin/specials", payload); }
export function adminUpdateSpecial(id, payload) { return adminHttp.put(`/api/v1/admin/specials/${id}`, payload); }
export function adminDeleteSpecial(id) { return adminHttp.delete(`/api/v1/admin/specials/${id}`); }
export function adminListCampaigns() { return adminHttp.get("/api/v1/admin/campaigns"); }
export function adminCreateCampaign(payload) { return adminHttp.post("/api/v1/admin/campaigns", payload); }
export function adminUpdateCampaign(id, payload) { return adminHttp.put(`/api/v1/admin/campaigns/${id}`, payload); }
export function adminDeleteCampaign(id) { return adminHttp.delete(`/api/v1/admin/campaigns/${id}`); }
