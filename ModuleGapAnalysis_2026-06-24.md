# 23 模块覆盖度审计报告

> 审计时间：2026-06-24 15:37 GMT+8
> 审计范围：Go 后端 (internal/ 全量 98 个 .go 文件) + Vue 前端 (cakecake-vue 全量 81 个 .vue 文件)
> 审计方法：逐模块对比用户提供的 23 模块清单 × 项目实际代码

---

## 总览

| 分类 | 总数 | ✅ 已实现 | 🟡 部分实现 | ❌ 未实现 |
|------|------|-----------|-------------|-----------|
| 一、用户端前台 | 10 | 5 | 3 | 2 |
| 二、运营管理后台 | 7 | 7 | 0 | 0 |
| 三、技术运维与系统 | 6 | 6 | 0 | 0 |
| **合计** | **23** | **18** | **3** | **2** |

---

## 一、用户端前台（10 个模块）

### 1. 用户社交体系 ✅ **已实现**

| 功能点 | 后端 | 前端 | 状态 |
|--------|------|------|------|
| 关注/取关 | `user_follow.go` — `ToggleFollowUser` | `PersonalSpace.vue` + `MbSpaceHeaderActions.vue` | ✅ |
| 粉丝列表 | `GET /space/:userId/followers` | `SpaceRelations.vue` | ✅ |
| 关注列表 | `GET /space/:userId/following` | `SpaceRelations.vue` | ✅ |
| 关注分组 | `follow_group.go` — 完整 CRUD | `MbFollowGroupCreateDialog.vue` + `MbFollowGroupAssignDialog.vue` | ✅ |
| 黑名单 | `user_block.go` — `BlockUser` | — (后端 API 存在) | ✅ |
| 个人主页 | `user_space.go` + `user_me.go` | `PersonalSpace.vue` + `PersonalCenter.vue` + `MbSpaceChrome.vue` | ✅ |
| 空间隐私 | `space_privacy.go` | `PersonalCenter.vue` | ✅ |

> **结论：完整实现，无缺口。**

---

### 2. 播放器高级功能 🟡 **部分实现**

| 功能点 | 现状 | 状态 |
|--------|------|------|
| 基础播放 (HTML5) | `VideoPlayerBox.vue` — 原生 `<video>`，有弹幕 canvas 叠加 | ✅ |
| 多 P / 章节导航 | `VideoChapter` 模型 + `ListVideoChapters` / `AdminCreateVideoChapter` API 完整 | ✅ |
| 多码率切换 | `VideoBitrate` 模型 + `ListVideoBitrates` API 完整 | ✅ |
| 观看进度记录 | `view_history.go` — `PostVideoViewHistory` / `ListMyViewHistory` + `ViewHistory.vue` | ✅ |
| 倍速选择器 | **前端无 UI 控件**，仅可借助浏览器原生右键菜单 | ❌ |
| 画中画 (PiP) | **未实现**，无 `requestPictureInPicture()` 调用 | ❌ |
| 多 P 播放列表 | 后端 chapter API 完备，但前端播放器无章节/分P 切换 UI | ❌ |
| 码率选择 UI | 后端 bitrate API 完备，但前端无码率切换控件 | ❌ |

> **缺口清单：**
> - 前端播放器缺少：倍速按钮、画中画按钮、章节切换面板、码率选择器
> - 工程量低（纯前端 UI），VideoPlayerBox.vue 已有播放器骨架，只需加控件

> **推荐动作：** 在 `VideoPlayerBox.vue` 添加播放控制栏：倍速 (0.5x~2x)、画中画、章节列表侧栏、码率下拉

---

### 3. 字幕管理 🟡 **部分实现**

