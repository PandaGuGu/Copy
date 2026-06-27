import adminHttp from "@/utils/adminHttp";
import http from "@/utils/http";

const isMinibili =
  import.meta.env.VITE_MINIBILI_API === "true" ||
  import.meta.env.VITE_MINIBILI_API === "1";

export function adminLogin(username, password) {
  return adminHttp.post(
    "/api/v1/admin/auth/login",
    { username, password },
    { skipGlobalErrorToast: true }
  );
}

export function adminMe() {
  return adminHttp.get("/api/v1/admin/me");
}

export function adminListBanners() {
  return adminHttp.get("/api/v1/admin/home-banners");
}

export function adminCreateBanner(payload) {
  return adminHttp.post("/api/v1/admin/home-banners", payload);
}

export function adminUpdateBanner(id, payload) {
  return adminHttp.put(`/api/v1/admin/home-banners/${id}`, payload);
}

export function adminDeleteBanner(id) {
  return adminHttp.delete(`/api/v1/admin/home-banners/${id}`);
}

/** 轮播图上传 OSS；新建用 upload-image，编辑已有轮播可传 bannerId 直接写库 */
export function adminUploadBannerImage(file, bannerId) {
  const fd = new FormData();
  fd.append("image", file);
  const opts = { timeout: 120000, skipGlobalErrorToast: false };
  if (bannerId) {
    return adminHttp.post(`/api/v1/admin/home-banners/${bannerId}/image`, fd, opts);
  }
  return adminHttp.post("/api/v1/admin/home-banners/upload-image", fd, opts);
}

export function adminListHotSearchOps() {
  return adminHttp.get("/api/v1/admin/hot-search/ops");
}

export function adminCreateHotSearchOp(payload) {
  return adminHttp.post("/api/v1/admin/hot-search/ops", payload);
}

export function adminUpdateHotSearchOp(id, payload) {
  return adminHttp.put(`/api/v1/admin/hot-search/ops/${id}`, payload);
}

export function adminDeleteHotSearchOp(id) {
  return adminHttp.delete(`/api/v1/admin/hot-search/ops/${id}`);
}

export function adminPreviewHotSearch(limit = 10) {
  return adminHttp.get("/api/v1/admin/hot-search/preview", {
    params: { limit }
  });
}

export function adminHotSearchDashboard(limit = 10, redisLimit = 30) {
  return adminHttp.get("/api/v1/admin/hot-search/dashboard", {
    params: { limit, redis_limit: redisLimit }
  });
}

export function adminRemoveHotSearchRedis(keyword) {
  return adminHttp.post("/api/v1/admin/hot-search/redis/remove", { keyword });
}

export function adminBoostHotSearchRedis(keyword, delta = 5) {
  return adminHttp.post("/api/v1/admin/hot-search/redis/boost", { keyword, delta });
}

export function adminQuickHotSearchOp(payload) {
  return adminHttp.post("/api/v1/admin/hot-search/quick-op", payload);
}

export function adminReorderHotSearch(items) {
  return adminHttp.post("/api/v1/admin/hot-search/reorder", { items });
}

export function adminResetHotSearchDisplayOrder() {
  return adminHttp.post("/api/v1/admin/hot-search/display-order/reset");
}

export function adminListVideos(params = {}) {
  return adminHttp.get("/api/v1/admin/videos", { params });
}

export function adminGetVideo(id) {
  return adminHttp.get(`/api/v1/admin/videos/${id}`);
}

export function adminApproveVideo(id) {
  return adminHttp.post(`/api/v1/admin/videos/${id}/approve`);
}

export function adminRejectVideo(id, reason) {
  return adminHttp.post(`/api/v1/admin/videos/${id}/reject`, { reason });
}

export function adminDeleteVideo(id) {
  return adminHttp.post(`/api/v1/admin/videos/${id}/delete`);
}

export function adminBatchApproveVideos() {
  return adminHttp.post("/api/v1/admin/videos/batch-approve");
}

export function adminListArticles(params = {}) {
  return adminHttp.get("/api/v1/admin/articles", { params });
}

export function adminGetArticle(id) {
  return adminHttp.get(`/api/v1/admin/articles/${id}`);
}

export function adminApproveArticle(id) {
  return adminHttp.post(`/api/v1/admin/articles/${id}/approve`);
}

export function adminRejectArticle(id, reason) {
  return adminHttp.post(`/api/v1/admin/articles/${id}/reject`, { reason });
}

export function adminDeleteArticle(id) {
  return adminHttp.post(`/api/v1/admin/articles/${id}/delete`);
}

export function adminListDynamics(params = {}) {
  return adminHttp.get("/api/v1/admin/dynamics", { params });
}

export function adminGetDynamic(id) {
  return adminHttp.get(`/api/v1/admin/dynamics/${id}`);
}

export function adminDeleteDynamic(id) {
  return adminHttp.post(`/api/v1/admin/dynamics/${id}/delete`);
}

