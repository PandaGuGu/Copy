const K_ACCESS = "minibili_admin_access_token";
const K_REFRESH = "minibili_admin_refresh_token";
const K_PERMS = "minibili_admin_perms";

export function getAdminAccessToken() {
  return localStorage.getItem(K_ACCESS) || "";
}

export function getAdminRefreshToken() {
  return localStorage.getItem(K_REFRESH) || "";
}

export function setAdminTokens(access, refresh) {
  if (access) {
    localStorage.setItem(K_ACCESS, access);
  }
  if (refresh) {
    localStorage.setItem(K_REFRESH, refresh);
  }
}

export function clearAdminTokens() {
  localStorage.removeItem(K_ACCESS);
  localStorage.removeItem(K_REFRESH);
  localStorage.removeItem(K_PERMS);
}

export function isAdminLoggedIn() {
  return !!getAdminAccessToken();
}

// ── RBAC permission cache (synced by AdminLayout.fetchPerms) ──
export function getAdminPerms() {
  try {
    return JSON.parse(localStorage.getItem(K_PERMS) || "[]");
  } catch {
    return [];
  }
}

export function setAdminPerms(perms) {
  localStorage.setItem(K_PERMS, JSON.stringify(perms || []));
}
