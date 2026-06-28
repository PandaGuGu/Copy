# 运营中心系统 PRD

**版本**: v1.0（架构回溯）
**日期**: 2026-06-25
**作者**: 架构师 Winston（BMAD 框架）
**轨道**: BMad Method
**状态**: 草稿 — 从现有代码逆向提取
**来源代码**: `internal/handler/admin_*.go` (24 文件) + `internal/model/module_extend.go`

> 此 PRD 从已实现的 23 模块代码中逆向提取，描述系统"实际做了什么"而非"计划做什么"。

---

## 目录

1. [系统概述](#1-系统概述)
2. [功能需求 —— 运营管理后台](#2-功能需求--运营管理后台)
3. [功能需求 —— 技术运维与系统](#3-功能需求--技术运维与系统)
4. [功能需求 —— 用户端扩展](#4-功能需求--用户端扩展)
5. [非功能需求](#5-非功能需求)
6. [验收标准](#6-验收标准)
7. [兼容要求](#7-兼容要求)
8. [排除范围](#8-排除范围)

---

## 1. 系统概述

### 1.1 目的

运营中心是 Mini-Bili 平台的管理中枢，为运营人员和技术运维人员提供一站式的后台管理能力。核心目标：
1. **运营管理** — 仪表盘、审核、举报、风控、版权、BI、客服
2. **技术运维** — 任务队列、告警、链路追踪、配置发布、CDN/存储、RBAC 审计
3. **用户端增强** — 播放器高级功能、字幕、评论增强、创作者中心、Feed 推荐、专题活动

### 1.2 范围

**在范围内（23 模块）：**

| # | 分类 | 模块 | 文件 |
|---|------|------|------|
| 1 | 运营后台 | 运营仪表盘 | `admin_dashboard.go` → `Dashboard.vue` |
| 2 | 运营后台 | 人工审核（视频/专栏/动态） | `admin_video.go` / `admin_article.go` / `admin_dynamic.go` |
| 3 | 运营后台 | 举报与工单系统 | `report.go` / `admin_ticket.go` |
| 4 | 运营后台 | 风控与封禁管理 | `admin_risk.go` |
| 5 | 运营后台 | 版权与下架管理 | `admin_copyright.go` |
| 6 | 运营后台 | BI 内容与用户统计 | `admin_bi.go` |
| 7 | 运营后台 | 客服后台 | `admin_cs.go` |
| 8 | 技术运维 | 队列与任务可视化 | `admin_ops.go` (task/queue) |
| 9 | 技术运维 | 实时监控与告警 | `admin_ops.go` (alert) |
| 10 | 技术运维 | 日志与链路追踪 | `admin_ops.go` (trace) |
| 11 | 技术运维 | 发布与配置管理 | `admin_config.go` |
| 12 | 技术运维 | CDN 与存储运维 | `admin_ops.go` (cdn/oss) |
| 13 | 技术运维 | 权限与审计 (RBAC) | `admin_rbac.go` |
| 14 | 运营扩展 | 用户管理 | `admin_user.go` |
| 15 | 运营扩展 | 评论管理 | `admin_comment.go` |
| 16 | 运营扩展 | Banner 管理 | `admin_banner.go` + upload |
| 17 | 运营扩展 | 热搜运营 | `admin_hot_search.go` + dashboard |
| 18 | 运营扩展 | AI 角色管理 | `admin_agent.go` |
| 19 | 运营扩展 | 系统设置 | `admin_settings.go` |
| 20 | 运营扩展 | LLM 配置 | `admin_llm_config.go` |
| 21 | 用户端 | 播放器高级 | `admin_player.go` / `feed.go` |
| 22 | 用户端 | 字幕管理 | `subtitle.go` |
| 23 | 用户端 | 评论增强 | `comment_enhance.go` + `creator_center.go` |
| 24 | 用户端 | Feed 推荐 | `feed.go` |
| 25 | 用户端 | 专题活动 | `admin_special.go` |

**不在范围内:**
- 移动端/小程序适配
- 支付、会员、充电商业化

### 1.3 架构驱动因素

最制约设计的 NFR：

1. **NFR-AUTH：认证与授权** — 管理员端独立 JWT 体系，RBAC 细粒度权限 (`resource:action`)，全写操作审计
2. **NFR-DATA：数据一致性** — MySQL + GORM AutoMigrate，Redis 缓存热数据，RabbitMQ 异步任务
3. **NFR-PERF：并发与延迟** — 单体架构支撑运营后台日常负载（~50 并发管理员）
4. **NFR-EXT：可扩展性** — 代码架构必须支持未来平滑拆分为微服务（BC-2 约束）

### 1.4 利益相关者

- **用户**: 运营管理员、技术运维、客服、内容审核员
- **团队**: 1 人全栈开发（PandaGuGu）
- **现有约束**: Go Gin 单体 + Vue3 SPA + MySQL/Redis/RabbitMQ + 阿里云 OSS

---

## 2. 功能需求 —— 运营管理后台（13 模块）

### FR-001：运营仪表盘

| 需求项 | 规格 |
|--------|------|
| 端点 | `GET /api/v1/admin/dashboard` |
| 权限 | 任一已认证管理员 |
| 数据范围 | 平台总览：用户总数、视频总数、今日播放量、活跃用户数、带宽占用、存储用量 |
| 前端 | `Dashboard.vue` — 面板布局，含流量/播放/活跃/带宽/存储概览卡片 |

### FR-002：人工审核平台

| 需求项 | 规格 |
|--------|------|
| 视频审核 | `GET/POST /api/v1/admin/videos` — 待审列表、预览、通过/驳回/删除 |
| 专栏审核 | `GET/POST /api/v1/admin/articles` — 待审列表、预览、通过/驳回/删除 |
| 动态审核 | `GET/POST /api/v1/admin/dynamics` — 待审列表、预览、删除 |
| 权限 | 阅读：所有管理员；写操作：`video.approve` / `article.approve` / `dynamic.manage` |
| 前端 | `VideoReview.vue` / `ArticleReview.vue` / `DynamicManage.vue` |

### FR-003：举报与工单系统

**FR-003-a：举报处理**

| 需求项 | 规格 |
|--------|------|
| 用户发起 | `POST /api/v1/reports` (auth) — 用户举报内容 |
| 管理员列表 | `GET /api/v1/admin/reports` — 举报记录列表 |
| 批量处理 | `POST /api/v1/admin/reports/batch` — 批量受理 |
| 单条处理 | `POST /api/v1/admin/reports/:id/handle` — 受理举报 |
| 删除 | `DELETE /api/v1/admin/reports/:id` |
| 权限 | `ticket.handle` |
| 前端 | `ReportManage.vue` |

**FR-003-b：工单系统**

| 需求项 | 规格 |
|--------|------|
| 用户发起 | `POST /api/v1/tickets` (auth) — 用户提交工单 |
| 用户查看 | `GET /api/v1/users/me/tickets` — 我的工单列表 |
| 用户追加 | `POST /api/v1/users/me/tickets/:id/messages` — 追加消息 |
| 用户申诉 | `POST /api/v1/users/me/tickets/:id/appeal` — 申诉 |
| 管理员列表 | `GET /api/v1/admin/tickets` — 全部工单 |
| 分配 | `POST /api/v1/admin/tickets/:id/assign` — 分配处理人 |
| 状态更新 | `POST /api/v1/admin/tickets/:id/status` |
| 关闭/重开 | `POST /api/v1/admin/tickets/:id/close` / `reopen` |
| 管理员回复 | `POST /api/v1/admin/tickets/:id/messages` |
| 权限 | `ticket.handle` |
| 前端 | `TicketManage.vue` |
| 数据模型 | `Ticket`, `TicketMessage`, `TicketSatisfaction` |

### FR-004：风控与封禁管理

| 需求项 | 规格 |
|--------|------|
| 风险规则 CRUD | `GET/POST/PUT/DELETE /api/v1/admin/risk/rules` |
| 规则启停 | `POST /api/v1/admin/risk/rules/:id/toggle` |
| 黑白名单 | `GET/POST/DELETE /api/v1/admin/risk/bw-list` |
| 命中日志 | `GET /api/v1/admin/risk/hits` |
| 用户封禁 | `POST /api/v1/admin/users/:id/ban` / `unban` — 权限 `user.ban` |
| 用户删除 | `POST /api/v1/admin/users/:id/delete` |
| 违规记录 | `GET /api/v1/admin/users/:id/violations` |
| 权限 | `risk.manage` (规则/名单), `user.ban` (封禁) |
| 前端 | `RiskManage.vue` |
| 数据模型 | `RiskRule`, `BlackWhiteList`, `RiskHitLog` |

### FR-005：版权与下架管理

| 需求项 | 规格 |
|--------|------|
| 用户投诉 | `POST /api/v1/copyright/complaints` (auth) |
| 投诉列表 | `GET /api/v1/admin/copyright/complaints` |
| 投诉详情 | `GET /api/v1/admin/copyright/complaints/:id` |
| 受理 | `POST /api/v1/admin/copyright/complaints/:id/accept` |
| 驳回 | `POST /api/v1/admin/copyright/complaints/:id/reject` |
| 下架 | `POST /api/v1/admin/copyright/complaints/:id/takedown` |
| 恢复 | `POST /api/v1/admin/copyright/complaints/:id/restore` |
| 权限 | `copyright.handle` |
| 前端 | `CopyrightManage.vue` |
| 数据模型 | `CopyrightComplaint`, `CounterNotice` |

### FR-006：BI 内容与用户统计

| 需求项 | 规格 |
|--------|------|
| 分区统计 | `GET /api/v1/admin/bi/zone-stats` |
| UP 主统计 | `GET /api/v1/admin/bi/creator-stats` |
| 时间序列 | `GET /api/v1/admin/bi/time-series` |
| 报表导出 | `POST /api/v1/admin/bi/export` |
| 报表保存 | `POST /api/v1/admin/bi/reports` |
| 报表列表 | `GET /api/v1/admin/bi/reports` |
| 报表删除 | `DELETE /api/v1/admin/bi/reports/:id` |
| 权限 | `dashboard.export` |
| 前端 | `BIReport.vue` |
| 数据模型 | `SavedReport`, `VideoDailyStat` |

### FR-007：客服后台

| 需求项 | 规格 |
|--------|------|
| 用户发起 | `POST /api/v1/cs/conversations` (auth) |
| 会话列表（管理员） | `GET /api/v1/admin/cs/conversations` |
| 分配客服 | `POST /api/v1/admin/cs/conversations/:id/assign` |
| 发送消息 | `POST /api/v1/admin/cs/conversations/:id/messages` |
| 关闭会话 | `POST /api/v1/admin/cs/conversations/:id/close` |
| 回复模板 CRUD | `GET/POST/PUT/DELETE /api/v1/admin/cs/templates` |
| 权限 | `cs.manage` |
| 前端 | `CSManage.vue` |
| 数据模型 | `CSConversation`, `CSMessage`, `CSTemplate` |

### FR-008：用户管理

| 需求项 | 规格 |
|--------|------|
| 用户列表 | `GET /api/v1/admin/users` — 分页、搜索 |
| 用户详情 | `GET /api/v1/admin/users/:id` |
| 前端 | `UserManage.vue` |

### FR-009：评论管理

| 需求项 | 规格 |
|--------|------|
| 评论列表 | `GET /api/v1/admin/comments` — 跨 user/comments/reply 三表联合查询 |
| 评论详情 | `GET /api/v1/admin/comments/:id` |
| 删除评论 | `POST/DELETE /api/v1/admin/comments/:id/delete` |
| 评论举报管理 | `GET /api/v1/admin/comment-reports` + `POST handle` |
| 权限 | 阅读：所有管理员；写：`comment.delete` |
| 前端 | `CommentManage.vue` |

### FR-010：Banner 管理

| 需求项 | 规格 |
|--------|------|
| Banner 列表 | `GET /api/v1/admin/home-banners` |
| 创建 Banner | `POST /api/v1/admin/home-banners` |
| 更新 Banner | `PUT /api/v1/admin/home-banners/:id` |
| 删除 Banner | `DELETE /api/v1/admin/home-banners/:id` |
| 图片上传 | `POST /api/v1/admin/home-banners/upload-image` |
| 权限 | `banner.manage` |
| 前端 | `BannerManage.vue` |
| 数据模型 | `HomeBanner` |

### FR-011：热搜运营

| 需求项 | 规格 |
|--------|------|
| 热搜运营列表 | `GET /api/v1/admin/hot-search/ops` |
| 运营 Dashboard | `GET /api/v1/admin/hot-search/dashboard` |
| 预览 | `GET /api/v1/admin/hot-search/preview` |
| 手动条目 CRUD | `POST/PUT/DELETE /api/v1/admin/hot-search/ops` |
| 快速操作 | `POST /api/v1/admin/hot-search/quick-op` |
| 排序 | `POST /api/v1/admin/hot-search/reorder` |
| 显示顺序重置 | `POST /api/v1/admin/hot-search/display-order/reset` |
| Redis 操作 | `POST remove/boost` — 管理 Redis 中的热搜数据 |
| 权限 | `hotsearch.manage` |
| 前端 | `HotSearchManage.vue` |

### FR-012：AI 角色管理

| 需求项 | 规格 |
|--------|------|
| Agent 全局设置 | `GET/PUT /api/v1/admin/agent-settings` |
| Agent 头像 | `POST /api/v1/admin/agent-settings/avatar` |
| Agent 角色 CRUD | `GET/POST/PUT/DELETE /api/v1/admin/agent-profiles` |
| 角色头像 | `POST /api/v1/admin/agent-profiles/:id/avatar` |
| 权限 | `agent.manage` |
| 前端 | `AgentManage.vue` |

### FR-013：系统设置与 LLM 配置

| 需求项 | 规格 |
|--------|------|
| LLM 配置获取/更新 | `GET/PUT /api/v1/admin/llm-config` |
| 系统设置获取/更新 | `GET/PUT /api/v1/admin/settings` |
| 保存策略 | 同步更新内存 Cfg + `.env` 文件（`updateEnvKeys()` 逐行更新，追加缺失 key） |
| 权限 | `setting.manage` |
| 前端 | `Settings.vue`（LLM 配置嵌入 `AgentManage.vue`） |

---

## 3. 功能需求 —— 技术运维与系统（6 模块）

### FR-014：队列与任务可视化（模块 18）

| 需求项 | 规格 |
|--------|------|
| 任务日志列表 | `GET /api/v1/admin/ops/tasks` |
| 任务重试 | `POST /api/v1/admin/ops/tasks/:id/retry` |
| 队列统计 | `GET /api/v1/admin/ops/queue-stats` |
| 权限 | `ops.manage` |
| 前端 | `OpsMonitor.vue` — 任务列表/队列统计/死信/重试 |
| 数据模型 | `TaskLog` — 支持 task_type 枚举（transcode, subtitle_asr 等） |

### FR-015：实时监控与告警（模块 19）

| 需求项 | 规格 |
|--------|------|
| 告警规则 CRUD | `GET/POST/PUT/DELETE /api/v1/admin/ops/alert-rules` |
| 规则启停 | `POST /api/v1/admin/ops/alert-rules/:id/toggle` |
| 告警记录 | `GET /api/v1/admin/ops/alert-records` |
| 告警确认 | `POST /api/v1/admin/ops/alert-records/:id/ack` |
| 系统健康检查 | `GET /api/v1/admin/ops/health` |
| 通知通道 | 多通道支持（log/dingtalk/wecom/email） |
| 权限 | `ops.manage` |
| 前端 | `OpsMonitor.vue` |
| 数据模型 | `AlertRule`, `AlertRecord` |

### FR-016：日志与链路追踪（模块 20）

| 需求项 | 规格 |
|--------|------|
| 链路查询 | `GET /api/v1/admin/ops/traces` |
| 链路详情 | `GET /api/v1/admin/ops/traces/:id` |
| 权限 | `ops.manage` |
| 前端 | `OpsMonitor.vue` — 链路查询面板 |
| 数据模型 | `TraceRecord` |

### FR-017：发布与配置管理（模块 21）

| 需求项 | 规格 |
|--------|------|
| Feature Flag CRUD | `GET/POST/PUT /api/v1/admin/config/feature-flags` |
| Flag 启停 | `POST /api/v1/admin/config/feature-flags/:id/toggle` |
| Flag 公开检查 | `GET /api/v1/config/feature-flags/:key` (无需认证) |
| 灰度策略 | FNV-1a hash 分桶 + whitelist + rollout_pct |
| 发布记录 | `GET/POST /api/v1/admin/config/releases` |
| 回滚 | `POST /api/v1/admin/config/releases/:id/rollback` |
| 权限 | `config.manage` |
| 前端 | `ConfigManage.vue` |
| 数据模型 | `FeatureFlag`, `ReleaseRecord` |

### FR-018：CDN 与存储运维（模块 22）

| 需求项 | 规格 |
|--------|------|
| CDN 刷新 | `POST /api/v1/admin/ops/cdn/refresh` — 创建刷新任务 |
| CDN 任务列表 | `GET /api/v1/admin/ops/cdn/refresh` |
| OSS 生命周期规则 CRUD | `GET/POST/DELETE /api/v1/admin/ops/oss/lifecycle` |
| 权限 | `ops.manage` |
| 前端 | `OpsMonitor.vue` |
| 数据模型 | `CDNRefreshTask`, `OSSLifecycleRule` |

### FR-019：权限与审计 RBAC（模块 23）

| 需求项 | 规格 |
|--------|------|
| 角色 CRUD | `GET/POST/PUT/DELETE /api/v1/admin/rbac/roles` |
| 权限列表 | `GET /api/v1/admin/rbac/permissions` |
| 角色-权限分配 | `POST /api/v1/admin/rbac/roles/:id/permissions` |
| 管理员列表 | `GET /api/v1/admin/rbac/admins` |
| 创建管理员 | `POST /api/v1/admin/rbac/admins` |
| 管理员-角色分配 | `POST /api/v1/admin/rbac/admins/:adminId/role` |
| 审计日志 | `GET /api/v1/admin/rbac/audit-logs` + 详情 |
| 登录日志 | `GET /api/v1/admin/rbac/login-logs` |
| 审批流 CRUD | `POST/GET /api/v1/admin/rbac/approval-flows` |
| 审批操作 | `POST approve/reject /api/v1/admin/rbac/approval-flows/:id` |
| 自权限查询 | `GET /api/v1/admin/rbac/me/permissions` (所有管理员) |
| 权限 | `rbac.manage` |
| 前端 | `RBACManage.vue` |
| 数据模型 | `AdminRole`, `AdminPermission`, `RolePermission`, `AdminRoleAssignment`, `AuditLog`, `AdminLoginLog`, `ApprovalFlow`, `ApprovalStep` |

---

## 4. 功能需求 —— 用户端扩展（6 模块）

### FR-020：播放器高级功能（模块 2）

| 需求项 | 规格 |
|--------|------|
| 视频章节 CRUD | `POST/DELETE /api/v1/videos/:id/chapters` (创作者端) + `GET/POST/DELETE /api/v1/admin/videos/:id/chapters` (管理端) |
| 码率变体管理 | `POST/DELETE /api/v1/videos/:id/bitrates` (创作者端) + `GET/POST/DELETE /api/v1/admin/videos/:id/bitrates` (管理端) |
| 公开访问 | `GET /api/v1/videos/:id/chapters` / `bitrates` |
| 观看进度 | `POST /api/v1/videos/:id/view-history` + `GET/DELETE /api/v1/users/me/view-history` |
| 前端 | `VideoPlayerBox.vue` — 基础播放器（待加强：倍速/PiP/章节面板） |
| 数据模型 | `VideoChapter`, `VideoBitrate`, `ViewHistory` |

### FR-021：字幕管理（模块 3）

| 需求项 | 规格 |
|--------|------|
| 公开列表 | `GET /api/v1/videos/:id/subtitles` |
| 字幕详情 | `GET /api/v1/videos/:id/subtitles/:subtitleId` |
| 上传（创作者） | `POST /api/v1/videos/:id/subtitles` (auth) |
| 删除（创作者） | `DELETE /api/v1/videos/:id/subtitles/:subtitleId` (auth) |
| 管理员列表 | `GET /api/v1/admin/subtitles` |
| 管理员删除 | `DELETE /api/v1/admin/subtitles/:id` |
| 权限 | `subtitle.manage` |
| 前端 | `SubtitleManage.vue` — 管理员字幕管理 |
| 待实现 | 字幕时间轴编辑器前端、ASR 自动转写 Worker |
| 数据模型 | `Subtitle` — lang/title/content/format/autoGen |

### FR-022：评论增强（模块 4）

| 需求项 | 规格 |
|--------|------|
| 评论图片 | `POST /api/v1/videos/:id/comments-with-image` + `POST/DELETE/GET /api/v1/comments/:id/images` |
| 评论排序 | `GET /api/v1/videos/:id/comments/config` — 获取可用排序选项 |
| 评论举报 | `POST /api/v1/comments/:id/report` (auth) |
| 置顶/精选 | 已有 `PinComment` / `ApproveComment` / `IgnoreCuratedComment` API |
| 点赞/踩 | 已有 `ToggleLike` / `ToggleDislike` |
| 数据模型 | `CommentImage` |
| 前端 | `VdCommentPanelMb.vue` — 待加强排序/过滤 UI |

### FR-023：创作者中心（模块 5）

| 需求项 | 规格 |
|--------|------|
| 草稿箱 | `POST/PUT /api/v1/videos/draft` — 保存/更新草稿 |
| 发布草稿 | `POST /api/v1/videos/:id/publish` |
| 定时发布 | `POST /api/v1/videos/:id/schedule` + `DELETE cancel` |
| 创作者统计 | `GET /api/v1/users/me/creator/stats` + `/video-stats` |
| 评论管理 | `GET /api/v1/users/me/creator/comments` |
| 弹幕管理 | `GET /api/v1/users/me/creator/danmakus` + `DELETE` |
| 章节管理 | `POST/DELETE /api/v1/videos/:id/chapters` (创作者端) |
| 视频媒体替换 | `POST /api/v1/videos/:id/replace-media` |
| 数据模型 | `ScheduledPublish` |
| 前端 | `manuscript.vue` / `videoPublish.vue` / `Upload.vue` |

### FR-024：Feed 推荐（模块 7）

| 需求项 | 规格 |
|--------|------|
| 推荐流 | `GET /api/v1/feed/recommendation` — 基于规则/热度排序 → MMR/DPP 多样性重排序 |
| 订阅流 | `GET /api/v1/feed/subscription` (auth) — 关注 UP 主内容 |
| 排行榜 | `GET /api/v1/leaderboard` |
| 分区推荐 | `GET /api/v1/zones/:zone/recommendation` |
| 前端 | `VideoFeed.vue` / `recommend.vue` / `ranking.vue` |
| 重排序 | MMR（最大边际相关性）或 DPP（行列式点过程），平衡相关性与多样性，避免同类内容聚集 |

### FR-025：专题与活动页（模块 9）

| 需求项 | 规格 |
|--------|------|
| 公开专题列表 | `GET /api/v1/specials` |
| 专题详情 | `GET /api/v1/specials/:slug` |
| 管理员专题 CRUD | `GET/POST/PUT/DELETE /api/v1/admin/specials` |
| 活动 CRUD | `GET/POST/PUT/DELETE /api/v1/admin/campaigns` |
| 权限 | `special.manage` |
| 前端 | `SpecialManage.vue` |
| 数据模型 | `SpecialPage`, `Campaign` |

---

## 5. 非功能需求

### NFR-AUTH：认证与授权

| 编号 | 需求 | 详细规格 |
|------|------|---------|
| NFR-AUTH-1 | 独立管理员认证 | 管理员使用独立 JWT 体系（Admin Token），端点为 `POST /api/v1/admin/auth/login`；Token 与用户端 JWT 完全隔离 |
| NFR-AUTH-2 | Token 刷新 | 支持 refresh token 轮换：`POST /api/v1/admin/auth/refresh` |
| NFR-AUTH-3 | RBAC 授权 | 细粒度权限模型 `resource:action`（如 `video:approve`、`user:ban`）；中间件层拦截 |
| NFR-AUTH-4 | 全操作审计 | 所有写操作（POST/PUT/DELETE）自动记录 AuditLog，包含 admin_id、action、resource、target_id、detail、ip、user_agent、timestamp |
| NFR-AUTH-5 | 审批流 | 多级审批流：`create → approval → approved/rejected`，支持 `resource_type + resource_id` 绑定 |

### NFR-DATA：数据与存储

| 编号 | 需求 | 详细规格 |
|------|------|---------|
| NFR-DATA-1 | 主存储 | MySQL + GORM AutoMigrate — 表结构由代码驱动，无需手动 SQL |
| NFR-DATA-2 | 缓存 | Redis — 播放量热数据、弹幕实时通道、Token 黑名单 |
| NFR-DATA-3 | 文件存储 | 阿里云 OSS `mini-bili` Bucket — 视频/封面/头像/字幕文件；生产路径 `videos/{id}.mp4`、`covers/{id}.{ext}`、`avatars/{user_id}.{ext}` |
| NFR-DATA-4 | 异步任务 | RabbitMQ — 视频转码、字幕 ASR 等耗时操作（`TaskLog.task_type` 枚举） |

### NFR-PERF：性能与可用性

| 编号 | 需求 | 详细规格 |
|------|------|---------|
| NFR-PERF-1 | 架构模式 | 模块化单体 (Gin) — 满足运营中心 ~50 并发管理员需求，为未来微服务拆分预留接口边界 |
| NFR-PERF-2 | 前端架构 | Vue 3 + Vite SPA，`AdminLayout.vue` 统一管理后台布局 + 侧边栏 19 项导航 |
| NFR-PERF-3 | API 风格 | RESTful，统一 JSON 信封格式 `{code, msg, data}`，统一错误码（`errcode` 包） |
| NFR-PERF-4 | 跨域 | CORS 中间件，允许所有来源（开发/低安全场景），生产通过 `Authorization` header |

### NFR-CONFIG：配置管理

| 编号 | 需求 | 详细规格 |
|------|------|---------|
| NFR-CONFIG-1 | 环境配置 | `.env` 文件管理，`updateEnvKeys()` 逐行更新 + 追加缺失 key |
| NFR-CONFIG-2 | LLM 配置 | 运行时更新内存 Cfg + `.env` 文件同步 |
| NFR-CONFIG-3 | Feature Flag | FNV-1a hash 分桶 + whitelist + rollout_pct 灰度策略 |

### NFR-EXT：可扩展性

| 编号 | 需求 | 详细规格 |
|------|------|---------|
| NFR-EXT-1 | 微服务就绪 | 当前模块化单体按文件拆分 handler，接口边界清晰；未来按模块拆分为独立服务（BC-2 约束） |
| NFR-EXT-2 | 前端模块化 | 每个管理页面为独立 Vue SFC，通过 `router/index.js` 路由注册 |
| NFR-EXT-3 | 数据模型预留 | 视频 `status` 字段预留 `pending_review`/`rejected`，`TaskLog.task_type` 可扩展枚举 |

---

## 6. 验收标准

| 编号 | 验收项 | 达标标准 |
|------|--------|---------|
| AC-ADM-01 | 管理员认证闭环 | 访问 `/admin/login` → 输入管理员凭据 → 获取 JWT → 携带 Token 访问 `/api/v1/admin/me` → 返回管理员信息 |
| AC-ADM-02 | RBAC 权限控制 | 创建"内容审核员"角色（仅 `video.approve` + `article.approve`）→ 分配角色 → 该管理员访问 `user.ban` 接口 → 返回 403 |
| AC-ADM-03 | 审计日志完整性 | 任一管理员执行写操作（如封禁用户）→ `audit_logs` 表新增一条记录，含 admin_id/action/resource/target_id/ip |
| AC-ADM-04 | 工单全生命周期 | 用户提交工单 → 管理员分配 → 处理人回复 → 用户确认 → 关闭 → （可选）用户申诉 → 重新打开 |
| AC-ADM-05 | 风控规则生效 | 创建"关键词过滤"规则 → 用户发送匹配内容 → `risk_hit_logs` 记录命中 |
| AC-ADM-06 | 版权投诉全链路 | 用户提交版权投诉 → 管理员受理 → 下架内容 → 恢复（或驳回） |
| AC-ADM-07 | Feature Flag 灰度 | 创建 Flag `new_player`（whitelist: admin用户, rollout_pct: 10%）→ 非白名单用户检查返回 false |
| AC-ADM-08 | 审批流通过 | 创建审批流（删除视频）→ 审批人批准 → 操作执行 → 审计日志记录 |
| AC-ADM-09 | 仪表盘数据准确 | 仪表盘展示的"今日播放量"与 Redis 实时增量 + MySQL 聚合结果一致 |

---

## 7. 兼容要求

| 编号 | 类型 | 规格 |
|------|------|------|
| BC-1 | 向后兼容 | v1.0 用户端 API 不受管理端改动影响；管理端 API 独立路由前缀 `/api/v1/admin` |
| BC-2 | 向前兼容 | 代码架构支持未来平滑拆分为 Kratos 微服务；handler 文件粒度对应未来服务边界；数据模型预留状态字段 |
| BC-3 | 前端兼容 | `AdminLayout.vue` 统一管理后台布局；侧边栏顺序固定 19 项；路由 `/admin/*` 独立于用户端 |

---

## 8. 排除范围

以下功能明确不在本版本（当前运营中心）范围内：

1. **移动端/小程序** — 当前 Vue SPA 以 PC 端为主，无响应式适配
2. **CDN 实际分发** — 仅有管理接口，未部署实际 CDN
3. **Whisper ASR** — 字幕自动转写 Worker 未实现（`TaskLog.task_type = subtitle_asr` 已预留）
4. **支付/会员/充电** — 商业化功能不在范围
5. **第三方 MCP/Connector 集成** — 虽 UI 预留了连接器管理，未实际对接

---

## 9. 模块依赖关系

```
运营仪表盘 ← 用户/视频/评论数据聚合
├── 人工审核 ← 用户上传触发 (FR-002)
├── 举报处理 ← 用户举报触发 (FR-003-a)
│   └── 工单系统 ← 举报转工单 / 用户直接提交 (FR-003-b)
├── 风控管理 ← 规则引擎 + 黑白名单 (FR-004)
├── 版权管理 ← 用户投诉 (FR-005)
├── BI 报表 ← 用户/视频/播放数据 (FR-006)
├── 客服后台 ← 用户会话 (FR-007)
├── 用户管理 ← users 表 (FR-008)
├── 评论管理 ← comments 表 (FR-009)
├── Banner 管理 ← HomeBanner 表 + OSS (FR-010)
├── 热搜运营 ← Redis hot-search key (FR-011)
├── AI 角色/LLM ← agent_profiles 表 + .env 配置 (FR-012/013)
├── 队列/告警/追踪 ← RabbitMQ + alert_rules 表 (FR-014/015/016)
├── 配置发布 ← FeatureFlag + ReleaseRecord (FR-017)
├── CDN/OSS ← cdn_refresh_tasks + oss_lifecycle_rules (FR-018)
└── RBAC 审计 ← 所有模块复用 AdminRole/Permission/AuditLog (FR-019)
```

---

**文档结束** — 移交至 BMAD Architecture Phase 进行 ADR 提取和架构文档生成。