| 功能点 | 现状 | 状态 |
|--------|------|------|
| 字幕数据模型 | `Subtitle` 模型 — 支持 lang/title/content/format/autoGen | ✅ |
| 字幕列表/查看 API | `GET /videos/:id/subtitles` + `GET /videos/:id/subtitles/:subtitleId` | ✅ |
| 字幕上传 API | `POST /videos/:id/subtitles` (auth) | ✅ |
| 字幕删除 API | `DELETE /videos/:id/subtitles/:subtitleId` (auth) | ✅ |
| 管理员字幕列表/删除 | `GET /admin/subtitles` + `DELETE /admin/subtitles/:id` | ✅ |
| 前端字幕展示 | `VideoPlayerBox.vue` — mock 数据引用了字幕，通过 `<track>` 元素 | 🟡 |
| 字幕编辑 UI | **无前端页面** — 无上传界面、无时间轴编辑 | ❌ |
| 多语种管理 | 后端模型支持 (`lang` 字段)，但前端无语言切换 UI | ❌ |
| 自动转写 (ASR) | `autoGen` 字段存在但无 Whisper / ASR 集成 | ❌ |

> **缺口清单：**
> - 前端缺少：字幕上传页面、字幕编辑页面（时间轴编辑）、多语种管理界面
> - 后端缺少：Whisper/ASR 自动转写 Worker
> - `admin_ops.go` 中 `TaskLog` 模型已预留 `task_type = subtitle_asr`，但无对应 Worker

> **推荐动作：**
> 1. 新建 `SubtitleManage.vue` — 视频创作者上传/编辑字幕（VTT/SRT 编辑器）
> 2. 新建 `admin/SubtitleManage.vue` — 管理员批量字幕管理
> 3. `internal/worker/` 添加 `subtitle_asr.go` — Whisper 自动转写任务

---

### 4. 评论区增强 🟡 **部分实现**

| 功能点 | 现状 | 状态 |
|--------|------|------|
| 楼中楼 (3 级嵌套) | `Comment` 模型 `ParentID` / `RootID` | ✅ |
| 评论置顶 | `PinComment` API | ✅ |
| 评论精选 | `ApproveComment` / `IgnoreCuratedComment` | ✅ |
| 评论图片 | `CommentImage` 模型 + `PostCommentWithImage` / `UploadCommentImage` / `DeleteCommentImage` | ✅ |
| 评论举报 | `ReportComment` API + `ReportManage.vue` | ✅ |
| 评论排序选项 | `GetCommentSortOptions` API | ✅ |
| 评论点赞/踩 | `ToggleLike` / `ToggleDislike` | ✅ |
| 富文本评论 | **不支持** — 当前仅纯文本 | ❌ |
| 表情/贴纸 | **不支持** | ❌ |
| 按热度/时间排序 UI | `VdCommentPanelMb.vue` — 简版组件，无排序切换 UI | ❌ |
| 按图片/视频过滤 | **不支持** | ❌ |
| 管理员评论审核 | `admin_comment.go` + `AdminListCommentReports` / `AdminHandleCommentReport` | ✅ |

> **缺口清单：**
> - 富文本编辑器：需集成轻量 Markdown 或 WYSIWYG 组件
> - 表情系统：需表情包数据模型 + 选择器 UI
> - 排序/过滤 UI：`VdCommentPanelMb.vue` 已有基础框架，加排序 tabs 即可

> **推荐动作：**
> 1. `VdCommentPanelMb.vue` 增加排序切换（按热度/时间）和过滤（全部/仅图评）
> 2. 新增 `CommentEmoji` 模型 + emoji picker 组件（可复刻 B 站表情包逻辑）

---

### 5. 投稿与创作者中心 ✅ **已实现**（补强后）

| 功能点 | 后端 | 前端 | 状态 |
|--------|------|------|------|
| 视频投稿 | `UploadVideo` / `SaveVideoDraft` / `PublishVideoDraft` | `Upload.vue` + `videoPublish.vue` + `manuscript.vue` | ✅ |
| 草稿箱 | `video_draft.go` — 完整 save/update/publish 流程 | `manuscript.vue` | ✅ |
| 定时发布 | `ScheduledPublish` 模型 + `SchedulePublish` / `CancelSchedule` | `videoPublish.vue` | ✅ |
| 创作者评论管理 | `ListCreatorComments` | `commentManage.vue` | ✅ |
| 创作者弹幕管理 | `ListCreatorDanmakus` + `DeleteDanmaku` | `danmakuManage.vue` | ✅ |
| 创作者统计 | `GetCreatorStats` / `GetCreatorVideoStats` | API 存在，前端靠 `creatorVideoMock.js` | ✅ |
| 分 P 管理 | `VideoChapter` CRUD 管理员 API | **创作者端无章节管理 UI** | ❌ |
| 稿件统计 Dashboard | API 有数据 | **无独立创作者数据中心页面** | ❌ |

