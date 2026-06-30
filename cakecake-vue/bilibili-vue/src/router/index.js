import { createRouter, createWebHashHistory } from "vue-router";
import { nextTick } from "vue";
import { ElMessageBox } from "element-plus";
import { clearStuckPageOverlays } from "@/utils/clearPageOverlays";
import { getAccessToken, getRefreshToken } from "@/utils/authTokens";
import { isAdminLoggedIn, getAdminPerms } from "@/utils/adminAuth";
import {
  isAccessTokenExpired,
  refreshMinibiliAccessToken
} from "@/utils/minibiliTokenRefresh";
import { shouldRedirectVideoToNotFound } from "@/utils/notFoundRedirect";
import {
  SITE_HOME_TITLE,
  buildDocumentTitle
} from "@/constants/siteTitle";

const minibiliEnv =
  import.meta.env.VITE_MINIBILI_API === "true" ||
  import.meta.env.VITE_MINIBILI_API === "1";

const routes = [
  {
    name: "home",
    path: "/",
    component: () => import("@/pages/home/index.vue"),
    meta: {
      title: SITE_HOME_TITLE
    }
  },
  {
    name: "Ranking",
    path: "/ranking",
    component: () => import("@/pages/ranking/ranking.vue"),
    redirect: "/ranking/all/0/0/0",
    children: [
      {
        name: "rankingDetail",
        path: ":type/:rid/:rankselect/:rankselect2",
        component: () => import("@/components/ranking/allList.vue"),
        meta: {
          title: buildDocumentTitle("热门视频排行榜")
        }
      }
    ]
  },
  {
    path: "/search",
    component: () => import("@/pages/search/search.vue"),
    redirect: "/search/all",
    children: [
      {
        name: "searchAll",
        path: "all",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      },
      {
        name: "searchVideo",
        path: "video",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      },
      {
        name: "searchBangumi",
        path: "bangumi",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      },
      {
        name: "searchPgc",
        path: "pgc",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      },
      {
        name: "searchLive",
        path: "live",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      },
      {
        name: "searchArticle",
        path: "article",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      },
      {
        name: "searchTopic",
        path: "topic",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      },
      {
        name: "upuser",
        path: "upuser",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      },
      {
        name: "photo",
        path: "photo",
        component: () => import("@/components/search/searchList.vue"),
        meta: {
          title: buildDocumentTitle("搜索结果")
        }
      }
    ]
  },
  {
    name: "zonePage",
    path: "/zone/:zoneName",
    component: () => import("@/pages/zone/ZonePage.vue"),
    meta: { title: (route) => `${route.params.zoneName}分区` }
  },
  {
    name: "articlePage",
    path: "/article",
    component: () => import("@/pages/article/ArticlePage.vue"),
    meta: { title: "专栏" }
  },
  {
    name: "activityPage",
    path: "/activity",
    component: () => import("@/pages/activity/ActivityPage.vue"),
    meta: { title: "活动" }
  },
  {
    name: "specialDetail",
    path: "/special/:slug",
    component: () => import("@/pages/special/SpecialDetail.vue"),
    meta: { title: "专题详情" }
  },
  {
    name: "classPage",
    path: "/class",
    component: () => import("@/pages/class/ClassPage.vue"),
    meta: { title: "课堂" }
  },
  {
    name: "communityPage",
    path: "/community",
    component: () => import("@/pages/community/CommunityPage.vue"),
    meta: { title: "社区中心" }
  },
  {
    name: "trendingPage",
    path: "/trending",
    component: () => import("@/pages/trending/TrendingPage.vue"),
    meta: { title: "热门" }
  },
  {
    name: "video",
    path: "/video/:aid",
    component: () => import("@/pages/video/video.vue"),
    meta: {
      title: ":aid - " + SITE_HOME_TITLE
    }
  },
  {
    name: "upload",
    path: "/upload",
    component: () => import("@/pages/upload/upload.vue"),
    meta: { title: buildDocumentTitle("创作中心") }
  },
  {
    name: "videoPublish",
    path: "/upload/publish",
    component: () => import("@/pages/upload/videoPublish.vue"),
    meta: { title: "投稿视频 - 创作中心", requireMinibiliAuth: true }
  },
  {
    name: "videoEdit",
    path: "/upload/edit/:id",
    component: () => import("@/pages/upload/videoPublish.vue"),
    meta: { title: "编辑视频 - 创作中心", requireMinibiliAuth: true }
  },
  {
    name: "articlePublish",
    path: "/upload/article/publish",
    component: () => import("@/pages/upload/articlePublish.vue"),
    meta: { title: "专栏投稿 - 创作中心", requireMinibiliAuth: true }
  },
  {
    name: "articleEdit",
    path: "/upload/article/edit/:id",
    component: () => import("@/pages/upload/articlePublish.vue"),
    meta: { title: "编辑专栏 - 创作中心", requireMinibiliAuth: true }
  },
  {
    name: "manuscript",
    path: "/upload/manuscript",
    component: () => import("@/pages/upload/manuscript.vue"),
    meta: { title: "稿件管理 - 创作中心" }
  },
  {
    name: "appeal",
    path: "/upload/appeal",
    component: () => import("@/pages/upload/appeal.vue"),
    meta: { title: "申诉管理 - 创作中心" }
  },
  {
    name: "creatorComments",
    path: "/upload/comments",
    component: () => import("@/pages/upload/commentManage.vue"),
    meta: { title: "评论管理 - 创作中心" }
  },
  {
    name: "creatorDanmakus",
    path: "/upload/danmakus",
    component: () => import("@/pages/upload/danmakuManage.vue"),
    meta: { title: "弹幕管理 - 创作中心" }
  },
  {
    name: "creatorDashboard",
    path: "/upload/dashboard",
    component: () => import("@/pages/upload/CreatorDashboard.vue"),
    meta: { title: "数据中心 - 创作中心", requireMinibiliAuth: true }
  },
  {
    path: "/upload/subtitles/:videoId",
    component: () => import("@/pages/minibili/SubtitleEdit.vue"),
    props: (route) => ({ videoId: Number(route.params.videoId) || 0 }),
    meta: { title: "字幕管理 - 创作中心", requireMinibiliAuth: true }
  },
  {
    path: "/minibili/login",
    name: "minibiliLogin",
    component: () => import("@/pages/minibili/Login.vue"),
    meta: { title: "cakecake 登录" }
  },
  {
    path: "/minibili/register",
    name: "minibiliRegister",
    component: () => import("@/pages/minibili/Register.vue"),
    meta: { title: "cakecake 注册" }
  },
  {
    name: "minibiliMessages",
    path: "/minibili/messages",
    component: () => import("@/pages/minibili/Messages.vue"),
    meta: { title: "cakecake 消息", requireMinibiliAuth: true }
  },
  {
    name: "minibiliPersonalCenter",
    path: "/minibili/account",
    component: () => import("@/pages/minibili/PersonalCenter.vue"),
    meta: { title: "个人中心 - cakecake", requireMinibiliAuth: true }
  },
  {
    name: "minibiliUserSpace",
    path: "/minibili/up/:userId",
    component: () => import("@/pages/minibili/PersonalSpace.vue"),
    meta: { title: "个人空间 - cakecake" }
  },
  {
    name: "minibiliUserSpaceRelations",
    path: "/minibili/up/:userId/relations",
    component: () => import("@/pages/minibili/SpaceRelations.vue"),
    meta: { title: "关注与粉丝 - cakecake" }
  },
  {
    name: "minibiliWatchLater",
    path: "/minibili/watch-later",
    component: () => import("@/pages/minibili/WatchLater.vue"),
    meta: { title: "稍后再看 - cakecake" }
  },
  {
    name: "minibiliDynamics",
    path: "/minibili/dynamics",
    component: () => import("@/pages/minibili/Dynamics.vue"),
    meta: { title: "动态 - cakecake", requireMinibiliAuth: true }
  },
  {
    name: "minibiliArticleRead",
    path: "/minibili/article/:id",
    component: () => import("@/pages/minibili/ArticleRead.vue"),
    meta: { title: "专栏 - cakecake" }
  },
  {
    name: "minibiliDynamicRead",
    path: "/minibili/dynamic/:id",
    component: () => import("@/pages/minibili/ArticleRead.vue"),
    meta: { title: "动态 - cakecake" }
  },
  {
    name: "minibiliViewHistory",
    path: "/minibili/history",
    component: () => import("@/pages/minibili/ViewHistory.vue"),
    meta: { title: "历史记录 - cakecake", requireMinibiliAuth: true }
  },
  {
    name: "minibiliUpload",
    path: "/minibili/upload",
    component: () => import("@/pages/minibili/Upload.vue"),
    meta: { title: "cakecake 上传", requireMinibiliAuth: true }
  },
  {
    name: "minibiliTicketCreate",
    path: "/minibili/ticket",
    component: () => import("@/pages/minibili/TicketCreate.vue"),
    meta: { title: "提交工单 - cakecake", requireMinibiliAuth: true }
  },
  {
    name: "customerService",
    path: "/customer-service",
    component: () => import("@/pages/minibili/CustomerService.vue"),
    meta: { title: "客服中心 - cakecake", requireMinibiliAuth: true }
  },
  {
    name: "csChat",
    path: "/cs-chat",
    component: () => import("@/pages/minibili/CSChat.vue"),
    meta: { title: "客服聊天 - cakecake", requireMinibiliAuth: true }
  },
  {
    name: "minibiliSpecialPage",
    path: "/minibili/special/:slug",
    component: () => import("@/pages/minibili/SpecialPage.vue"),
    meta: { title: "专题 - cakecake" }
  },
  // === 直播模块 ===
  {
    name: "minibiliLiveList",
    path: "/minibili/live",
    component: () => import("@/pages/live/LivePage.vue"),
    meta: { title: "直播 - cakecake" }
  },
  {
    name: "minibiliLiveRoom",
    path: "/minibili/live/:roomId",
    component: () => import("@/pages/minibili/LiveRoom.vue"),
    meta: { title: "直播间 - cakecake" }
  },
  {
    name: "minibiliLiveCreate",
    path: "/minibili/live/create",
    component: () => import("@/pages/minibili/LiveCreate.vue"),
    meta: { title: "开播设置 - cakecake", requireMinibiliAuth: true }
  },
  {
    name: "minibiliLiveSettings",
    path: "/minibili/live/:roomId/settings",
    component: () => import("@/pages/minibili/LiveSettings.vue"),
    meta: { title: "直播间设置 - cakecake", requireMinibiliAuth: true }
  },
  {
    path: "/admin/login",
    name: "adminLogin",
    component: () => import("@/pages/admin/AdminLogin.vue"),
    meta: { title: "运营后台登录", hideGlobalChrome: true }
  },
  {
    path: "/admin",
    component: () => import("@/pages/admin/AdminLayout.vue"),
    meta: { hideGlobalChrome: true, requireAdminAuth: true },
    children: [
      { path: "", redirect: { name: "adminDashboard" } },
      {
        path: "dashboard",
        name: "adminDashboard",
        component: () => import("@/pages/admin/data/Dashboard.vue"),
        meta: { title: "数据仪表盘 - 运营后台", perm: "dashboard:view" }
      },
      {
        path: "banners",
        name: "adminBanners",
        component: () => import("@/pages/admin/content/BannerManage.vue"),
        meta: { title: "首页轮播 - 运营后台", perm: "banner:manage" }
      },
      {
        path: "hot-search",
        name: "adminHotSearch",
        component: () => import("@/pages/admin/content/HotSearchManage.vue"),
        meta: { title: "热搜运营 - 运营后台", perm: "hotsearch:manage" }
      },
      {
        path: "users",
        name: "adminUsers",
        component: () => import("@/pages/admin/UserManage.vue"),
        meta: { title: "用户管理 - 运营后台", perm: "user:ban" }
      },
      {
        path: "video-review",
        name: "adminVideoReview",
        component: () => import("@/pages/admin/review/VideoReview.vue"),
        meta: { title: "视频审核 - 运营后台", perm: "video:approve" }
      },
      {
        path: "article-review",
        name: "adminArticleReview",
        component: () => import("@/pages/admin/review/ArticleReview.vue"),
        meta: { title: "专栏审核 - 运营后台", perm: "article:approve" }
      },
      {
        path: "dynamic-manage",
        name: "adminDynamicManage",
        component: () => import("@/pages/admin/DynamicManage.vue"),
        meta: { title: "动态管理 - 运营后台", perm: "dynamic:manage" }
      },
      {
        path: "comments",
        name: "adminComments",
        component: () => import("@/pages/admin/social/CommentManage.vue"),
        meta: { title: "评论管理 - 运营后台", perm: "comment:delete" }
      },
      {
        path: "settings",
        name: "adminSettings",
        component: () => import("@/pages/admin/Settings.vue"),
        meta: { title: "系统设置 - 运营后台", perm: "setting:manage" }
      },
      {
        path: "reports",
        name: "adminReports",
        component: () => import("@/pages/admin/social/ReportManage.vue"),
        meta: { title: "举报处理 - 运营后台", perm: "report:handle" }
      },
      {
        path: "agent",
        name: "adminAgent",
        component: () => import("@/pages/admin/AgentManage.vue"),
        meta: { title: "AI 角色 - 运营后台", perm: "agent:manage" }
      },
      // ─── 23-module expansion: new admin routes ───
      {
        path: "tickets",
        name: "adminTicketManage",
        component: () => import("@/pages/admin/social/TicketManage.vue"),
        meta: { title: "工单管理 - 运营后台", perm: "ticket:handle" }
      },
      {
        path: "risk",
        name: "adminRiskManage",
        component: () => import("@/pages/admin/social/RiskManage.vue"),
        meta: { title: "风控管理 - 运营后台", perm: "risk:manage" }
      },
      {
        path: "copyright",
        name: "adminCopyrightManage",
        component: () => import("@/pages/admin/social/CopyrightManage.vue"),
        meta: { title: "版权管理 - 运营后台", perm: "copyright:handle" }
      },
      {
        path: "bi",
        name: "adminBIReport",
        component: () => import("@/pages/admin/data/BIReport.vue"),
        meta: { title: "数据报表 - 运营后台", perm: "dashboard:export" }
      },
      {
        path: "cs",
        name: "adminCSManage",
        component: () => import("@/pages/admin/social/CSManage.vue"),
        meta: { title: "客服后台 - 运营后台", perm: "cs:manage" }
      },
      {
        path: "ops",
        name: "adminOpsMonitor",
        component: () => import("@/pages/admin/ops/OpsMonitor.vue"),
        meta: { title: "运维监控 - 运营后台", perm: "ops:manage" }
      },
      {
        path: "config",
        name: "adminConfigManage",
        component: () => import("@/pages/admin/ops/ConfigManage.vue"),
        meta: { title: "配置发布 - 运营后台", perm: "config:manage" }
      },
      {
        path: "rbac",
        name: "adminRBACManage",
        component: () => import("@/pages/admin/ops/RBACManage.vue"),
        meta: { title: "权限审计 - 运营后台", perm: "rbac:manage" }
      },
      {
        path: "subtitles",
        name: "adminSubtitleManage",
        component: () => import("@/pages/admin/content/SubtitleManage.vue"),
        meta: { title: "字幕管理 - 运营后台", perm: "subtitle:manage" }
      },
      {
        path: "player-advanced",
        name: "adminPlayerAdvanced",
        component: () => import("@/pages/admin/media/PlayerAdvanced.vue"),
        meta: { title: "播放器高级 - 运营后台", perm: "video:approve" }
      },
      {
        path: "specials",
        name: "adminSpecialManage",
        component: () => import("@/pages/admin/content/SpecialManage.vue"),
        meta: { title: "专题活动 - 运营后台", perm: "special:manage" }
      },
      {
        path: "live",
        name: "adminLiveManage",
        component: () => import("@/pages/admin/review/LiveManage.vue"),
        meta: { title: "直播管理 - 运营后台", perm: "live:manage" }
      }
    ]
  },
  {
    path: "/404",
    name: "notFound",
    component: () => import("@/pages/notFound/404.vue"),
    meta: {
      title: "页面不存在 - cakecake"
    }
  },
  {
    path: "/:pathMatch(.*)*",
    redirect: "/404"
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior(_to, _from, savedPosition) {
    if (savedPosition) {
      return savedPosition;
    }
    return { top: 0, left: 0 };
  }
});

