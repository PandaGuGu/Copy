import adminHttp from "@/utils/adminHttp";

export function adminListConversations(params) { return adminHttp.get("/api/v1/admin/cs/conversations", { params }); }
export function adminGetConversation(id) { return adminHttp.get(`/api/v1/admin/cs/conversations/${id}`); }
export function adminSendConversationMessage(id, content) { return adminHttp.post(`/api/v1/admin/cs/conversations/${id}/messages`, { content }); }
export function adminCloseConversation(id) { return adminHttp.post(`/api/v1/admin/cs/conversations/${id}/close`); }
export function adminAssignConversation(id) { return adminHttp.post(`/api/v1/admin/cs/conversations/${id}/assign`); }
export function adminListCsTemplates() { return adminHttp.get("/api/v1/admin/cs/templates"); }
export function adminCreateCsTemplate(payload) { return adminHttp.post("/api/v1/admin/cs/templates", payload); }
export function adminUpdateCsTemplate(id, payload) { return adminHttp.put(`/api/v1/admin/cs/templates/${id}`, payload); }
export function adminDeleteCsTemplate(id) { return adminHttp.delete(`/api/v1/admin/cs/templates/${id}`); }
