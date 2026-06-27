import adminHttp from "@/utils/adminHttp";

export function adminGetDashboard() { return adminHttp.get("/api/v1/admin/dashboard"); }
