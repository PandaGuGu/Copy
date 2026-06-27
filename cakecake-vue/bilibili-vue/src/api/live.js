import http from "@/utils/http";
import adminHttp from "@/utils/adminHttp";

// ---- 用户端 ----

export function listLiveRooms(params = {}) {
  const qs = new URLSearchParams();
  if (params.status) qs.set("status", params.status);
  if (params.page) qs.set("page", String(params.page));
  if (params.page_size) qs.set("page_size", String(params.page_size));
  const query = qs.toString();
  return http.get(`/api/v1/live/rooms${query ? "?" + query : ""}`);
}

export function getLiveRoom(roomId) {
  return http.get(`/api/v1/live/room/${roomId}`);
}

export function createLiveRoom(payload) {
  return http.post("/api/v1/live/room/create", payload);
}

export function getMyLiveRoom() {
  return http.get("/api/v1/live/room/my");
}

export function updateLiveRoom(roomId, payload) {
  return http.put(`/api/v1/live/room/${roomId}`, payload);
}

export function startLiveRoom(roomId) {
  return http.post(`/api/v1/live/room/${roomId}/start`);
}

export function endLiveRoom(roomId) {
  return http.post(`/api/v1/live/room/${roomId}/end`);
}

export function regenerateStreamKey(roomId) {
  return http.post(`/api/v1/live/room/${roomId}/regenerate-key`);
}

export function uploadLiveCover(roomId, file) {
  const fd = new FormData();
  fd.append("cover", file);
  return http.post(`/api/v1/live/room/${roomId}/cover`, fd);
}

// ---- 管理端 ----

export function adminListLiveRooms(params = {}) {
  const qs = new URLSearchParams();
  if (params.status) qs.set("status", params.status);
  if (params.page) qs.set("page", String(params.page));
  if (params.page_size) qs.set("page_size", String(params.page_size));
  const query = qs.toString();
  return adminHttp.get(`/api/v1/admin/live/rooms${query ? "?" + query : ""}`);
}

export function adminGetLiveRoom(roomId) {
  return adminHttp.get(`/api/v1/admin/live/room/${roomId}`);
}

export function adminBanLiveRoom(roomId) {
  return adminHttp.post(`/api/v1/admin/live/room/${roomId}/ban`);
}

export function adminUnbanLiveRoom(roomId) {
  return adminHttp.post(`/api/v1/admin/live/room/${roomId}/unban`);
}

export function adminDeleteLiveRoom(roomId) {
  return adminHttp.delete(`/api/v1/admin/live/room/${roomId}`);
}

export function adminWarnLiveRoom(roomId, reason) {
  return adminHttp.post(`/api/v1/admin/live/room/${roomId}/warn`, { reason });
}

// 警告模板管理
export function listWarnTemplates() {
  return adminHttp.get("/api/v1/admin/live/warn-templates");
}
export function createWarnTemplate(data) {
  return adminHttp.post("/api/v1/admin/live/warn-templates", data);
}
export function updateWarnTemplate(id, data) {
  return adminHttp.put(`/api/v1/admin/live/warn-templates/${id}`, data);
}
export function deleteWarnTemplate(id) {
  return adminHttp.delete(`/api/v1/admin/live/warn-templates/${id}`);
}
