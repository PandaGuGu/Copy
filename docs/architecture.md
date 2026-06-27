# 系统架构: Mini-Bili 运营中心

**文档版本:** 1.1
**日期:** 2026-06-27
**作者:** 架构师 Winston（BMAD 框架）
**轨道:** BMad Method
**状态:** 草稿
**来源 PRD:** `bmad-output/prd.md`

> 这是横切技术决策的唯一真相源。所有后续开发任务继承此处记录的**锁定**决策。
> 在此层面捕获对齐的成本约为实现阶段的 1/10。

---

## 目录

1. [系统概述](#1-系统概述)
2. [架构模式](#2-架构模式)
3. [架构决策记录](#3-架构决策记录)
4. [组件设计](#4-组件设计)
5. [数据模型](#5-数据模型)
6. [API 规范](#6-api-规范)
7. [FR/NFR 覆盖矩阵](#7-frnfr-覆盖矩阵)
8. [技术栈](#8-技术栈)
9. [权衡分析](#9-权衡分析)
10. [部署架构](#10-部署架构)
11. [未来考虑](#11-未来考虑)

---

## 1. 系统概述

### 目的

Mini-Bili 运营中心是平台的管理中枢系统，为运营人员（内容审核、风控、客服）和技术运维人员（告警、配置发布、CDN 管理）提供一站式的后台管理能力。系统管理 25 个功能模块，通过 RBAC 细粒度权限模型实现差异化访问控制。

### 范围

**在范围内:**
- 运营管理后台 13 模块：仪表盘、审核、举报工单、风控、版权、BI、客服、用户/评论/Banner/热搜/AI/设置管理
- 技术运维 6 模块：任务队列、告警、链路追踪、配置发布、CDN/OSS、RBAC 审计
- 用户端扩展 6 模块：播放器高级、字幕、评论增强、创作者中心、Feed 推荐、专题活动
- 62+ API 端点（admin 端）、20+ auth 端点、5+ public 端点
- 25+ 数据模型（module_extend.go）+ 8+ RBAC 相关模型

**不在范围内:**
- ML 推荐引擎、直播功能、移动端/小程序、支付/会员商业化、CDN 实际分发、Whisper ASR

### 架构驱动因素

最制约设计的 NFR：

1. **NFR-AUTH-3 (RBAC 授权)** — 25 个模块需要 19 种 `resource:action` 细粒度权限，中间件层拦截
2. **NFR-PERF-1 (模块化单体)** — 满足 ~50 并发管理员，必须为未来微服务拆分预留边界
3. **NFR-AUTH-4 (审计日志)** — 所有写操作自动记录，含 admin_id/action/resource/target_id/ip

### 利益相关者与约束

- **用户:** 运营管理员、内容审核员、技术运维、客服
- **团队:** 1 人全栈开发（PandaGuGu），Windows 环境，Go + Vue3 技术栈
- **现有约束:** Go Gin 单体、Vue3/Vite SPA、MySQL/Redis/RabbitMQ、阿里云 OSS
- **兼容性:** BC-2 要求支持未来平滑拆分为 Kratos 微服务

---

## 2. 架构模式

**模式:** 模块化单体（Modular Monolith）

**论证:**
- 1 人团队维护微服务的运维负担（部署/监控/分布式追踪/服务发现）远超当前规模收益
- 运营中心并发需求低（<50 管理员），单体足以支撑
- 文件级模块拆分（每个功能一个 handler 文件）已为未来微服务拆分预留边界
- BC-2 约束满足：handler 间不互相调用，通过共享 `API` 结构体的 `DB`/`Log`/`Cfg` 进行松耦合

**考虑的替代方案:**
- **Kratos 微服务:** 1 人团队维护 10+ 独立服务的部署、配置、监控成本过高，当前并发无需独立扩缩容
- **纯单体无拆分规划:** 违反 BC-2 要求，未来重构成本指数级增长

**应用方式:**
```
minibili（单进程）
├── internal/handler/       ← 按模块拆分的 API 层（未来微服务的 service 雏形）
│   ├── admin_dashboard.go
│   ├── admin_risk.go
│   ├── admin_rbac.go
│   └── ...（24 个 handler 文件）
├── internal/middleware/    ← 横切关注点（认证/授权/审计）
│   ├── admin_auth.go
│   └── rbac_permission.go
├── internal/model/         ← 共享数据模型
│   └── module_extend.go
└── internal/pkg/           ← 共享工具包
    ├── jwttoken/
    ├── resp/
    └── errcode/
```

微服务拆分路径（未来）：handler 文件 → service 层提取 → 独立 Kratos 服务 → API 网关路由。

---

### 状态管理策略

**前端状态:** Vuex 4.x 集中式状态管理。
- **服务端状态:** 管理后台数据通过 API 响应直接消费，不做客户端缓存（每次页面切换重新请求）。
- **认证状态:** JWT Token 存储在 `localStorage`，通过 Axios 拦截器自动注入 `Authorization` header。
- **路由状态:** `vue-router` 管理管理页面路由，路由守卫检查认证状态 + 权限（通过 `GET /admin/rbac/me/permissions` 获取）。
- **UI 状态:** Vuex store 管理侧边栏折叠/展开、当前管理员信息、权限列表等全局 UI 状态。

**锁定规则:** 所有管理页面通过 `AdminLayout.vue` 统一布局；跨页面共享状态（如管理员信息）存储在 Vuex store，不通过 props 逐层传递。

---

## 3. 架构决策记录（ADR）

> 核心产出物。详见 `bmad-output/decision-log.md`。

| ADR | 标题 | 状态 | 驱动 |
|-----|------|------|------|
| ADR-001 | REST + JSON 信封响应格式 | 已接受 | NFR-PERF-3 |
| ADR-002 | MySQL + GORM AutoMigrate 持久化 | 已接受 | NFR-DATA-1/2/3/4 |
| ADR-003 | 独立管理员 JWT 双 Token 认证 | 已接受 | NFR-AUTH-1/2 |
| ADR-004 | RBAC resource:action 细粒度授权 | 已接受 | NFR-AUTH-3 |
| ADR-005 | 模块化单体架构 (Gin) | 已接受 | NFR-PERF-1 / NFR-EXT-1 |
| ADR-006 | 全写操作自动审计日志 | 已接受 | NFR-AUTH-4 |
| ADR-007 | 统一错误码体系（errcode 包） | 已接受 | NFR-PERF-3 |
| ADR-008 | Feature Flag FNV-1a 灰度策略 | 已接受 | NFR-CONFIG-3 |
| ADR-009 | 审批流多级串行审核 | 已接受 | NFR-AUTH-5 |

---

## 4. 组件设计

### 组件总览

```
                     ┌──────────────────────────┐
                     │   AdminLayout.vue        │
                     │   (Vue3 SPA 管理后台)       │
                     │   侧边栏 19 项          │
                     │   Router: /admin/*       │
                     └──────────┬───────────────┘
                                │ HTTP REST + JWT
                                ▼
              ┌─────────────────────────────────┐
              │    Gin Router (router.go)        │
              │    /api/v1/admin/*              │
              │    ┌───── Middleware Chain ───┐ │
              │    │ 1. CORS                  │ │
              │    │ 2. AdminJWTAuth          │ │
              │    │ 3. RequirePermission     │ │
              │    │ 4. recordAudit (handler) │ │
              │    └──────────────────────────┘ │
              └───────┬──────────┬──────────────┘
                      │          │
         ┌────────────▼──┐  ┌───▼────────────┐
         │  Admin Handlers│  │ Auth Handlers  │
         │  (24 files)    │  │ (admin_auth)   │
         └───┬───┬───┬────┘  └────────────────┘
             │   │   │
     ┌───────▼┐ ┌▼───▼──┐ ┌▼──────────┐
     │ MySQL  │ │ Redis │ │ RabbitMQ   │
     │(GORM)  │ │(Cache)│ │(Async Job)│
     └────────┘ └───────┘ └────────────┘
                      │
              ┌───────▼───────┐
              │  Aliyun OSS   │
              │  mini-bili    │
              └───────────────┘
```

### 组件: Admin Authentication

**职责:** 管理员身份验证和 Token 管理  
**提供的接口:** `POST /admin/auth/login`, `POST /admin/auth/refresh`, `GET /admin/me`  
**需要的接口:** `jwttoken.Manager`, `gorm.DB`（Admin 表）  
**拥有的数据:** `Admin` 模型  
**约束它的 ADR:** ADR-003  
**处理的 NFR:** NFR-AUTH-1, NFR-AUTH-2  

### 组件: RBAC Authorization Middleware

**职责:** 基于 resource:action 的权限拦截  
**提供的接口:** `RequirePermission(db, resource, action) gin.HandlerFunc`  
**需要的接口:** `gorm.DB`, `middleware.AdminID()`  
**拥有的数据:** `AdminRole`, `AdminPermission`, `RolePermission`, `AdminRoleAssignment`  
**约束它的 ADR:** ADR-004  
**处理的 NFR:** NFR-AUTH-3  

### 组件: RBAC Management

**职责:** 角色/权限/管理员分配 CRUD + 审计日志查询 + 审批流管理  
**提供的接口:** `/admin/rbac/*`（19 个端点）  
**需要的接口:** `gorm.DB`, `bcrypt`（管理员密码哈希）  
**拥有的数据:** `AdminRole`, `AdminPermission`, `RolePermission`, `AdminRoleAssignment`, `AuditLog`, `AdminLoginLog`, `ApprovalFlow`, `ApprovalStep`  
**约束它的 ADR:** ADR-004, ADR-006, ADR-009  
**处理的 NFR:** NFR-AUTH-3, NFR-AUTH-4, NFR-AUTH-5  

### 组件: Dashboard

**职责:** 平台数据聚合概览  
**提供的接口:** `GET /admin/dashboard`  
**需要的接口:** `gorm.DB`, `redis.Client`（播放量热数据）  
**拥有的数据:** 聚合查询（跨 users/videos/comments 表）  
**约束它的 ADR:** ADR-001, ADR-002  
**处理的 NFR:** NFR-PERF-3  

### 组件: Content Review

**职责:** 视频/专栏/动态的人工审核  
**提供的接口:** `/admin/videos/*`, `/admin/articles/*`, `/admin/dynamics/*`  
**权限组:** `video.approve`, `article.approve`, `dynamic.manage`  
**约束它的 ADR:** ADR-004, ADR-006  

### 组件: Risk & Ban

**职责:** 风控规则引擎 + 黑白名单 + 封禁管理  
**提供的接口:** `/admin/risk/*`, `/admin/users/:id/ban`  
**权限组:** `risk.manage`, `user.ban`  
**约束它的 ADR:** ADR-004, ADR-006  

### 组件: Ticket & Report

**职责:** 举报受理 + 工单全生命周期  
**提供的接口:** `/admin/reports/*`, `/admin/tickets/*`, `/tickets` (auth)  
**权限组:** `ticket.handle`  
**约束它的 ADR:** ADR-004, ADR-006  

### 组件: Copyright

**职责:** 版权投诉受理 → 下架/恢复内容  
**提供的接口:** `/admin/copyright/*`, `/copyright/complaints` (auth)  
**权限组:** `copyright.handle`  
**约束它的 ADR:** ADR-004, ADR-006  

### 组件: BI Reports

**职责:** 分区/UP主统计、时间序列、报表导出/保存  
**提供的接口:** `/admin/bi/*`  
**权限组:** `dashboard.export`  
**约束它的 ADR:** ADR-004  

### 组件: Customer Service

**职责:** 客服会话管理 + 回复模板  
**提供的接口:** `/admin/cs/*`, `/cs/conversations` (auth)  
**权限组:** `cs.manage`  
**约束它的 ADR:** ADR-004, ADR-006  

### 组件: Ops Monitoring

**职责:** 任务队列可视化、告警管理、链路追踪、系统健康、CDN 刷新、OSS 生命周期  
**提供的接口:** `/admin/ops/*`（20+ 端点）  
**权限组:** `ops.manage`  
**拥有的数据:** `TaskLog`, `AlertRule`, `AlertRecord`, `TraceRecord`, `CDNRefreshTask`, `OSSLifecycleRule`  
**约束它的 ADR:** ADR-004, ADR-002  

### 组件: Config Management

**职责:** Feature Flag 灰度发布 + Release 记录 + 回滚  
**提供的接口:** `/admin/config/*`, `/config/feature-flags/:key` (public)  
**权限组:** `config.manage`  
**拥有的数据:** `FeatureFlag`, `ReleaseRecord`  
**约束它的 ADR:** ADR-008  

---

## 5. 数据模型

> 受 ADR-002 约束。所有开发共享这些实体形态。

### 核心运营实体

| 实体 | 表名 | 用途 | 关键属性 |
|------|------|------|---------|
| `Ticket` | `tickets` | 工单 | id, user_id, type, title, description, status, priority, assignee_id, created_at |
| `TicketMessage` | `ticket_messages` | 工单消息 | id, ticket_id, sender_type(user/admin), sender_id, content |
| `RiskRule` | `risk_rules` | 风控规则 | id, name, type(keyword/rate/behavior), config JSON, enabled, priority |
| `BlackWhiteList` | `black_white_lists` | 黑白名单 | id, type(black/white), target_type(user/ip/device), target_value |
| `RiskHitLog` | `risk_hit_logs` | 命中日志 | id, rule_id, user_id, content, matched_keyword, created_at |
| `CopyrightComplaint` | `copyright_complaints` | 版权投诉 | id, complainant_id, content_type, content_id, evidence_urls JSON, status, handler_id |
| `SavedReport` | `saved_reports` | BI 报表 | id, title, type, config JSON, data JSON, created_by |
| `CSConversation` | `cs_conversations` | 客服会话 | id, user_id, status, assignee_id, last_message_at |
| `CSMessage` | `cs_messages` | 客服消息 | id, conversation_id, sender_type, sender_id, content |
| `CSTemplate` | `cs_templates` | 回复模板 | id, title, category, content |

### 技术运维实体

| 实体 | 表名 | 用途 | 关键属性 |
|------|------|------|---------|
| `TaskLog` | `task_logs` | 异步任务日志 | id, task_type(transcode/subtitle_asr), target_id, status, result, retry_count |
| `AlertRule` | `alert_rules` | 告警规则 | id, name, metric, operator(gt/lt/eq), threshold, channels JSON(d钉/webhook/email) |
| `AlertRecord` | `alert_records` | 告警记录 | id, rule_id, triggered_value, status, acked_by, acked_at |
| `TraceRecord` | `trace_records` | 链路追踪 | id, trace_id, span_id, operation, duration_ms, tags JSON |
| `FeatureFlag` | `feature_flags` | 功能开关 | id, key, description, enabled, rollout_pct, whitelist JSON |
| `ReleaseRecord` | `release_records` | 发布记录 | id, version, description, status, rolled_back, config_snapshot JSON |
| `CDNRefreshTask` | `cdn_refresh_tasks` | CDN 刷新 | id, urls JSON, status, result |
| `OSSLifecycleRule` | `oss_lifecycle_rules` | OSS 生命周期 | id, prefix, days_to_expire, status |

### RBAC 实体

| 实体 | 表名 | 用途 |
|------|------|------|
| `AdminRole` | `admin_roles` | 角色定义 |
| `AdminPermission` | `admin_permissions` | 权限定义 (resource + action) |
| `RolePermission` | `role_permissions` | 角色-权限关联 |
| `AdminRoleAssignment` | `admin_role_assignments` | 管理员-角色关联 |
| `AuditLog` | `audit_logs` | 操作审计日志 |
| `AdminLoginLog` | `admin_login_logs` | 管理员登录日志 |
| `ApprovalFlow` | `approval_flows` | 审批流 |
| `ApprovalStep` | `approval_steps` | 审批步骤 |

### 存储策略

- **主存储:** MySQL 8.x + GORM AutoMigrate（ADR-002）
- **缓存:** Redis — 播放量热数据（INCR，每 10s 刷新 MySQL）、Token 黑名单、弹幕中转
- **文件/对象:** 阿里云 OSS `mini-bili` Bucket — `videos/{id}.mp4` / `covers/{id}.{ext}` / `avatars/{user_id}.{ext}`
- **消息队列:** RabbitMQ — 视频转码任务（`task_type=transcode`），预留 `subtitle_asr`
- **备份策略:** 当前无自动备份，生产环境应启 MySQL binlog 备份 + OSS 版本控制

---

## 6. API 规范

> 受 ADR-001（REST+JSON）、ADR-003（JWT）、ADR-007（错误码）约束。

**协议:** REST over HTTP  
**认证:** Bearer Admin JWT (ADR-003)  
**版本化:** URL 路径 `/api/v1/admin/`  

### 6.1 认证端点

#### `POST /api/v1/admin/auth/login`
**目的:** 管理员登录  
**认证:** 无需  
**请求:** `{ username, password }`  
**响应:** `{ code: 0, data: { access_token, refresh_token, admin: { id, username, display_name } } }`  
**错误:** `40301` = 密码错误, `40400` = 管理员不存在  

#### `POST /api/v1/admin/auth/refresh`
**目的:** 刷新 Access Token  
**认证:** 无需（使用 refresh token）  
**响应:** `{ access_token, refresh_token }`  

### 6.2 读操作（所有已认证管理员）

```
GET /api/v1/admin/me                    — 当前管理员信息
GET /api/v1/admin/dashboard             — 仪表盘数据
GET /api/v1/admin/users                 — 用户列表（分页）
GET /api/v1/admin/users/:id             — 用户详情
GET /api/v1/admin/videos                — 视频列表
GET /api/v1/admin/comments              — 评论列表
GET /api/v1/admin/reports               — 举报列表
GET /api/v1/admin/rbac/me/permissions   — 当前管理员权限（侧边栏控制）
```

### 6.3 写操作（按权限分组）

| 路由组 | 权限 | 示例端点 |
|--------|------|---------|
| `/admin/users/:id/ban\|unban\|delete` | `user.ban` | POST |
| `/admin/videos/:id/approve\|reject\|delete` + chapters/bitrates | `video.approve` | POST/DELETE |
| `/admin/comments/:id/delete` + comment-reports | `comment.delete` | POST |
| `/admin/tickets/*` + `/admin/reports/*` | `ticket.handle` | GET/POST |
| `/admin/risk/*` | `risk.manage` | GET/POST/PUT/DELETE |
| `/admin/copyright/*` | `copyright.handle` | GET/POST |
| `/admin/bi/*` | `dashboard.export` | GET/POST/DELETE |
| `/admin/cs/*` | `cs.manage` | GET/POST/PUT/DELETE |
| `/admin/ops/*` | `ops.manage` | GET/POST/PUT/DELETE |
| `/admin/config/*` | `config.manage` | GET/POST/PUT |
| `/admin/rbac/*` | `rbac.manage` | GET/POST/PUT/DELETE |
| `/admin/hot-search/*` | `hotsearch.manage` | GET/POST/PUT/DELETE |
| `/admin/agent-*` | `agent.manage` | GET/POST/PUT/DELETE |
| `/admin/settings` + `/admin/llm-config` | `setting.manage` | GET/PUT |
| `/admin/home-banners/*` | `banner.manage` | GET/POST/PUT/DELETE |
| `/admin/specials` + `/admin/campaigns` | `special.manage` | GET/POST/PUT/DELETE |
| `/admin/articles/:id/*` | `article.approve` | POST/DELETE |
| `/admin/dynamics/:id/*` | `dynamic.manage` | POST/DELETE |
| `/admin/subtitles/*` | `subtitle.manage` | GET/DELETE |

### 6.4 错误响应约定

所有错误响应遵循：
```json
{ "code": 40300, "msg": "无操作权限: user.ban" }
```

HTTP 状态码归类：
- `200` — 成功
- `400` — 参数/业务错误
- `401` — 未认证
- `403` — 无权限
- `404` — 资源不存在
- `500` — 服务器错误

---

## 7. FR/NFR 覆盖矩阵

> 必需。每项 FR 和每项 NFR 一行。状态 = 已处理 | 部分 | 推迟。

| ID | 类型 | 需求 | 组件 | ADR | 状态 |
|----|------|------|------|-----|------|
| FR-001 | FR | 运营仪表盘 | Dashboard | ADR-001/002 | 已处理 |
| FR-002 | FR | 人工审核（视频/专栏/动态） | Content Review | ADR-004/006 | 已处理 |
| FR-003 | FR | 举报与工单系统 | Ticket & Report | ADR-004/006 | 已处理 |
| FR-004 | FR | 风控与封禁管理 | Risk & Ban | ADR-004/006 | 已处理 |
| FR-005 | FR | 版权与下架管理 | Copyright | ADR-004/006 | 已处理 |
| FR-006 | FR | BI 统计报表 | BI Reports | ADR-004 | 已处理 |
| FR-007 | FR | 客服后台 | Customer Service | ADR-004/006 | 已处理 |
| FR-008 | FR | 用户管理 | User Management | ADR-002 | 已处理 |
| FR-009 | FR | 评论管理 | Comment Management | ADR-002 | 已处理 |
| FR-010 | FR | Banner 管理 | Banner Management | ADR-002/004 | 已处理 |
| FR-011 | FR | 热搜运营 | Hot Search Ops | ADR-002/004 | 已处理 |
| FR-012 | FR | AI 角色管理 | Agent Management | ADR-004 | 已处理 |
| FR-013 | FR | 系统设置与 LLM | Settings & LLM | ADR-002 | 已处理 |
| FR-014 | FR | 队列与任务可视化 | Ops Monitoring | ADR-004 | 已处理 |
| FR-015 | FR | 实时监控与告警 | Ops Monitoring | ADR-004 | 已处理 |
| FR-016 | FR | 日志与链路追踪 | Ops Monitoring | ADR-004 | 已处理 |
| FR-017 | FR | 发布与配置管理 | Config Management | ADR-004/008 | 已处理 |
| FR-018 | FR | CDN 与存储运维 | Ops Monitoring | ADR-004 | 已处理 |
| FR-019 | FR | RBAC 审计 | RBAC Management | ADR-004/006/009 | 已处理 |
| FR-020 | FR | 播放器高级功能 | Player Advanced | ADR-002 | 已处理 |
| FR-021 | FR | 字幕管理 | Subtitle Management | ADR-002/004 | 已处理 |
| FR-022 | FR | 评论增强 | Comment Enhancement | ADR-002 | 已处理 |
| FR-023 | FR | 创作者中心 | Creator Center | ADR-002 | 已处理 |
| FR-024 | FR | Feed 推荐 | Feed & Ranking | ADR-001 | 已处理 |
| FR-025 | FR | 专题与活动页 | Special Pages | ADR-004 | 已处理 |
| NFR-AUTH-1 | NFR | 独立管理员 JWT 认证 | Admin Auth | ADR-003 | 已处理 |
| NFR-AUTH-2 | NFR | Token 刷新轮换 | Admin Auth | ADR-003 | 已处理 |
| NFR-AUTH-3 | NFR | RBAC resource:action 授权 | RBAC Middleware | ADR-004 | 已处理 |
| NFR-AUTH-4 | NFR | 全操作自动审计 | recordAudit (admin_ops.go) | ADR-006 | 已处理 |
| NFR-AUTH-5 | NFR | 多级审批流 | ApprovalFlow | ADR-009 | 已处理 |
| NFR-DATA-1 | NFR | MySQL + GORM AutoMigrate | Data Layer | ADR-002 | 已处理 |
| NFR-DATA-2 | NFR | Redis 缓存 | Cache Layer | ADR-002 | 已处理 |
| NFR-DATA-3 | NFR | OSS 文件存储 | Aliyun OSS | ADR-002 | 已处理 |
| NFR-DATA-4 | NFR | RabbitMQ 异步任务 | Queue Layer | ADR-002 | 已处理 |
| NFR-PERF-1 | NFR | 模块化单体架构 | Architecture | ADR-005 | 已处理 |
| NFR-PERF-2 | NFR | Vue3 SPA 前端 | AdminLayout.vue | — | 已处理 |
| NFR-PERF-3 | NFR | REST + JSON 信封 | All Handlers | ADR-001/007 | 已处理 |
| NFR-PERF-4 | NFR | CORS 中间件 | Gin Middleware | — | 已处理 |
| NFR-CONFIG-1 | NFR | .env 配置管理 | Settings | — | 已处理 |
| NFR-CONFIG-2 | NFR | LLM 运行时配置同步 | Settings & LLM | — | 已处理 |
| NFR-CONFIG-3 | NFR | FNV-1a Feature Flag | Config Management | ADR-008 | 已处理 |
| NFR-EXT-1 | NFR | 微服务就绪（BC-2） | Architecture | ADR-005 | 已处理 |
| NFR-EXT-2 | NFR | 前端模块化 | Vue Router | — | 已处理 |
| NFR-EXT-3 | NFR | 数据模型预留 | Model Layer | ADR-002 | 已处理 |

### 覆盖缺口（部分/推迟）

| ID | 需求 | 缺口 | 状态 |
|----|------|------|------|
| FR-020 | 播放器高级功能 | 前端缺少倍速选择器、画中画按钮、章节面板、码率选择器 UI | 部分 |
| FR-021 | 字幕管理 | 前端缺少字幕时间轴编辑器 UI 和 ASR 自动转写 Worker | 部分 |
| FR-022 | 评论增强 | 前端缺少排序/过滤 UI 和表情系统 | 部分 |
| FR-023 | 创作者中心 | 前端缺少创作者章节管理 UI 和独立数据中心页面 | 部分 |

---

## 8. 技术栈

> 每项选择附带理由。不用"因为它流行"。

| 层级 | 选择 | 版本 | 理由 | ADR |
|------|------|------|------|-----|
| 前端框架 | Vue 3 + Vite | 3.x / 5.x | SPEC v1.0 约束 (NF-5)；纯 SPA 无需 SSR | — |
| 前端状态 | Vuex | 4.x | 管理后台状态集中管理；配合 `vue-router` 路由守卫 | — |
| 前端 UI | Element Plus | 2.x | 中文社区成熟，管理后台组件库（Table/Form/Dialog）丰富 | — |
| 后端语言 | Go | 1.23+ | SPEC v1.0 约束 (NF-6)；标准项目布局 | — |
| 后端框架 | Gin | 1.10+ | 高性能 HTTP 路由；中间件链式组合；社区成熟 | ADR-005 |
| ORM | GORM v2 | 2.x | AutoMigrate 消除 SQL 管理；预加载处理关联查询 | ADR-002 |
| 数据库 | MySQL | 8.x | 关系型数据（用户/视频/权限多表关联）；阿里云 RDS 集成 | ADR-002 |
| 缓存 | Redis | 7.x | 播放量热数据 INCR；弹幕实时通道；Token 黑名单 | ADR-002 |
| 消息队列 | RabbitMQ | 3.x | 视频转码异步解耦；死信队列与重试 | ADR-002 |
| 文件存储 | 阿里云 OSS | — | SPEC v1.0 约束 (NF-7)；单 Bucket 目录前缀分区 | ADR-002 |
| 认证 | JWT (golang-jwt) | 5.x | 无状态认证；双 Token 轮换；独立管理员体系 | ADR-003 |
| 密码哈希 | bcrypt | — | SPEC v1.0 约束 (R-AUTH-2) | — |
| 日志 | Zap | 1.x | 结构化高性能日志 | — |

---

## 9. 权衡分析

### 权衡: 模块化单体 vs 微服务

**决策:** 模块化单体（ADR-005）

**选项:**
- **A - 模块化单体:** 单进程部署，handler 文件级拆分，共享 DB/Redis/RabbitMQ
- **B - Kratos 微服务:** 每个模块独立进程，通过 API 网关通信

| 维度 | 模块化单体 | Kratos 微服务 |
|------|-----------|--------------|
| 部署复杂度 | 1 个二进制 | 10+ 个服务 + 注册中心 + 网关 |
| 调试效率 | 单步调试 | 分布式追踪 |
| 扩缩容 | 整体扩容 | 按模块独立扩容 |
| 团队适配 | 1 人开发 | 3+ 人团队 |

**理由:** 1 人团队维护微服务不可行；当前并发（<50 管理员）无需独立扩缩容；文件级拆分已为未来过渡预留路径。

**接受:** 收益: 部署简单、开发效率高  
**代价:** 无法独立扩缩容、任何模块故障影响整体  
**缓解:** 严格 handler 间不调用规则；模块边界清晰；未来按 handler 文件直接提取为独立服务

**重新审视条件:** 团队 > 3 人或并发 > 500

### 权衡: 自建 RBAC vs Casbin

**决策:** 自建 RBAC（ADR-004）

**选项:**
- **A - 自建 RBAC:** admin_roles + admin_permissions + role_permissions 三表 JOIN
- **B - Casbin:** 策略文件 + 适配器

**理由:** 19 种权限，模式简单（resource:action），自建 3 表 JOIN 即可满足。Casbin 引入 DSL 学习成本和额外依赖。

**接受:** 收益: 代码简单、无额外依赖  
**代价:** 新增更复杂的 ABAC 规则时需重构  
**缓解:** 预留 `RequirePermission` 接口，内部实现可替换

**重新审视条件:** 需要基于属性的访问控制（ABAC）如"仅工作日可操作"时

### 权衡: GORM AutoMigrate vs 数据库迁移工具

**决策:** GORM AutoMigrate（ADR-002）

**选项:**
- **A - GORM AutoMigrate:** 代码定义模型 → 启动时自动建表
- **B - golang-migrate / Flyway:** 独立迁移脚本

**理由:** 1 人开发无需 DBA 审批流程；AutoMigrate 消除版本管理负担。

**接受:** 收益: 零迁移脚本维护  
**代价:** 不支持复杂迁移（如列重命名）；生产环境缺乏迁移版本控制  
**缓解:** 谨慎修改已有模型；不改列名（仅新增）

**重新审视条件:** 生产环境部署或多人协作时切换为 golang-migrate

---

## 10. 部署架构

### 环境

- **开发:** Windows 本地 — `go run cmd/main.go` + `npm run dev`
- **预发布:** 暂无（1 人团队直接上线）
- **生产:** 单台 Linux 服务器（阿里云 ECS）— systemd service

### 拓扑

```
                 Internet
                    │
              ┌─────▼─────┐
              │  Nginx    │ ← 反向代理 + 静态文件 (Vue dist/)
              └─────┬─────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
   ┌─────────┐ ┌───────┐ ┌──────────┐
   │ Gin App │ │ Redis │ │ RabbitMQ │
   │ :8080   │ │ :6379 │ │ :5672    │
   └────┬────┘ └───────┘ └──────────┘
        │
   ┌────▼────┐
   │  MySQL  │
   │  :3306  │
   └─────────┘
        │
   ┌────▼────────┐
   │ Aliyun OSS  │
   │ (外部服务)   │
   └─────────────┘
```

### 策略

- **部署方式:** systemd service 管理 Gin 进程；Nginx 静态文件服务 + 反向代理
- **回滚:** 保留上一版本二进制；`systemctl restart` 即可回滚（配合 ADR-008 Feature Flag 灰度控制）
- **扩缩容:** 当前单实例，未来水平扩展需引入 Redis session 共享 + MySQL 读写分离

---

## 11. B站对标路线图

> 从"仿B站核心链路"到"接近真实B站体验"的系统性差距分析与分阶段规划。
> 差距分为 P0（补齐核心体验）→ P1（建社区生态）→ P2（商业化变现）→ P3（平台扩张）四级。

### 11.1 差距全景

```
               Cakecake 现状          │          真实 B站
  ───────────────────────────────────┼──────────────────────────────────
  内容生产:  视频上传 ✅ / 专栏 ✅     │  + 直播 ❌ / 音频 ❌ / 互动视频 ❌
  内容消费:  HTML5 播放器 🟡          │  + 倍速/PiP/画质切换/快捷键/投屏
  内容发现:  热度排序 🟡              │  + 协同过滤推荐 / 标签话题 / 分区
  社区互动:  评论弹幕投币收藏 ✅       │  + 高级弹幕 / 社区公约 / 弹幕投票
  创作者:    基础管理 🟡              │  + 数据中心 / 认证 / 激励 / 充电
  商业化:    无 ❌                    │  + 大会员 / 充电 / 课堂 / 电商
  平台化:    单PC端 🟡               │  + 移动端 / 开放API / 水印 / DRM
```

### 11.2 分阶段功能需求

#### P0 — 补齐核心体验（1-2 月）: 对标"能看"

| 编号 | 功能 | 差距 | 技术路线 |
|------|------|------|---------|
| FR-026 | 播放器倍速 | 完全缺失 | `VideoPlayerBox.vue` 添加 0.5x/0.75x/1.0x/1.25x/1.5x/2.0x 倍速选择器；HTML5 `<video>.playbackRate` |
| FR-027 | 画中画 (PiP) | 完全缺失 | `document.pictureInPictureEnabled` API；PiP 按钮 + 视频信息浮层 |
| FR-028 | 播放器快捷键 | 完全缺失 | 空格(暂停)、←→(快进后退)、F(全屏)、M(静音)、↑↓(音量) |
| FR-029 | 多清晰度转码 | 仅单码率 H.264 | FFmpeg 多码率转码：1080P@6Mbps / 720P@3Mbps / 480P@1Mbps；`VideoBitrate` 模型已有 |
| FR-030 | 清晰度切换 UI | 完全缺失 | 播放器清晰度选择菜单；无缝切换（记录当前播放位置） |
| FR-031 | 字幕编辑器前端 | 后端就绪 🟡 | 新建 `SubtitleEditor.vue`：时间轴波形可视化 + SRT/VTT 解析 + 拖拽对齐 + 实时预览 |
| FR-032 | ASR 自动转写 | Worker 未实现 | Whisper 模型集成 Worker（`TaskLog.task_type = subtitle_asr` 已预留）→ 输出 SRT → 自动创建字幕 |
| FR-033 | 评论排序/过滤 UI | 前端缺失 | `VdCommentPanelMb.vue`：按热度/时间排序；按"含图"/"UP主点赞"过滤 |
| FR-034 | 视频合集 | 完全缺失 | 新建 `playlists` 表（creator_id, title, cover, video_ids JSON, play_count）；`GET /api/v1/playlists/:id` 连续播放 |
| FR-035 | 创作者数据中心 | 前端缺失 🟡 | 独立页面 `CreatorData.vue`：播放量趋势、粉丝增长、互动率、收益概览；复用 `creator_center.go` 统计 API |

#### P1 — 建社区生态（2-4 月）: 对标"好用"

| 编号 | 功能 | 差距 | 技术路线 |
|------|------|------|---------|
| FR-036 | 推荐算法升级 | 仅热度规则 | **协同过滤** (UserCF/ItemCF) → Redis 预计算相似度矩阵 → 召回层；**内容召回** (标签/分区匹配) → 排序层 (CTR预估)；离线计算 + 在线服务 |
| FR-037 | 视频标签系统 | 完全缺失 | `video_tags` 多对多表；UP主上传时打标签 + 管理员标签库管理；前端标签云展示 |
| FR-038 | 话题系统 | 完全缺失 | `topics` 表（name, description, cover, video_count）；视频关联话题；话题详情页聚合展示 |
| FR-039 | 二级分区 | 仅一级浏览 | 分区树形结构 `zones` 表(id, parent_id, name)；`GET /api/v1/zones` 树形返回；分区排行榜 |
| FR-040 | 认证体系 | 完全缺失 | `user_verifications` 表（user_id, type: personal/organization, status, credentials）；认证标识显示（昵称旁 V 标） |
| FR-041 | 高级弹幕 | 仅基础滚动 | 特效弹幕（渐变色、旋转、弹跳）；UP主特权色；弹幕屏蔽词（用户自定义）；弹幕密度智能调节 |
| FR-042 | 视频水印 | 完全缺失 | FFmpeg overlay 滤镜 → 转码时叠加用户ID水印（右下角半透明）；后台开关控制 |
| FR-043 | 移动端响应式 | PC端为主 | 全局 CSS 媒体查询 + Tailwind `sm/md/lg` 断点；播放器移动端适配（手势控制）；管理后台移动端最小可用 |
| FR-044 | 相关视频推荐 | 完全缺失 | `GET /api/v1/videos/:id/related` — 同标签/同UP主/同分区 + 播放量加权排序；详情页侧栏展示 |

#### P2 — 商业化变现（4-8 月）: 对标"能赚"

| 编号 | 功能 | 差距 | 技术路线 |
|------|------|------|---------|
| FR-045 | 大会员系统 | 完全缺失 | `memberships` 表（user_id, level, expired_at, auto_renew）；支付集成（支付宝/微信）；大会员权益（1080P+、专属弹幕色、去水印、提前看） |
| FR-046 | 充电/打赏 | 完全缺失 | `tips` 表（from_user_id, to_user_id, amount, message）；充电排行榜；UP主收益提现（`withdrawals` 表） |
| FR-047 | 创作激励 | 完全缺失 | `creator_earnings` 表：按播放量/互动量计算收益；月度结算；激励规则后台配置（`incentive_rules` 表） |
| FR-048 | 数据开放平台 | 完全缺失 | `GET /open/v1/*` — API Key 认证；限流（Token Bucket）；文档自动生成（Swagger）；开发者中心页面 |
| FR-049 | AIGC 创作工具 | 完全缺失 | AI 生成封面（集成 ImageGen）；AI 视频摘要/标题建议；AI 评论回复建议；集成现有 LLM 配置 |

#### P3 — 平台扩张（8-12 月+）: 对标"完整"

| 编号 | 功能 | 差距 | 技术路线 |
|------|------|------|---------|
| FR-050 | 直播系统 | 完全缺失 | SRS/ZLMediaKit 流媒体服务器；RTMP 推流 → HLS/WebRTC 播放；直播间（WebSocket 弹幕 + 礼物）；直播回放自动转点播 |
| FR-051 | 电商带货 | 完全缺失 | `shop_items` 表（video_id, product_name, price, link）；视频下方商品卡片；佣金分账 |
| FR-052 | 付费课堂 | 完全缺失 | `courses` 表（title, price, chapters JSON）；课程购买 → 解锁观看；学习进度追踪 |
| FR-053 | 游戏中心 | 完全缺失 | `games` 表 + 分发页；云游戏（可选）；游戏视频聚合 |
| FR-054 | 音频/播客 | 完全缺失 | 音频上传 + 转码；音频播放器组件；播客订阅 |

### 11.3 新增 NFR

| 编号 | 类型 | 需求 | 驱动阶段 |
|------|------|------|---------|
| NFR-MOBILE-1 | 响应式 | 移动端适配，支持 320px-1920px 视口 | P1 (FR-043) |
| NFR-MOBILE-2 | 手势 | 播放器移动端手势：双击暂停、左右滑动快进、上下滑音量/亮度 | P1 |
| NFR-REC-1 | 推荐性能 | 推荐接口延迟 ≤ 100ms；召回候选集 ≥ 500；支持 AB 实验框架 | P1 (FR-036) |
| NFR-REC-2 | 离线计算 | 协同过滤相似度矩阵每日凌晨重算；增量更新每小时执行 | P1 |
| NFR-TRANSCODE-1 | 多码率 | 转码 Pipeline 产出 3 档清晰度；1080P 转码 ≤ 2x 视频时长 | P0 (FR-029) |
| NFR-TRANSCODE-2 | 存储 | 多码率视频 OSS 路径 `videos/{id}/1080p.mp4` / `720p.mp4` / `480p.mp4` | P0 |
| NFR-PAY-1 | 支付安全 | 支付集成必须 HTTPS + 签名验证；金额以分(cent)为单位存储 | P2 (FR-045) |
| NFR-PAY-2 | 对账 | 每日自动对账（支付平台账单 vs 本地流水）；差异告警 | P2 |
| NFR-LIVE-1 | 直播延迟 | RTMP→HLS 延迟 ≤ 5s；WebRTC 延迟 ≤ 1s | P3 (FR-050) |
| NFR-LIVE-2 | 并发 | 单直播间支持 10000 人同时观看 | P3 |
| NFR-OPEN-1 | API 限流 | Token Bucket：每 Key 1000 req/min；429 超限响应 | P2 (FR-048) |
| NFR-I18N-1 | 多语言 | 前端 i18n (vue-i18n)；后端错误码多语言映射 | P1+ |

### 11.4 新增 ADR（扩展）

| ADR | 标题 | 状态 | 驱动阶段 | 核心决策 |
|-----|------|------|---------|---------|
| ADR-010 | 多清晰度转码 Pipeline | 提议 | P0 | FFmpeg 一次输入多路输出；3 档码率；`VideoBitrate` 表管理 |
| ADR-011 | 推荐系统混合架构 | 提议 | P1 | 召回层(协同过滤+标签)→排序层(CTR)→重排层；离线+在线分离 |
| ADR-012 | 视频水印 FFmpeg overlay | 提议 | P1 | 转码时叠加，不可前端去除；用户ID+时间戳；后台可关闭 |
| ADR-013 | 移动端渐进式适配 | 提议 | P1 | 先用 CSS 响应式覆盖核心页面；播放器独立移动端组件；管理后台最小可用 |
| ADR-014 | 支付集成架构 | 提议 | P2 | 统一支付网关抽象（支付宝/微信）；回调幂等处理；对账定时任务 |
| ADR-015 | 直播技术选型 | 提议 | P3 | SRS 流媒体服务器；RTMP 推流 → HLS/WebRTC 拉流；与现有 WebSocket 弹幕复用 |

### 11.5 覆盖矩阵增量（新增 FR/NFR）

| ID | 类型 | 需求 | 组件 | ADR | 状态 |
|----|------|------|------|-----|------|
| FR-026 | FR | 播放器倍速 | Player Advanced | — | 待实施 |
| FR-027 | FR | 画中画 PiP | Player Advanced | — | 待实施 |
| FR-028 | FR | 播放器快捷键 | Player Advanced | — | 待实施 |
| FR-029 | FR | 多清晰度转码 | Transcode Pipeline | ADR-010 | 待实施 |
| FR-030 | FR | 清晰度切换 UI | Player Advanced | ADR-010 | 待实施 |
| FR-031 | FR | 字幕编辑器前端 | Subtitle Editor | — | 待实施 |
| FR-032 | FR | ASR 自动转写 | Whisper Worker | — | 待实施 |
| FR-033 | FR | 评论排序/过滤 UI | Comment Enhancement | — | 待实施 |
| FR-034 | FR | 视频合集/播放列表 | Playlist System | ADR-002 | 待实施 |
| FR-035 | FR | 创作者数据中心 | Creator Center | — | 待实施 |
| FR-036 | FR | 推荐算法升级 | Recommendation Engine | ADR-011 | 待实施 |
| FR-037 | FR | 视频标签系统 | Tag System | ADR-002 | 待实施 |
| FR-038 | FR | 话题系统 | Topic System | ADR-002 | 待实施 |
| FR-039 | FR | 二级分区 | Zone System | ADR-002 | 待实施 |
| FR-040 | FR | 认证体系 | Verification System | ADR-002 | 待实施 |
| FR-041 | FR | 高级弹幕特效 | Danmaku Engine | — | 待实施 |
| FR-042 | FR | 视频水印 | Transcode Pipeline | ADR-012 | 待实施 |
| FR-043 | FR | 移动端响应式 | Frontend Layout | ADR-013 | 待实施 |
| FR-044 | FR | 相关视频推荐 | Recommendation | ADR-011 | 待实施 |
| FR-045 | FR | 大会员系统 | Membership | ADR-014 | 待实施 |
| FR-046 | FR | 充电/打赏 | Tipping | ADR-014 | 待实施 |
| FR-047 | FR | 创作激励 | Creator Incentive | ADR-014 | 待实施 |
| FR-048 | FR | 数据开放平台 | Open API | — | 待实施 |
| FR-049 | FR | AIGC 创作工具 | AI Tools | — | 待实施 |
| FR-050 | FR | 直播系统 | Live Streaming | ADR-015 | 待实施 |
| FR-051 | FR | 电商带货 | E-commerce | — | 待实施 |
| FR-052 | FR | 付费课堂 | Paid Courses | ADR-014 | 待实施 |
| FR-053 | FR | 游戏中心 | Game Center | — | 待实施 |
| FR-054 | FR | 音频/播客 | Audio Content | — | 待实施 |
| NFR-MOBILE-1 | NFR | 响应式适配 | Frontend | ADR-013 | 待处理 |
| NFR-REC-1 | NFR | 推荐性能 | Recommendation | ADR-011 | 待处理 |
| NFR-TRANSCODE-1 | NFR | 多码率转码 | Transcode | ADR-010 | 待处理 |
| NFR-PAY-1 | NFR | 支付安全 | Payment | ADR-014 | 待处理 |
| NFR-LIVE-1 | NFR | 直播延迟 | Live | ADR-015 | 待处理 |
| NFR-OPEN-1 | NFR | API 限流 | Open API | — | 待处理 |

### 11.6 扩展路径

```
当前容量: ~50 并发管理员，单实例，单码率
  │
  ├── P0 达成: 播放器增强 + 多码率转码 + 字幕完善 + 合集 + 创作者数据中心
  │   → 播放体验对标 B站"能看"
  │
  ├── P1 达成: 推荐引擎 + 标签话题 + 认证 + 移动端 + 高级弹幕 + 水印
  │   → 社区生态对标 B站"好用"
  │   → 并发目标: 500 用户，需水平扩展 Gin + Nginx 负载均衡
  │
  ├── P2 达成: 大会员 + 充电 + 激励 + 开放平台 + AIGC
  │   → 商业化对标 B站"能赚"
  │   → 需引入: 支付SDK、对账系统、API 网关限流
  │
  └── P3 达成: 直播 + 电商 + 课堂 + 游戏 + 音频
      → 完整平台对标 B站
      → 需引入: SRS/ZLMediaKit、商品系统、课程系统
      → 架构升级: 微服务拆分（按 ADR-005 路径）
```

### 11.7 重新审视触发条件（汇总自 ADR）

| 触发条件 | 应重新评估的决策 |
|---------|----------------|
| 开始 P0 多码率转码 | ADR-010: 确认 FFmpeg 参数；评估 H.265/AV1 编码 |
| 开始 P1 推荐引擎 | ADR-011: 确认协同过滤 vs 向量召回 mix 比例 |
| 移动端流量 > 20% | ADR-013: 评估独立移动端 SPA 或 PWA |
| 开始 P2 支付 | ADR-014: 选择支付服务商，确认对账方案 |
| 管理并发 > 200 | ADR-004: RBAC 权限缓存到 Redis |
| 团队规模 > 3 人 | ADR-005: 启动微服务拆分 |
| 每日审计日志 > 10 万条 | ADR-006: 审计日志异步写入 + 分表 |
| 流量 > 5 万并发用户 | ADR-001: 评估 GraphQL 聚合查询 |
| 审批流平均等待 > 24h | ADR-009: 引入自动提升机制 |
| 开始 P3 直播 | ADR-015: SRS vs ZLMediaKit 最终选型；WebRTC vs HLS 策略 |
| 需要 ABAC 访问控制 | ADR-004: 评估 Casbin 迁移 |

---

## 附录

### 术语表

| 术语 | 定义 |
|------|------|
| 运营中心 | Mini-Bili 后台管理系统，含运营后台 (13 模块) + 技术运维 (6 模块) + 用户端扩展 (6 模块) |
| 模块化单体 | 单进程部署，handler 文件级模块拆分，为未来微服务拆分预留边界的架构模式 |
| RBAC | 基于角色的访问控制 (Role-Based Access Control)，resource:action 细粒度权限 |
| ADR | 架构决策记录 (Architecture Decision Record)，记录重大设计决策及背景 |
| Feature Flag | 功能开关，支持 FNV-1a hash 分桶 + 白名单 + rollout_pct 三层灰度策略 |
| BMad Method | 项目架构方法论轨道，适用于 1-5 人团队的中型项目 |

### 参考资料

- PRD: `bmad-output/prd.md`
- 决策日志: `bmad-output/decision-log.md`
- 原始 SPEC: `SPEC.md` (v1.0 — 运营中心"明确排除"后推翻)
- 模块审计: `ModuleGapAnalysis_2026-06-24.md`
- 工作记忆: `.workbuddy/memory/MEMORY.md`

### 文档历史

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-06-25 | 架构师 Winston | 初始架构（从现有代码逆向提取） |
| 1.1 | 2026-06-27 | 架构师 Winston | B站对标路线图：新增 FR-026~054（29个功能需求）、NFR 6项、ADR-010~015（6个新决策） |

---

**文档结束** — 验证通过后即可进入下一阶段（Epic 拆分与 Sprint 规划）。
