import adminHttp from "@/utils/adminHttp";

export function adminGetSettings() { return adminHttp.get("/api/v1/admin/settings"); }
export function adminPutSettings(payload) { return adminHttp.put("/api/v1/admin/settings", payload); }
