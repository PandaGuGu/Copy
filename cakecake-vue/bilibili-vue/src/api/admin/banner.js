import adminHttp from "@/utils/adminHttp";
import http from "@/utils/http";

const isMinibili = import.meta.env.VITE_MINIBILI_API === "true" || import.meta.env.VITE_MINIBILI_API === "1";

export function adminListBanners() { return adminHttp.get("/api/v1/admin/home-banners"); }
export function adminCreateBanner(payload) { return adminHttp.post("/api/v1/admin/home-banners", payload); }
export function adminUpdateBanner(id, payload) { return adminHttp.put(`/api/v1/admin/home-banners/${id}`, payload); }
export function adminDeleteBanner(id) { return adminHttp.delete(`/api/v1/admin/home-banners/${id}`); }
export function adminUploadBannerImage(file, bannerId) {
  const fd = new FormData(); fd.append("image", file);
  const opts = { timeout: 120000, skipGlobalErrorToast: false };
  if (bannerId) return adminHttp.post(`/api/v1/admin/home-banners/${bannerId}/image`, fd, opts);
  return adminHttp.post("/api/v1/admin/home-banners/upload-image", fd, opts);
}
export function getHomeBannersPublic() {
  if (!isMinibili) return Promise.resolve({ code: 0, data: { items: [] } });
  return http.get("/api/v1/home-banners", { skipGlobalErrorToast: true });
}