export function adminGetAgentSettings() {
  return adminHttp.get("/api/v1/admin/agent-settings");
}

export function adminPutAgentSettings(payload) {
  return adminHttp.put("/api/v1/admin/agent-settings", payload);
}

export function adminUploadAgentAvatar(file) {
  const fd = new FormData();
  fd.append("image", file);
  return adminHttp.post("/api/v1/admin/agent-settings/avatar", fd);
}

export function adminListAgentProfiles() {
  return adminHttp.get("/api/v1/admin/agent-profiles");
}

export function adminCreateAgentProfile(payload) {
  return adminHttp.post("/api/v1/admin/agent-profiles", payload);
}

export function adminUpdateAgentProfile(id, payload) {
  return adminHttp.put(`/api/v1/admin/agent-profiles/${id}`, payload);
}

export function adminDeleteAgentProfile(id) {
  return adminHttp.delete(`/api/v1/admin/agent-profiles/${id}`);
}

export function adminUploadAgentProfileAvatar(id, file) {
  const fd = new FormData();
  fd.append("image", file);
  return adminHttp.post(`/api/v1/admin/agent-profiles/${id}/avatar`, fd);
}

/** 主站首页轮播（公开接口） */
export function getHomeBannersPublic() {
  if (!isMinibili) {
    return Promise.resolve({ code: 0, data: { items: [] } });
  }
  return http.get("/api/v1/home-banners", { skipGlobalErrorToast: true });
}

// ---------- 用户管理 ----------

// ---------- LLM 模型配置 ----------

export function adminGetLLMConfig() {
  return adminHttp.get("/api/v1/admin/llm-config");
}

export function adminPutLLMConfig(payload) {
  return adminHttp.put("/api/v1/admin/llm-config", payload);
}

// ---------- 评论管理 ----------

export function adminListComments(params) {
  return adminHttp.get("/api/v1/admin/comments", { params });
}

export function adminGetComment(id, type) {
  return adminHttp.get(`/api/v1/admin/comments/${id}`, { params: { type } });
}

export function adminDeleteComment(id, type) {
  return adminHttp.post(`/api/v1/admin/comments/${id}/delete`, null, { params: { type } });
}

// ---------- 系统设置 ----------

export function adminGetSettings() {
  return adminHttp.get("/api/v1/admin/settings");
}

export function adminPutSettings(payload) {
  return adminHttp.put("/api/v1/admin/settings", payload);
}

// ---------- 数据仪表盘 ----------

export function adminGetDashboard() {
  return adminHttp.get("/api/v1/admin/dashboard");
}

// ---------- 用户管理 ----------

export function adminListUsers(params) {
  return adminHttp.get("/api/v1/admin/users", { params });
}

export function adminGetUser(id) {
  return adminHttp.get(`/api/v1/admin/users/${id}`);
}

export function adminBanUser(id, reason) {
  return adminHttp.post(`/api/v1/admin/users/${id}/ban`, { reason });
}

export function adminUnbanUser(id) {
  return adminHttp.post(`/api/v1/admin/users/${id}/unban`);
}

export function adminDeleteUser(id) {
  return adminHttp.post(`/api/v1/admin/users/${id}/delete`);
}

// ---------- 举报处理 ----------

export function adminListReports(params) {
  return adminHttp.get("/api/v1/admin/reports", { params });
}

export function adminHandleReport(id, payload) {
  return adminHttp.post(`/api/v1/admin/reports/${id}/handle`, payload);
}

export function adminBatchHandleReports(payload) {
  return adminHttp.post("/api/v1/admin/reports/batch", payload);
}

export function adminDeleteReport(id) {
  return adminHttp.delete(`/api/v1/admin/reports/${id}`);
}

// ---------- RBAC / 权限审计 ----------

export function adminListRoles() {
  return adminHttp.get("/api/v1/admin/rbac/roles");
}

export function adminCreateRole(payload) {
  return adminHttp.post("/api/v1/admin/rbac/roles", payload);
}

export function adminUpdateRole(id, payload) {
  return adminHttp.put(`/api/v1/admin/rbac/roles/${id}`, payload);
}

export function adminDeleteRole(id) {
  return adminHttp.delete(`/api/v1/admin/rbac/roles/${id}`);
}

export function adminGetRole(id) {
  return adminHttp.get(`/api/v1/admin/rbac/roles/${id}`);
}

export function adminAssignPermissions(roleId, permissions) {
  return adminHttp.post(`/api/v1/admin/rbac/roles/${roleId}/permissions`, { permissions });
}

export function adminListAdmins(params) {
  return adminHttp.get("/api/v1/admin/rbac/admins", { params });
}

export function adminCreateAdmin(payload) {
  return adminHttp.post("/api/v1/admin/rbac/admins", payload);
}

export function adminAssignRole(adminId, roleId) {
  return adminHttp.post(`/api/v1/admin/rbac/admins/${adminId}/role`, { role_id: roleId });
}

export function adminListAuditLogs(params) {
  return adminHttp.get("/api/v1/admin/rbac/audit-logs", { params });
}

