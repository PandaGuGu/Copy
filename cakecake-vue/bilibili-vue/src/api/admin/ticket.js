import adminHttp from "@/utils/adminHttp";

export function adminListTickets(params) { return adminHttp.get("/api/v1/admin/tickets", { params }); }
export function adminGetTicket(id) { return adminHttp.get(`/api/v1/admin/tickets/${id}`); }
export function adminAssignTicket(id, adminId) { return adminHttp.post(`/api/v1/admin/tickets/${id}/assign`, { admin_id: adminId }); }
export function adminAutoAssignTicket(id) { return adminHttp.post(`/api/v1/admin/tickets/${id}/auto-assign`); }
export function adminUpdateTicketStatus(id, status) { return adminHttp.post(`/api/v1/admin/tickets/${id}/status`, { status }); }
export function adminTicketSendMessage(id, content) { return adminHttp.post(`/api/v1/admin/tickets/${id}/messages`, { content }); }
export function adminCloseTicket(id) { return adminHttp.post(`/api/v1/admin/tickets/${id}/close`); }
export function adminReopenTicket(id) { return adminHttp.post(`/api/v1/admin/tickets/${id}/reopen`); }
