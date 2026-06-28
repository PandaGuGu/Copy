// Barrel file — re-exports admin API functions from per-module files.
// Add new admin endpoints in the matching module under api/admin/.
export { adminLogin, adminMe } from "./admin/auth";
export {
  adminListBanners, adminCreateBanner, adminUpdateBanner,
  adminDeleteBanner, adminUploadBannerImage, getHomeBannersPublic,
} from "./admin/banner";
export {
  adminListHotSearchOps, adminCreateHotSearchOp, adminUpdateHotSearchOp,
  adminDeleteHotSearchOp, adminPreviewHotSearch, adminHotSearchDashboard,
  adminRemoveHotSearchRedis, adminBoostHotSearchRedis,
  adminQuickHotSearchOp, adminReorderHotSearch, adminResetHotSearchDisplayOrder,
} from "./admin/hot-search";
export {
  adminListVideos, adminGetVideo, adminApproveVideo,
  adminRejectVideo, adminDeleteVideo, adminBatchApproveVideos,
} from "./admin/video";
export {
  adminListArticles, adminGetArticle, adminApproveArticle,
  adminRejectArticle, adminDeleteArticle,
} from "./admin/article";
export { adminListDynamics, adminGetDynamic, adminDeleteDynamic } from "./admin/dynamic";
export {
  adminGetAgentSettings, adminPutAgentSettings, adminUploadAgentAvatar,
  adminListAgentProfiles, adminCreateAgentProfile, adminUpdateAgentProfile,
  adminDeleteAgentProfile, adminUploadAgentProfileAvatar,
  adminGetLLMConfig, adminPutLLMConfig,
  adminListLLMProviders, adminCreateLLMProvider, adminUpdateLLMProvider,
  adminDeleteLLMProvider, adminSetDefaultLLMProvider,
} from "./admin/agent";
export {
  adminListComments, adminGetComment, adminDeleteComment,
} from "./admin/comment";
export {
  adminListUsers, adminGetUser, adminBanUser, adminUnbanUser,
  adminDeleteUser, adminGetUserViolations,
} from "./admin/user";
export {
  adminListReports, adminHandleReport, adminBatchHandleReports, adminDeleteReport,
} from "./admin/report";
export { adminGetSettings, adminPutSettings } from "./admin/settings";
export { adminGetDashboard } from "./admin/dashboard";
export {
  adminListRoles, adminCreateRole, adminUpdateRole, adminDeleteRole,
  adminGetRole, adminAssignPermissions,
  adminListAdmins, adminCreateAdmin, adminAssignRole,
  adminListAuditLogs, adminListApprovals, adminApproveFlow, adminRejectFlow,
  adminGetMyPermissions,
} from "./admin/rbac";
export {
  adminListConversations, adminGetConversation, adminSendConversationMessage,
  adminCloseConversation, adminAssignConversation,
  adminListCsTemplates, adminCreateCsTemplate, adminUpdateCsTemplate, adminDeleteCsTemplate,
} from "./admin/cs";
export {
  adminListTickets, adminGetTicket, adminAssignTicket, adminAutoAssignTicket,
  adminUpdateTicketStatus, adminTicketSendMessage, adminCloseTicket, adminReopenTicket,
} from "./admin/ticket";
export {
  adminListCopyrightComplaints, adminGetCopyrightComplaint,
  adminAcceptCopyrightComplaint, adminRejectCopyrightComplaint,
  adminTakedownCopyright, adminRestoreCopyright,
} from "./admin/copyright";
export {
  adminListSpecials, adminCreateSpecial, adminUpdateSpecial, adminDeleteSpecial,
  adminListCampaigns, adminCreateCampaign, adminUpdateCampaign, adminDeleteCampaign,
} from "./admin/special";