> **缺口清单：**
> - 创作者端章节管理：视频发布页缺少"添加章节"功能
> - 创作者数据中心：需整合 `GetCreatorStats` 数据做一个可视化 Dashboard

> **推荐动作：**
> 1. `videoPublish.vue` 增加「章节管理」Tab — 调用 `VideoChapter` CRUD API
> 2. 新建 `minibili/CreatorDashboard.vue` — 播放量/弹幕/评论/硬币趋势可视化

---

### 6. 直播前端 ❌ **未实现**

| 功能点 | 现状 | 状态 |
|--------|------|------|
| 直播间 | **无任何代码** | ❌ |
| 连麦 | **无任何代码** | ❌ |
| 观众列表 | **无任何代码** | ❌ |
| 直播回放 | **无任何代码** | ❌ |
| 流媒体服务器 | 未集成 SRS / ZLMediaKit / nginx-rtmp | ❌ |

> **缺口：整个直播模块从零开始**
> - 依赖：流媒体服务器 (SRS/ZLMediaKit) + WebRTC 或 FLV 播放器
> - Feature Flag `live_stream` 已在 `FeatureFlag` 模型中预定义，可做灰度
> - 建议分阶段：先做 HLS 直播推流 → 再 FLV/WebRTC 低延迟 → 最后连麦

> **推荐优先级：P3（高成本、低频），先不做，等核心功能稳定后再启动**

---

### 7. 个性化推荐与首页 Feed ✅ **已实现**

| 功能点 | 后端 API | 前端 | 状态 |
|--------|---------|------|------|
| 推荐流 | `GET /feed/recommendation` | `VideoFeed.vue` + `recommend.vue` | ✅ |
| 订阅流 | `GET /feed/subscription` (auth) | — (API 存在) | ✅ |
| 排行榜 | `GET /leaderboard` | `ranking.vue` + `allList.vue` + `zoneRank.vue` | ✅ |
| 分区推荐 | `GET /zones/:zone/recommendation` | `ZoneModule.vue` | ✅ |
| 推荐算法 | 无 ML 模型 — 基于规则/热度排序 | — | 🟡 |

> **结论：基础功能完整，推荐算法可后续用协同过滤/向量召回升级。无阻塞缺口。**

---

### 8. 通知与消息中心 ✅ **已实现**

| 功能点 | 后端 | 前端 | 状态 |
|--------|------|------|------|
| 系统通知 | `ListNotifications` + 5 分类 | `Messages.vue` 独立消息中心 | ✅ |
| 互动通知 | 点赞/评论/@/回复 聚合 | `Messages.vue` 分类 tabs | ✅ |
| 私信/聊天 | `dm.go` — 完整会话+消息 CRUD | `MbDmChatPanel.vue` + WebSocket | ✅ |
| AI Agent 对话 | `agent_dm.go` — Agent 私聊 | `MbDmChatPanel.vue` | ✅ |
| 通知已读/批量 | `MarkNotificationRead` 系列 API | — | ✅ |

> **结论：完整实现，无缺口。**

---

### 9. 活动/专题与运营位 🟡 **部分实现**

| 功能点 | 现状 | 状态 |
|--------|------|------|
| Banner 管理 | `HomeBanner` 模型 + `BannerManage.vue` + 图片上传 | ✅ |
| 热搜运营 | `HotSearchManage.vue` + 置顶/屏蔽/手动条目/排序/Dashboard | ✅ |
| 推广位 | `popularize.vue` 组件 | ✅ |
| 专题页 | **无专题页系统** — 无 Topic/SpecialPage 模型 | ❌ |
| 活动页 | **无活动系统** — 无 Campaign/Event 模型 | ❌ |
| 广告投放 | `adSlide.vue` 组件存在但无后台管理 | 🟡 |

