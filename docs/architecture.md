# 系统架构: Mini-Bili 运营中心

**文档版本:** 2.0
**日期:** 2026-06-28
**作者:** 架构师 Winston（BMAD 框架）
**轨道:** BMad Method
**状态:** 已发布（v2.0，对标 SPEC v2.0）
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
- 运营管理后台 23 模块：仪表盘、首页轮播、热搜运营、用户管理、视频审核、专栏审核、直播管理、动态管理、评论管理、系统设置、举报处理、AI 角色、工单管理、风控管理、版权管理、BI 报表、客服后台、运维监控、配置发布、权限审计、字幕管理、专题活动
- 直播系统（SPEC F12）：SRS 推流、flv.js 播放、WebSocket 实时聊天+礼物、观众追踪、直播历史、管理后台审核/警告/封禁
- 社交体系（SPEC F13）：关注/拉黑、私信、动态发布、收藏夹、投币
- 搜索与发现（SPEC F14）：ES 全文搜索、热搜运营、搜索历史、Feed 推荐、排行榜
- Service 层架构（SPEC F15）：handler → service → DB 三层解耦，`internal/service/` 包
- 技术运维 6 模块：任务队列、告警、链路追踪、配置发布、CDN/OSS、RBAC 审计
- 80+ admin API + 20+ auth API + 5+ public API + 用户端 API
- 82 张数据表 + 30+ 数据模型（含 RBAC 8 模型）

**不在范围内:**
- ML 推荐引擎、移动端/小程序、支付/会员商业化、CDN 实际分发、Whisper ASR

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

### Service 层架构（SPEC F15）

v2.0 引入三层解耦，所有业务逻辑从 handler 迁移到 service：

```
handler（HTTP 请求处理，参数验证/响应格式化）
  → service（业务逻辑，跨表事务/缓存协调/外部服务调用）
    → gorm.DB / redis.Client（数据访问）
```

**实现:**
```
internal/service/
├── services.go           ← Services 容器结构体，聚合所有子 Service
├── video_service.go      ← 视频上传/转码/状态管理/播放量
├── user_service.go       ← 用户注册/登录/关注/投币
└── comment_service.go    ← 评论CRUD/点赞/通知

handler/API 结构体:
  Dependencies.Svcs *service.Services  ← DI 注入，handler 通过 Svcs.Video.XXX() 调用
```

**锁定规则:** 新增业务逻辑优先写入 service；handler 仅保留 HTTP 层职责；service 不直接引用 gin.Context。

### WebSocket 架构

平台有三套独立的 WebSocket 通信通道：

| 通道 | 端点 | 用途 | 技术 | 并发目标 |
|------|------|------|------|---------|
| 弹幕 | `ws://host/ws/danmaku?video_id=X` | 实时弹幕推送 | gorilla/websocket，5s 冷却，敏感词过滤 | 100 在线 ≤200ms |
| 直播聊天 | `ws://host/ws/live?room_id=X` | 直播间聊天+礼物 | 同一 WebSocket 库，消息类型(chat/gift)区分 | 单房间多观众 |
| 私信 | `ws://host/ws/chat?token=X` | 实时私信推送 | JWT 鉴权，双向通信，conversation_id 路由 | 按需 |

**弹幕通道流程:**
```
前端 Canvas 渲染 ← WebSocket ← 后端 Danmaku Hub (goroutine)
                                  ↑
                            POST /api/v1/danmaku (REST 写入)
                                  ↓
                            Redis 热缓存 (200条历史) → MySQL 落库
```

**锁定规则:** 三套 WS 复用心跳机制（30s ping/pong）；消息体统一 JSON 信封 `{type, data, timestamp}`；断线自动重连（指数退避，最大 30s）。

### 推荐引擎架构（规划中 — v2.1）

> 当前状态：全局 `ORDER BY play_count DESC`，所有用户看到同一份榜单。v2.1 计划引入 ItemCF 协同过滤个性化推荐。

#### 四层漏斗