/** cakecake：未登录访问需鉴权页时跳转首页 */
router.beforeEach(async (to, _from, next) => {
  if (!document.querySelector(".mm-del-overlay")) {
    clearStuckPageOverlays();
  }
  const needAdmin = to.matched.some(
    r => r.meta && r.meta.requireAdminAuth === true
  );
  if (needAdmin && !isAdminLoggedIn()) {
    next({ name: "adminLogin", replace: true });
    return;
  }
  // RBAC permission guard: redirect to dashboard if admin lacks permission for this route
  if (needAdmin && to.meta.perm) {
    const perms = getAdminPerms();
    if (perms.length > 0 && !perms.includes(to.meta.perm)) {
      next({ name: "adminDashboard", replace: true });
      return;
    }
  }
  if (to.name === "adminLogin" && isAdminLoggedIn()) {
    next({ name: "adminBanners", replace: true });
    return;
  }
  if (shouldRedirectVideoToNotFound(to)) {
    next({ name: "notFound", replace: true });
    return;
  }
  if (!minibiliEnv) {
    next();
    return;
  }
  const need = to.matched.some(
    r => r.meta && r.meta.requireMinibiliAuth === true
  );
  if (need && (!getAccessToken() || isAccessTokenExpired())) {
    if (getRefreshToken()) {
      const ok = await refreshMinibiliAccessToken();
      if (ok) {
        next();
        return;
      }
    }
    next({ path: "/", replace: true });
    return;
  }
  next();
});

/** 离开发布/编辑页：关对话框 + 清理 MessageBox 滚动锁（勿关掉投稿成功审核提示） */
router.afterEach((to, from) => {
  if (from.name === "videoPublish" || from.name === "videoEdit") {
    const keepReviewNotice =
      (to.name === "upload" &&
        String(to.query.success || "").toLowerCase() === "publish") ||
      (to.name === "manuscript" && String(to.query.reviewNotice) === "1");
    if (!keepReviewNotice) {
      ElMessageBox.close();
      nextTick(() => {
        document.body.classList.remove("el-popup-parent--hidden");
        document.body.style.removeProperty("width");
      });
    }
  }
  if (!document.querySelector(".mm-del-overlay")) {
    clearStuckPageOverlays();
  }
});

export default router;
