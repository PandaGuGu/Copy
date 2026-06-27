/**
 * Admin panel shared utilities.
 * Import in any admin page: import { formatTime } from "@/utils/admin-helpers";
 */

export function formatTime(t) {
  if (!t) return "—";
  const d = new Date(t);
  if (Number.isNaN(d.getTime())) return String(t);
  const pad = (n) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