```
全量视频池（~百万级）
  ↓ ① 召回层（百万 → 千）
  - ItemCF 召回: 用户交互过的视频 → 查找相似视频
  - 内容召回: 同分区 / 同标签匹配
  - 热门召回: 时间衰减热门（冷启动兜底）
  - 社交召回: 关注 UP 主的新内容
  ↓ ② 粗排（千 → 百）
  - 加权公式: like×120 + coin×200 + fav×90 + dm×85 + play×1.2 + 时间衰减
  ↓ ③ 精排（百 → 十）
  - 一期规则打散；二期接入 LR/GBDT
  ↓ ④ 重排序
  - 类目打散（避免同类扎堆）+ 频控（Redis 计数器）
  ↓ 用户看到的内容
```

#### 离线计算（每日凌晨 Go 定时任务）

```
用户行为采集（7种）→ 用户-视频交互矩阵 → ItemCF 相似度计算 → MySQL/Redis 存储
     │
     ├── VideoLike（点赞=1.0）
     ├── VideoCoin（投币=3.0，强信号）
     ├── VideoFavorite（收藏=2.0）
     ├── VideoViewHistory（观看>50%进度=0.5）
     ├── Comment（评论=1.5）
     └── Danmaku（弹幕=1.0）

相似度公式: Cosine Similarity on user-item interaction matrix
输出表: video_similarities (video_id_a, video_id_b, score, updated_at)
```

#### 在线服务（实时 API）

```
GET /api/v1/feed/recommendation?user_id=X
  → 1. 查用户最近 N 条交互 (Redis/MySQL)
  → 2. 查 video_similarities 表召回相似视频 Top-K
  → 3. 融合热门召回（同一批，加权比例 3:1）
  → 4. 内容召回（同 zone，加权比例 2:1）
  → 5. 去重 + 过滤已看 → 返回
```

#### 冷启动策略

| 场景 | 策略 |
|------|------|
| 新用户（<5 条交互） | 100% 热门召回 + 多样性打散 |
| 新视频（<100 播放） | 内容相似度提权 ×2.0，时效加权 |
| 冷门分区 | 降低 ItemCF 阈值，扩大召回窗口 |

#### 新增数据模型

| 表名 | 用途 | 关键字段 |
|------|------|---------|
| `video_similarities` | 视频相似度矩阵 | video_id_a, video_id_b, score(0-1), updated_at |
| `user_embedding_cache` | 用户兴趣向量（Redis） | user_id → {video_ids + weights} |
| `rec_exposure_log` | 推荐曝光日志 | user_id, video_id, position, feed_type, created_at |

#### 评估体系

| 指标 | 离线 | 在线（AB实验） |
|------|------|---------------|
| 核心 | Precision@K, Recall@K, NDCG@K | CTR, 人均播放数 |
| 护栏 | 覆盖率（长尾视频被推荐比例） | 停留时长, 互动率 |
| 长期 | — | 7日留存, 新增关注数 |

**锁定规则:** ItemCF 每日凌晨全量重算（非增量）；相似度阈值≥0.15 入表；在线服务延迟 ≤50ms（Redis 缓存）；新视频 48h 内容召回兜底。

### ES 搜索架构

```
用户搜索请求
  ↓
GET /api/v1/search?q=X&type=video|article|user
  ↓
ES 全文检索（ik 中文分词）
  ↓
返回 ID 列表 → MySQL 补全详情 → JSON 响应
```

**索引策略:**
- `videos` 索引: title, description, tags (ik_smart 分词)
- `articles` 索引: title, content (ik_max_word 分词)
- `users` 索引: username, nickname (keyword + ik_smart)
- 热搜运营: Redis Sorted Set 热词 + 管理后台人工干预
- 搜索历史: Redis List per user，最大 50 条

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
| ADR-016 | ItemCF 协同过滤推荐引擎 | 部分实施（MMR/DPP 重排序已落地） | NFR-REC-1/2 |

---

## 4. 组件设计

### 组件总览