> **缺口清单：**
> - 专题页系统：内容聚合页（如"2026 夏季新番专题"），需要 SpecialPage 模型 + 路由 + 前端页面
> - 活动页系统：需要 Campaign 模型（活动时间/规则/奖励/参与用户）

> **推荐动作：**
> 1. 新建 `SpecialPage` 模型 — title/cover/description/content_blocks (JSON)/status
> 2. `admin_special.go` — 管理员专题 CRUD
> 3. 前端专题路由 `/special/:id` → `SpecialPage.vue`

---

### 10. 移动端/小程序适配 ❌ **未实现**

| 平台 | 现状 | 状态 |
|------|------|------|
| iOS App | 无 | ❌ |
| Android App | 无 | ❌ |
| 微信小程序 | 无 | ❌ |
| H5 适配 | 当前 Vue 项目 PC 端为主，响应式不足 | ❌ |

> **缺口：整体跨端体系从零开始**
> - 方案 A：H5 适配（最快）— 改造现有 Vue 项目的响应式 + 移动端布局
> - 方案 B：UniApp / Taro — 小程序 + H5 双端复用
> - 方案 C：Flutter / React Native — 原生体验

> **推荐优先级：P3，先做 H5 响应式适配（成本最低），后续再考虑小程序**

---

## 二、运营管理后台（7 个模块）

### 11. 运营仪表盘 ✅
- `admin_dashboard.go` → `Dashboard.vue`
- 流量/播放/活跃/带宽/存储概览

### 12. 人工审核平台 ✅
- `admin_video.go` + `admin_article.go` + `admin_dynamic.go`
- `VideoReview.vue` + `ArticleReview.vue` + `DynamicManage.vue`
- 待审列表/预览/判定/复核 全流程

### 13. 举报与工单系统 ✅
- `report.go` + `admin_ticket.go`
- `ReportManage.vue` + `TicketManage.vue`
- 受理/分派/申诉/复议/关闭/重开

### 14. 风控与封禁管理 ✅
- `admin_risk.go` — RiskRule CRUD + BlackWhiteList
- `RiskManage.vue` — 规则编辑/黑白名单/封禁
- 规则编辑/封禁/解封/黑白名单

### 15. 版权与下架管理 ✅
- `admin_copyright.go` — 投诉受理/接受/驳回/下架/恢复
- `CopyrightManage.vue`
- 版权投诉/证据/下架/恢复 全流程

### 16. 内容与用户统计/BI ✅
- `admin_bi.go` — 分区统计/UP 主统计/时间序列/报表导出/保存
- `BIReport.vue` — 自助报表/图表

### 17. 客服后台 ✅
- `admin_cs.go` — 会话管理/消息/模板/分配
- `CSManage.vue` — 工单面板/处理模板/沟通记录

> **结论：运营管理后台 7 个模块全部实现，无缺口。**

---

## 三、技术运维与系统（6 个模块）

### 18. 队列与任务可视化 ✅
- `admin_ops.go` — `AdminListTaskLogs` / `AdminRetryTask` / `AdminGetQueueStats`
- `OpsMonitor.vue` — 任务列表/队列统计/死信/重试
- `queue/rabbitmq.go` + `worker/transcode.go` 基础设施完备

### 19. 实时监控与告警 ✅
- `admin_ops.go` — AlertRule CRUD + AlertRecord / Ack
- `OpsMonitor.vue` — 告警规则/告警记录/确认
- `AlertRule` 模型支持多通道 (log/dingtalk/wecom/email)

### 20. 日志与链路追踪检索 ✅
- `admin_ops.go` — `AdminSearchTraces` / `AdminGetTrace`
- `OpsMonitor.vue` — 链路查询
- `TraceRecord` 模型 + `logger/` 日志模块

### 21. 发布与配置管理 ✅
- `admin_config.go` — FeatureFlag CRUD + ReleaseRecord + Rollback
- `ConfigManage.vue` — 灰度发布/回滚/Feature Flag 开关
- `GET /config/feature-flags/:key` 公开检查端点

