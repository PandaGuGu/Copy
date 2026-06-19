import { getAccessToken } from "./authTokens";

export function isMinibiliApiEnv() {
  return (
    import.meta.env.VITE_MINIBILI_API === "true" ||
    import.meta.env.VITE_MINIBILI_API === "1"
  );
}

/**
 * 投稿按钮始终跳转 /upload 页面，不再弹登录弹窗。
 */
export function minibiliUploadOpensLoginModal() {
  return false;
}

/**
 * 投稿按钮始终跳转 /upload（创作中心）页面。
 */
export function resolveMinibiliUploadNavTo() {
  return { name: "upload" };
}