```
                     ┌──────────────────────────────────┐
                     │   Vue 3 SPA                      │
                     │   ┌────────────┐ ┌─────────────┐ │
                     │   │ AdminLayout│ │ UserLayout  │ │
                     │   │ /admin/*   │ │ /* (用户端)  │ │
                     │   └─────┬──────┘ └──────┬──────┘ │
                     └─────────┼───────────────┼────────┘
                               │ HTTP REST + JWT│
                               ▼                ▼
              ┌─────────────────────────────────────────┐
              │         Gin Router (router.go)           │
              │    /api/v1/admin/*  │  /api/v1/*         │
              │    ┌───── Middleware Chain ───────────┐  │
              │    │ 1. CORS                           │  │
              │    │ 2. JWT Auth (Admin / User 双体系) │  │
              │    │ 3. Trace (middleware/trace.go)    │  │
              │    │ 4. RequirePermission (admin 路由) │  │
              │    │ 5. recordAudit (写操作 handler)   │  │
              │    └───────────────────────────────────┘  │
              └───────┬──────────────┬───────────────────┘
                      │              │
         ┌────────────▼──┐  ┌───────▼──────────┐
         │ Admin Handlers│  │  User Handlers   │
         │ (24 files)    │  │  (auth/video/     │
         │               │  │   comment/social/ │
         │               │  │   live/search)    │
         └───┬───────────┘  └───┬──────────────┘
             │                  │
             │    ┌─────────────▼──────────────┐
             │    │   Service Layer             │
             │    │   internal/service/         │
             │    │   ├── services.go           │
             │    │   ├── video_service.go      │
             │    │   ├── user_service.go       │
             │    │   └── comment_service.go    │
             │    └─────────────┬──────────────┘
             │                  │
     ┌───────▼──────┬───────────▼───────┬──────────┐
     │    MySQL     │     Redis         │ RabbitMQ │
     │   (GORM)     │   (Cache/Queue)   │(Async)   │
     └───────┬──────┴───────────────────┴──────────┘
             │
     ┌───────▼───────┐     ┌────────────────┐
     │  Aliyun OSS   │     │ Elasticsearch  │
     │  mini-bili    │     │ (全文搜索)     │
     └───────────────┘     └────────────────┘
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

### 用户端核心组件

#### 组件: Video Player

**职责:** HTML5 视频播放器（含弹幕 Canvas 叠加层）  
**提供的接口:** `VideoPlayerBox.vue` — 播放/暂停/进度/音量/全屏 + 弹幕发送/显示 + 章节面板  
**需要的接口:** `GET /api/v1/videos/:id`, `GET /api/v1/danmaku/:video_id`, WebSocket 弹幕通道  
**拥有的数据:** `Video`, `Danmaku`, `VideoChapter`  
**约束它的 ADR:** ADR-001

#### 组件: Social System

**职责:** 关注/拉黑/私信/动态/收藏/投币  
**提供的接口:** `/follows/*`, `/messages/*`, `/dynamics/*`, `/favorites/*`, `/coins/*`  
**需要的接口:** `internal/service/user_service.go`  
**拥有的数据:** `Follow`, `Block`, `Message`, `Dynamic`, `Favorite`, `CoinLedger`  
**约束它的 ADR:** ADR-002, ADR-003

#### 组件: Live Streaming

**职责:** 直播间创建、SRS 推流、flv.js 播放、WebSocket 聊天+礼物  
**提供的接口:** `/live/rooms/*`, `/live/chat/*`, `/live/gifts/*`  
**需要的接口:** SRS 回调、flv.js 播放器  
**拥有的数据:** `LiveRoom`, `LiveChatMessage`, `LiveGift`  
**约束它的 ADR:** ADR-015

### 前端共享组件体系（SPEC F16）

```
src/
├── components/admin/
│   ├── AdminDataTable.vue    ← 统一搜索+表格+分页（已接入 9 个 admin 页面）
│   └── AdminFormDialog.vue   ← 统一新增/编辑弹窗
├── utils/
│   └── admin-helpers.js      ← 共享 formatTime() 等工具函数
└── api/admin/                ← 18 模块模块化 API（按模块拆分文件）
    ├── auth.js, banner.js, video.js, comment.js,
    ├── user.js, rbac.js, cs.js, ticket.js, copyright.js,
    ├── risk.js, bi.js, ops.js, config.js, special.js,
    ├── subtitle.js, dashboard.js, dynamic.js, article.js
```

**锁定规则:** 所有新增 admin 页面必须通过 `@/api/admin/` barrel 导入 API 方法；列表页优先使用 `AdminDataTable`；表单弹窗优先使用 `AdminFormDialog`。  

---

## 5. 数据模型

> 受 ADR-002 约束。所有开发共享这些实体形态。

### v1.0 核心实体（用户端）

| 实体 | 表名 | 用途 | 关键属性 |
|------|------|------|---------|
| `User` | `users` | 用户 | id, username, email, password(bcrypt), nickname, avatar, bio, status |
| `Video` | `videos` | 视频 | id, user_id, title, description, cover_url, duration, status, play_count, zone_id, bitrate_id |
| `VideoBitrate` | `video_bitrates` | 多码率 | id, video_id, resolution(1080p/720p/480p), url, bitrate |
| `VideoChapter` | `video_chapters` | 章节 | id, video_id, title, start_time, end_time |
| `Comment` | `comments` | 评论 | id, user_id, video_id, content, parent_id, root_id, level, like_count, is_pinned, is_featured |
| `Danmaku` | `danmakus` | 弹幕 | id, user_id, video_id, content, position_sec, color, type, mode |
| `Article` | `articles` | 专栏 | id, user_id, title, content, cover_url, category, view_count, status |
| `Dynamic` | `dynamics` | 动态 | id, user_id, content, images JSON, type(text/image/video_share) |
| `Notification` | `notifications` | 通知 | id, user_id, type(comment/like/follow/system), content, is_read, target_id |
| `Banner` | `home_banners` | 首页轮播 | id, title, image_url, link_url, sort_order, enabled |
| `HotSearch` | `hot_searches` | 热搜词 | id, keyword, search_count, rank, is_manual, enabled |

### 社交/直播/搜索实体

| 实体 | 表名 | 用途 | 关键属性 |
|------|------|------|---------|
| `Follow` | `follows` | 关注关系 | id, follower_id, followee_id, group_id, created_at |
| `FollowGroup` | `follow_groups` | 关注分组 | id, user_id, name, sort_order |
| `Block` | `blocks` | 拉黑 | id, user_id, blocked_user_id, reason |
| `Message` | `messages` | 私信 | id, from_user_id, to_user_id, content, is_read, conversation_id |
| `Favorite` | `favorites` | 收藏夹 | id, user_id, name, description, is_public, count |
| `FavoriteItem` | `favorite_items` | 收藏项 | id, favorite_id, content_type(video/article), content_id |
| `CoinLedger` | `coin_ledgers` | 投币流水 | id, from_user_id, to_user_id, video_id, coins, created_at |
| `LiveRoom` | `live_rooms` | 直播间 | id, user_id, title, cover_url, push_url, play_url, status(live/offline), viewer_count |
| `LiveChatMessage` | `live_chat_messages` | 直播聊天 | id, room_id, user_id, content, type(chat/gift), gift_id |
| `LiveGift` | `live_gifts` | 礼物 | id, name, icon_url, price, effect_type |
| `ESIndex` | (Elasticsearch) | 搜索索引 | 视频/文章/用户全文检索，中文分词 |

### 运营后台核心实体

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
- **缓存:** Redis — 播放量热数据（INCR，每 10s 刷新 MySQL）、Token 黑名单（logout/refresh 后加入，TTL=Access Token 过期时间）、弹幕中转（Pub/Sub 模式广播到 WebSocket Hub）、热搜词（Sorted Set ZINCRBY + 定时 decay）、搜索历史（per-user List，LRANGE 取最近 50 条）
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

### 6.3 用户端 API（用户 JWT 认证）

```
POST /api/v1/auth/register            — 用户注册
POST /api/v1/auth/login               — 用户登录
POST /api/v1/auth/refresh             — Token 刷新
GET  /api/v1/users/me                 — 个人信息
PUT  /api/v1/users/me                 — 更新个人信息
POST /api/v1/users/me/avatar          — 上传头像
PUT  /api/v1/users/me/password        — 修改密码
GET  /api/v1/videos                   — 视频列表（首页/分区/排行）
GET  /api/v1/videos/:id               — 视频详情（含弹幕历史）
POST /api/v1/videos                   — 上传视频
POST /api/v1/danmaku                  — 发送弹幕
GET  /api/v1/danmaku/:video_id        — 获取弹幕历史
GET  /api/v1/comments                 — 评论列表（3级嵌套）
POST /api/v1/comments                 — 发表评论
POST /api/v1/comments/:id/like        — 评论点赞
POST /api/v1/follows                  — 关注用户
DELETE /api/v1/follows/:id            — 取关
GET  /api/v1/messages                 — 私信列表
POST /api/v1/messages                 — 发送私信
GET  /api/v1/favorites                — 收藏夹列表
POST /api/v1/favorites                — 创建收藏夹
POST /api/v1/favorites/:id/items      — 添加收藏
POST /api/v1/coins                    — 投币
GET  /api/v1/dynamics                 — 动态列表
POST /api/v1/dynamics                 — 发布动态
GET  /api/v1/search                   — ES 全文搜索
GET  /api/v1/live/rooms               — 直播间列表
POST /api/v1/live/rooms               — 创建直播间
GET  /api/v1/live/rooms/:id           — 直播间详情（含播放地址）
```

### 6.4 写操作（按权限分组）

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

### 6.5 错误响应约定

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
| NFR-REL-1 | NFR | MySQL 每日备份 | 运维脚本 | ADR-002 | 待处理 |
| NFR-REL-2 | NFR | 灾难恢复方案（RPO=24h, RTO=4h） | 运维流程 | ADR-002 | 待处理 |
| NFR-REL-3 | NFR | 健康检查端点 | Ops Monitoring (GET /ops/health) | — | 已处理 |
| NFR-REL-4 | NFR | 优雅降级（非核心功能优先关闭） | Gin Middleware | — | 待处理 |
| NFR-REL-5 | NFR | 错误率目标 ≤1% (5xx/1h 滚动窗口) | 监控系统 | — | 待处理 |
| NFR-AVAIL-1 | NFR | 正常运行时间目标 99%（非关键系统） | 部署架构 | — | 已处理 |
| NFR-AVAIL-2 | NFR | 数据库恢复验证（每季度演练） | 运维流程 | ADR-002 | 待处理 |
| NFR-AVAIL-3 | NFR | 监控告警（CPU/内存/磁盘/错误率） | Ops Monitoring (POST /ops/alerts/evaluate) | — | 已处理 |
| NFR-SEC-1 | NFR | 生产环境 HTTPS/TLS 1.2+ | Nginx 配置 | — | 待处理 |
| NFR-SEC-2 | NFR | 密码 bcrypt 哈希（cost=12） | User/Admin Model | ADR-003 | 已处理 |
| NFR-SEC-3 | NFR | 输入验证（GORM tag + Gin binding） | All Handlers | ADR-001 | 已处理 |
| NFR-SEC-4 | NFR | API 限流（未来：Token Bucket per IP） | Middleware | — | 推迟 |
| NFR-OBS-1 | NFR | 结构化日志（Zap JSON 格式） | Logger (middleware/logger.go) | — | 已处理 |
| NFR-OBS-2 | NFR | 请求链路追踪（trace_id 贯穿） | Trace Middleware (middleware/trace.go) | — | 已处理 |
| NFR-OBS-3 | NFR | 系统指标采集（CPU/内存/QPS/延迟） | Ops Monitoring | — | 已处理 |
| NFR-OBS-4 | NFR | 集中式日志聚合（Elasticsearch/Loki） | 基础设施 | — | 推迟 |
| NFR-OBS-5 | NFR | 错误追踪与告警（Sentry 或自建） | 基础设施 | — | 推迟 |
| NFR-COMP-1 | NFR | 用户数据保留（账号存续期间 + 删除后 30 天清理） | User Service | — | 待处理 |
| NFR-COMP-2 | NFR | 审计日志不可篡改（append-only） | AuditLog Model | ADR-006 | 已处理 |
| NFR-COMP-3 | NFR | 用户数据删除权（账号注销 API） | User Service | — | 推迟 |
| NFR-COMP-4 | NFR | 敏感信息脱敏（日志中隐藏密码/token） | Logger Middleware | — | 待处理 |
| NFR-COST-1 | NFR | 月基础设施成本 ≤500 CNY | 部署架构 | — | 已处理 |
| NFR-COST-2 | NFR | OSS 生命周期自动清理（30 天临时文件） | OSSLifecycleRule | — | 已处理 |
| NFR-COST-3 | NFR | 单实例资源规格规划（2C4G ECS + 1C2G RDS） | 部署架构 | — | 待处理 |
| NFR-DI-1 | NFR | 多表写操作使用 GORM 事务 | Service Layer | ADR-002 | 已处理 |
| NFR-DI-2 | NFR | 外键约束确保引用完整性 | GORM Model | ADR-002 | 已处理 |
| NFR-DI-3 | NFR | 强一致性保证（单 MySQL 实例内） | Data Layer | ADR-002 | 已处理 |
| NFR-MAINT-1 | NFR | 编译验证 `go build ./...` 零错误 | CI/开发流程 | — | 已处理 |
| NFR-MAINT-2 | NFR | 代码风格统一（go fmt + ESLint） | 开发流程 | — | 已处理 |
| NFR-UA-1 | NFR | PC 端浏览器兼容（Chrome/Firefox/Edge 最新版） | Vue 3 SPA | — | 已处理 |
| NFR-UA-2 | NFR | 中文界面全覆盖 | 前端 i18n | — | 已处理 |

### 覆盖缺口（部分/推迟）

| ID | 需求 | 缺口 | 状态 |
|----|------|------|------|
| FR-020 | 播放器高级功能 | Admin CRUD 已就绪；用户端缺少倍速选择器、画中画按钮、章节面板、码率切换 UI | 部分 |
| FR-021 | 字幕管理 | Admin CRUD + SubtitleManage 已就绪；用户端缺少字幕时间轴编辑器 UI 和 ASR 自动转写 Worker | 部分 |
| FR-022 | 评论增强 | Admin CRUD 已就绪；用户端缺少排序/过滤 UI 和表情系统 | 部分 |
| FR-023 | 创作者中心 | API 已就绪（creator_center.go）；用户端缺少创作者章节管理 UI 和独立数据中心页面 | 部分 |

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

- **开发:** Windows 本地 — `mini-bili.exe`（Go 编译产物）+ `npm run dev`（Vue 前端 HMR）
- **容器化:** `docker-compose.yml` — MySQL + Redis + RabbitMQ 基础设施容器化
- **预发布:** 暂无（1 人团队直接上线）
- **生产:** 单台 Linux 服务器（阿里云 ECS）— systemd service 管理进程

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
  内容生产:  视频上传 ✅ / 专栏 ✅     │  + 直播 ✅ / 音频 ❌ / 互动视频 ❌
  内容消费:  HTML5 播放器 🟡          │  + 倍速/PiP/画质切换/快捷键/投屏
  内容发现:  热度排序 🟡              │  + 协同过滤推荐 / 标签话题 / 分区
  社区互动:  评论弹幕投币收藏 ✅       │  + 高级弹幕 / 社区公约 / 弹幕投票
  创作者:    基础管理 🟡              │  + 数据中心 / 认证 / 激励 / 充电
  商业化:    无 ❌                    │  + 大会员 / 充电 / 课堂 / 电商
  平台化:    单PC端 🟡               │  + 移动端 / 开放API / 水印 / DRM
```

### 11.2 分阶段功能需求

#### P0 — 补齐核心体验（1-2 月）: 对标"能看"

> **注：** 直播系统（SRS 推流、flv.js 播放、WebSocket 聊天+礼物、直播审核）已在 v2.0 实现，不列入 P0。

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
| NFR-LIVE-1 | 直播延迟 | FLV 播放延迟 ≤ 3s；WebSocket 聊天实时推送 | **已达标**（v2.0） |
| NFR-LIVE-2 | 并发 | 单直播间 WebSocket 架构支持多观众并发 | **已达标**（v2.0） |
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
| ADR-015 | 直播技术选型 | **已实施**（v2.0） | P3 | SRS 流媒体服务器；RTMP 推流 → FLV 拉流；与现有 WebSocket 弹幕复用 |
| ADR-016 | ItemCF 推荐引擎 | 部分实施 | P1 | 用户行为交互矩阵 → Cosine 相似度 → 多路召回融合；Go 离线计算 + Redis 在线服务 |

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
| FR-036 | FR | 推荐算法升级 | Recommendation Engine | ADR-011 / ADR-016 | 部分实施 |
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
| FR-050 | FR | 直播系统 | Live Streaming | ADR-015 | **已实施**（v2.0） |
| FR-051 | FR | 电商带货 | E-commerce | — | 待实施 |
| FR-052 | FR | 付费课堂 | Paid Courses | ADR-014 | 待实施 |
| FR-053 | FR | 游戏中心 | Game Center | — | 待实施 |
| FR-054 | FR | 音频/播客 | Audio Content | — | 待实施 |
| NFR-MOBILE-1 | NFR | 响应式适配 | Frontend | ADR-013 | 待处理 |
| NFR-REC-1 | NFR | 推荐性能 | Recommendation | ADR-011 / ADR-016 | 部分实施 |
| NFR-TRANSCODE-1 | NFR | 多码率转码 | Transcode | ADR-010 | 待处理 |
| NFR-PAY-1 | NFR | 支付安全 | Payment | ADR-014 | 待处理 |
| NFR-LIVE-1 | NFR | 直播延迟 | Live | ADR-015 | **已处理**（v2.0） |
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
  └── P3 达成: 电商 + 课堂 + 游戏 + 音频
      → 完整平台对标 B站
      → 需引入: 商品系统、课程系统
      → 架构升级: 微服务拆分（按 ADR-005 路径）

> 注：直播系统（P3 原 FR-050）已在 v2.0 提前实施完成，不含电商带货。
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
| 开始 P3 直播 | ADR-015: SRS vs ZLMediaKit 最终选型；WebRTC vs HLS 策略 | **已达成**（v2.0，SRS + FLV） |
| 需要 ABAC 访问控制 | ADR-004: 评估 Casbin 迁移 |
| 推荐系统上线 | ADR-016: ItemCF 离线任务每日凌晨重算；相似度阈值 0.15；Redis 缓存 top-200 |
| 视频量 > 10 万 | ADR-016: ItemCF 矩阵过大 → 升级为 Embedding 召回（Item2Vec/DSSM） |

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
| 1.2 | 2026-06-28 | 架构师 Winston | P0 修复：直播纳入范围、数据模型补全 v1.0 核心实体、API 计数更新、部署描述修正、FR/路线图状态修正 |
| 2.0 | 2026-06-28 | 架构师 Winston | 全面重评估：P1 Service层+WebSocket+ES+组件补全，P2 NFR全面覆盖（可靠性/可用性/安全/可观测/合规/成本/数据完整 7类25+条目），P3 组件图更新+版本同步 |

---

**文档结束** — 验证通过后即可进入下一阶段（Epic 拆分与 Sprint 规划）。