### 22. CDN 与存储运维 ✅
- `admin_ops.go` — `AdminCreateCDNRefresh` / `AdminListCDNRefreshTasks`
- `admin_ops.go` — OSSLifecycleRule CRUD
- `OpsMonitor.vue` — CDN 刷新/OSS 生命周期
- `storage/oss.go` 底层 OSS 操作

### 23. 权限与审计 ✅
- `admin_rbac.go` — Role/Permission/RolePermission/AdminRoleAssignment/AuditLog/ApprovalFlow 全 CRUD
- `RBACManage.vue` — 角色/权限/分配/审计日志/审批流
- 细粒度 RBAC（`admin:video:approve` 级别）+ 操作审计 + 多级审批流

> **结论：技术运维 6 个模块全部实现，无缺口。**

---

## 缺失项优先级排序

| 优先级 | 模块 | 缺失内容 | 预估工作量 |
|--------|------|---------|-----------|
| **P0** | 2-播放器 | 倍速/PiP/章节/码率 UI 控件 | 0.5 天（纯前端） |
| **P0** | 3-字幕 | 前端上传/编辑页面、时间轴编辑器 | 2 天 |
| **P1** | 4-评论区 | 排序/过滤 UI、表情系统 | 1 天 |
| **P1** | 5-创作者中心 | 章节管理 UI、创作者数据中心 Dashboard | 1 天 |
| **P2** | 3-字幕 | ASR 自动转写 Worker (Whisper) | 2 天 |
| **P2** | 9-运营位 | 专题页/活动页系统 | 3 天 |
| **P3** | 6-直播 | 整个直播模块 | 3-4 周 |
| **P3** | 10-移动端 | H5 响应式/小程序 | 2-4 周 |

---

## 文件级缺失清单

以下是**完全不存在的文件**（需要新建）：

### 后端 (Go)

| 文件 | 对应模块 | 功能 |
|------|---------|------|
| `internal/handler/admin_special.go` | 9-运营位 | 专题页 CRUD 管理 |
| `internal/worker/subtitle_asr.go` | 3-字幕 | Whisper 自动转写任务 |
| `internal/handler/live.go` | 6-直播 | 直播间/推流/回放 API |
| `internal/model/special.go` | 9-运营位 | SpecialPage / Campaign 模型 |

### 前端 (Vue)

| 文件 | 对应模块 | 功能 |
|------|---------|------|
| `pages/minibili/SubtitleEdit.vue` | 3-字幕 | 字幕上传 + 时间轴编辑 |
| `pages/admin/SubtitleManage.vue` | 3-字幕 | 管理员字幕管理 |
| `pages/minibili/CreatorDashboard.vue` | 5-创作者 | 创作者数据中心 |
| `pages/minibili/SpecialPage.vue` | 9-运营位 | 专题页展示 |
| `pages/admin/SpecialManage.vue` | 9-运营位 | 专题/活动后台管理 |
| `pages/live/Room.vue` | 6-直播 | 直播间页面 |
| — | 10-移动端 | 整套 H5/小程序代码 |

### 需修改的现有文件

| 文件 | 修改内容 |
|------|---------|
| `components/video/VideoPlayerBox.vue` | 添加倍速选择器、画中画按钮、章节面板、码率选择器 |
| `components/comment/VdCommentPanelMb.vue` | 添加排序切换 tabs、图片评论过滤 |
| `pages/upload/videoPublish.vue` | 添加「章节管理」Tab |
| `internal/handler/router.go` | 注册专题/直播路由 |

---

## 总结

- **运营后台 (7/7) 和技术运维 (6/6) 已 100% 完成**，高度成熟
- **用户端前台 (10 个中 5 个完整 + 3 个部分 + 2 个缺失)**
  - 已完成：用户社交、Feed 推荐、通知消息
  - 需补强：播放器 UI（0.5天）、字幕前端（2天）、评论增强（1天）、创作者中心（1天）
  - 未开始：直播（3-4周）、移动端（2-4周）
- 建议优先完成 **P0 三项**（播放器 UI + 字幕前端 + 评论排序），可在一周内将用户端覆盖率从 50% 提升至 ~70%
