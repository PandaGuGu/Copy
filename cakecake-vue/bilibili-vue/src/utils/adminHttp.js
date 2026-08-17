import axios from "axios";
import { ElMessage } from "element-plus";
import {
  clearAdminTokens,
  getAdminAccessToken,
  getAdminRefreshToken,
  setAdminTokens
} from "./adminAuth";

const remoteRaw = import.meta.env.VITE_REMOTE_API_BASE;
const baseURL =
  remoteRaw && String(remoteRaw).trim() !== ""
    ? String(remoteRaw).replace(/\/$/, "")
    : "";

const adminHttp = axios.create({
  baseURL,
  timeout: 60000
});

adminHttp.interceptors.request.use(config => {
  const t = getAdminAccessToken();
  if (t) {
    config.headers.Authorization = `Bearer ${t}`;
  }
  if (config.data instanceof FormData && config.headers) {
    delete config.headers["Content-Type"];
    delete config.headers["content-type"];
  }
  return config;
});

let refreshPromise = null;

async function refreshAdminToken() {
  const rt = getAdminRefreshToken();
  if (!rt) {
    return false;
  }
  try {
    const res = await axios.post(`${baseURL}/api/v1/admin/auth/refresh`, {
      refresh_token: rt
    });
    const body = res.data;
    if (!body || body.code !== 0 || !body.data) {
      return false;
    }
    setAdminTokens(body.data.access_token, body.data.refresh_token);
    return true;
  } catch (e) {
    // refresh 本身也可能 401/网络失败 —— 必须吞掉异常，否则外层 await 会抛错
    // 导致 clearAdminTokens() 与登录页跳转永远执行不到（401 死循环）
    return false;
  }
}

adminHttp.interceptors.response.use(
  res => {
    const body = res.data;
    if (body && typeof body.code === "number" && body.code !== 0) {
      const err = new Error(body.msg || "请求失败");
      err.minibiliApiCode = body.code;
      return Promise.reject(err);
    }
    return body;
  },
  async err => {
    const cfg = err.config || {};
    const st = err.response && err.response.status;
    if (st === 401 && !cfg._adminRetry) {
      if (!refreshPromise) {
        refreshPromise = refreshAdminToken().finally(() => {
          refreshPromise = null;
        });
      }
      const ok = await refreshPromise.catch(() => false);
      if (ok) {
        cfg._adminRetry = true;
        cfg.headers = cfg.headers || {};
        cfg.headers.Authorization = `Bearer ${getAdminAccessToken()}`;
        return adminHttp(cfg);
      }
      clearAdminTokens();
      if (typeof window !== "undefined") {
        // 路由为 history 模式（createWebHistory），hash 跳转无效，必须用 href 整页跳转
        window.location.href = "/admin/login";
      }
    }
    const msg =
      (err.response && err.response.data && err.response.data.msg) ||
      err.message ||
      "请求失败";
    if (!cfg.skipGlobalErrorToast) {
      ElMessage.error(msg);
    }
    return Promise.reject(err);
  }
);

export default adminHttp;