export function adminListApprovals(params) {
  return adminHttp.get("/api/v1/admin/rbac/approval-flows", { params });
}

export function adminApproveFlow(id) {
  return adminHttp.post(`/api/v1/admin/rbac/approval-flows/${id}/approve`);
}

export function adminRejectFlow(id) {
  return adminHttp.post(`/api/v1/admin/rbac/approval-flows/${id}/reject`);
}

// ---------- 客服管理 ----------

export function adminListConversations(params) {
  return adminHttp.get("/api/v1/admin/cs/conversations", { params });
}

export function adminGetConversation(id) {
  return adminHttp.get(`/api/v1/admin/cs/conversations/${id}`);
}

export function adminSendConversationMessage(id, content) {
  return adminHttp.post(`/api/v1/admin/cs/conversations/${id}/messages`, { content });
}

export function adminCloseConversation(id) {
  return adminHttp.post(`/api/v1/admin/cs/conversations/${id}/close`);
}

export function adminAssignConversation(id) {
  return adminHttp.post(`/api/v1/admin/cs/conversations/${id}/assign`);
}

export function adminListCsTemplates() {
  return adminHttp.get("/api/v1/admin/cs/templates");
}

export function adminCreateCsTemplate(payload) {
  return adminHttp.post("/api/v1/admin/cs/templates", payload);
}

export function adminUpdateCsTemplate(id, payload) {
  return adminHttp.put(`/api/v1/admin/cs/templates/${id}`, payload);
}

export function adminDeleteCsTemplate(id) {
  return adminHttp.delete(`/api/v1/admin/cs/templates/${id}`);
}

// ---------- 工单管理 ----------

export function adminListTickets(params) {
  return adminHttp.get("/api/v1/admin/tickets", { params });
}

export function adminGetTicket(id) {
  return adminHttp.get(`/api/v1/admin/tickets/${id}`);
}

export function adminAssignTicket(id, adminId) {
  return adminHttp.post(`/api/v1/admin/tickets/${id}/assign`, { admin_id: adminId });
}

export function adminAutoAssignTicket(id) {
  return adminHttp.post(`/api/v1/admin/tickets/${id}/auto-assign`);
}

export function adminUpdateTicketStatus(id, status) {
  return adminHttp.post(`/api/v1/admin/tickets/${id}/status`, { status });
}

export function adminTicketSendMessage(id, content) {
  return adminHttp.post(`/api/v1/admin/tickets/${id}/messages`, { content });
}

export function adminCloseTicket(id) {
  return adminHttp.post(`/api/v1/admin/tickets/${id}/close`);
}

export function adminReopenTicket(id) {
  return adminHttp.post(`/api/v1/admin/tickets/${id}/reopen`);
}

// ---------- 版权管理 ----------

export function adminListCopyrightComplaints(params) {
  return adminHttp.get("/api/v1/admin/copyright/complaints", { params });
}

export function adminGetCopyrightComplaint(id) {
  return adminHttp.get(`/api/v1/admin/copyright/complaints/${id}`);
}

export function adminAcceptCopyrightComplaint(id) {
  return adminHttp.post(`/api/v1/admin/copyright/complaints/${id}/accept`);
}

export function adminRejectCopyrightComplaint(id, comment) {
  return adminHttp.post(`/api/v1/admin/copyright/complaints/${id}/reject`, { handler_comment: comment || '' });
}

export function adminTakedownCopyright(id) {
  return adminHttp.post(`/api/v1/admin/copyright/complaints/${id}/takedown`);
}

export function adminRestoreCopyright(id) {
  return adminHttp.post(`/api/v1/admin/copyright/complaints/${id}/restore`);
}

// ---------- 自用 ----------

export function adminGetMyPermissions() {
  return adminHttp.get("/api/v1/admin/rbac/me/permissions");
}

export function adminGetUserViolations(uid) {
  return adminHttp.get(`/api/v1/admin/users/${uid}/violations`);
}

// ---------- 专题 & 活动 ----------

export function adminListSpecials() {
  return adminHttp.get("/api/v1/admin/specials");
}
export function adminCreateSpecial(payload) {
  return adminHttp.post("/api/v1/admin/specials", payload);
}
export function adminUpdateSpecial(id, payload) {
  return adminHttp.put(`/api/v1/admin/specials/${id}`, payload);
}
export function adminDeleteSpecial(id) {
  return adminHttp.delete(`/api/v1/admin/specials/${id}`);
}
export function adminListCampaigns() {
  return adminHttp.get("/api/v1/admin/campaigns");
}
export function adminCreateCampaign(payload) {
  return adminHttp.post("/api/v1/admin/campaigns", payload);
}
export function adminUpdateCampaign(id, payload) {
  return adminHttp.put(`/api/v1/admin/campaigns/${id}`, payload);
}
export function adminDeleteCampaign(id) {
  return adminHttp.delete(`/api/v1/admin/campaigns/${id}`);
}
