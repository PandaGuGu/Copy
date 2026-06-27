import adminHttp from "@/utils/adminHttp";

export function adminListCopyrightComplaints(params) { return adminHttp.get("/api/v1/admin/copyright/complaints", { params }); }
export function adminGetCopyrightComplaint(id) { return adminHttp.get(`/api/v1/admin/copyright/complaints/${id}`); }
export function adminAcceptCopyrightComplaint(id) { return adminHttp.post(`/api/v1/admin/copyright/complaints/${id}/accept`); }
export function adminRejectCopyrightComplaint(id, comment) { return adminHttp.post(`/api/v1/admin/copyright/complaints/${id}/reject`, { handler_comment: comment || '' }); }
export function adminTakedownCopyright(id) { return adminHttp.post(`/api/v1/admin/copyright/complaints/${id}/takedown`); }
export function adminRestoreCopyright(id) { return adminHttp.post(`/api/v1/admin/copyright/complaints/${id}/restore`); }
