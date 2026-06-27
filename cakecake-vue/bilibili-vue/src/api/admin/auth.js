import adminHttp from "@/utils/adminHttp";

export function adminLogin(username, password) {
  return adminHttp.post("/api/v1/admin/auth/login", { username, password }, { skipGlobalErrorToast: true });
}

export function adminMe() {
  return adminHttp.get("/api/v1/admin/me");
}
