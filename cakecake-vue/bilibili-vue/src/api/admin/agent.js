import adminHttp from "@/utils/adminHttp";

export function adminGetAgentSettings() { return adminHttp.get("/api/v1/admin/agent-settings"); }
export function adminPutAgentSettings(payload) { return adminHttp.put("/api/v1/admin/agent-settings", payload); }
export function adminUploadAgentAvatar(file) { const fd = new FormData(); fd.append("image", file); return adminHttp.post("/api/v1/admin/agent-settings/avatar", fd); }
export function adminListAgentProfiles() { return adminHttp.get("/api/v1/admin/agent-profiles"); }
export function adminCreateAgentProfile(payload) { return adminHttp.post("/api/v1/admin/agent-profiles", payload); }
export function adminUpdateAgentProfile(id, payload) { return adminHttp.put(`/api/v1/admin/agent-profiles/${id}`, payload); }
export function adminDeleteAgentProfile(id) { return adminHttp.delete(`/api/v1/admin/agent-profiles/${id}`); }
export function adminUploadAgentProfileAvatar(id, file) { const fd = new FormData(); fd.append("image", file); return adminHttp.post(`/api/v1/admin/agent-profiles/${id}/avatar`, fd); }
export function adminGetLLMConfig() { return adminHttp.get("/api/v1/admin/llm-config"); }
export function adminPutLLMConfig(payload) { return adminHttp.put("/api/v1/admin/llm-config", payload); }
export function adminListLLMProviders() { return adminHttp.get("/api/v1/admin/llm-config/providers"); }
export function adminCreateLLMProvider(payload) { return adminHttp.post("/api/v1/admin/llm-config/providers", payload); }
export function adminUpdateLLMProvider(id, payload) { return adminHttp.put(`/api/v1/admin/llm-config/providers/${id}`, payload); }
export function adminDeleteLLMProvider(id) { return adminHttp.delete(`/api/v1/admin/llm-config/providers/${id}`); }
export function adminSetDefaultLLMProvider(id) { return adminHttp.post(`/api/v1/admin/llm-config/providers/${id}/set-default`); }
