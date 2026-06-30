# 系统架构: Cakecake（Mini-Bili）全栈视频社交平台

**文档版本:** 3.0
**日期:** 2026-06-30
**作者:** 架构师 Winston（BMAD 框架）
**轨道:** BMad Method
**状态:** 已发布（v3.0，对标 SPEC v2.1 + 审计修正）
**来源 PRD:** `SPEC.md`（充当 PRD 角色）
**审计依据:** `docs/architecture-audit-2026-06-28.md`

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

Cakecake 是仿 B 站核心链路的全栈视频社交平台，后端 Go 模块名 `minibili`。系统涵盖用户认证、视频上传/转码、实时弹幕、多级评论、直播、社交体系（关注/私信/动态/收藏/投币）、ES 全文搜索、Feed 推荐、风控引擎，以及 **23 个运营后台模块**（全前后端对齐）。

### 范围

**在范围内:**
- **用户端:** 注册/登录（JWT 双 Token）、视频上传（≤500MB/≤30min→FFmpeg H.264 MP4→OSS）、弹幕 WebSocket（≤200ms, 5s 冷却+敏感词）、3 级评论（视频/文章/动态三套独立表）、点赞/投币/收藏（硬币经济）、关注/拉黑/私信（WebSocket 实时）、直播（SRS 推流+flv.js + WebSocket 聊天+礼物+弹幕飘屏）、Feed 推荐（MMR 多样性排序+协同过滤）、ES 全文搜索、历史追踪、每日任务
- **运营后台 23 模块:** 数据概览、首页轮播、热搜运营、用户管理、视频审核、专栏审核、直播管理、动态管理、评论管理、系统设置、举报处理、AI 角色、工单管理、风控管理、版权管理、BI 报表、客服后台、运维监控 5 合 1、配置发布、权限审计、播放器高级、字幕管理、Feed 推荐
- **社交体系:** 关注/取关、拉黑（双向互阻）、关注分组、多收藏夹、投币（coin_ledgers）、图文动态发布、私信 WebSocket 实时推送
- **搜索与发现:** ES 全文搜索、热搜运营（Redis 热词+管理干预）、搜索历史、Feed 推荐（规则/热度+MMR 重排序）、排行榜
- **Service 层架构:** handler → service → DB 三层解耦，`internal/service/` 包（19 个 service 文件）
- **基础设施:** MySQL 8.x + Redis 7.x + RabbitMQ 3.x + 阿里云 OSS + Elasticsearch 8.x（可选）+ SRS 5.x

**不在范围内:**
- 支付/会员/充电等任何商业化功能（Non-Commercial License）
- ML 推荐算法（当前为规则/热度排序 + MMR 多样性重排序）
- 移动端/小程序适配
- CDN 实际分发（仅有管理 CRUD 接口）
- Whisper ASR（Worker 预留但未实现）

### 架构驱动因素

最制约设计的 NFR（从 SPEC.md 提取）：

1. **NFR-3（鉴权）:** 用户 JWT + Admin JWT 双体系隔离；RBAC resource:action 细粒度（23 种权限码）；全写操作审计
2. **NFR-1（并发）:** 弹幕 100 人在线 ≤200ms；运营后台 ~50 并发管理员
3. **NFR-2（存储）:** MySQL + Redis + RabbitMQ + OSS 四层存储
4. **NFR-4（API）:** RESTful，JSON 信封统一响应格式，统一错误码
5. **NFR-6（配置）:** .env 文件管理 + Feature Flag FNV-1a hash 灰度

### 代码规模（实测数据 2026-06-30）

| 指标 | 数值 |
|------|------|
| Go 源文件 | **193** 个（`internal/` 目录） |
| GORM AutoMigrate 模型 | **86** 个 |
| RBAC 权限码 | **23** 种（`resource:action` 格式） |
| Admin API 端点 | **~190** 个 |
| 用户端 API 端点 | **~140** 个 |
| 公开只读端点 | **~46** 个 |
| WebSocket 通道 | **3** 套（弹幕/私信/直播聊天） |
| 总路由注册 | **~380** 行 |
| Vue 3 前端页面 | **24+** 个 admin 页面 + 用户端全栈 |

### 利益相关者与约束

- **用户:** 普通用户、UP 主、运营管理员、内容审核员、客服、技术运维
- **团队:** 1 人全栈开发（PandaGuGu），Windows 环境，Go + Vue 3 技术栈
- **现有约束:** Go Gin 模块化单体、Vue 3 + Vite SPA、MySQL/Redis/RabbitMQ、阿里云 OSS
- **兼容性:** BC-2 要求支持未来平滑拆分为 Kratos 微服务

---

## 2. 架构模式

**模式:** 模块化单体（Modular Monolith）

**论证:**
- 1 人团队维护微服务的运维负担远超当前规模收益
- 运营中心并发需求低（<50 管理员），用户端并发可控，单体足以支撑
- 文件级模块拆分（每个功能一个 handler 文件，共 83 个 handler 文件）已为未来微服务拆分预留边界
- BC-2 约束满足：handler 间不互相调用，通过共享 `API` 结构体的 `DB`/`Log`/`Svcs` 进行松耦合
- 三层架构（handler → service → DB）确保业务逻辑隔离

**考虑的替代方案:**
- **Kratos 微服务:** 1 人团队维护 10+ 独立服务的部署、配置、监控成本过高，当前并发无需独立扩缩容
- **纯单体无拆分规划:** 违反 BC-2 要求，未来重构成本指数级增长

**应用方式:**
```
minibili（单进程）
├── internal/handler/         ← 83 个 handler 文件，按模块拆分
│   ├── admin_*.go            ← 25 个运营后台 handler
│   ├── auth.go, video.go, …  ← 用户端 handler
│   ├── router.go             ← 路由注册（~380 条）
│   └── deps.go               ← API 结构体（DI 依赖注入容器）
├── internal/service/         ← 业务逻辑层（19 个文件）
│   ├── services.go           ← Services 容器
│   ├── video_service.go      ← 视频 CRUD + 状态管理
│   ├── user_service.go       ← 用户管理 + 社交
│   ├── comment_service.go    ← 评论 + 通知
│   ├── feed_service.go       ← Feed 推荐 + MMR 重排序
│   └── ...
├── internal/middleware/      ← 横切关注点（认证/授权/追踪）
├── internal/model/           ← 86 个 GORM 模型
├── internal/data/            ← 数据层（DB + migrate + rbac_seed）
├── internal/worker/          ← RabbitMQ 消费者（转码 Worker）
├── internal/ws/              ← WebSocket Hub（弹幕/私信/直播）
└── internal/pkg/             ← 共享工具包（jwttoken/resp/errcode）
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
├── services.go           ← Services 容器，聚合所有子 Service（DI 注入）
├── video_service.go      ← 视频上传/转码/状态管理/播放量
├── video_publish.go      ← 视频发布逻辑
├── user_service.go       ← 用户注册/登录/关注/投币
├── user_profile.go       ← 用户资料管理
├── comment_service.go    ← 评论CRUD/点赞/通知
├── feed_service.go       ← Feed 推荐（MMR/DPP）
├── rerank.go             ← 重排序算法（MMR）
├── playcount.go          ← Redis 播放计数 10s 落库
├── search_hot.go         ← 热搜聚合
├── search_suggest.go     ← 搜索建议
├── hot_search_admin.go   ← 热搜运营
├── hot_search_feed.go    ← 热搜 Feed
├── hot_search_layout.go  ← 热搜布局
├── agent.go              ← AI Agent 对话
├── article_publish.go    ← 文章发布
└── danmaku_relay.go      ← 弹幕中继
```

**锁定规则:** 新增业务逻辑优先写入 service；handler 仅保留 HTTP 层职责；service 不直接引用 `gin.Context`。

### WebSocket 架构

平台有三套独立的 WebSocket 通信通道：

| 通道 | 端点 | 用途 | 技术 | 并发目标 |
|------|------|------|------|---------|
| 弹幕 | `ws://host/ws/danmaku?video_id=X&token=X` | 实时弹幕推送 | gorilla/websocket，5s 冷却，敏感词过滤，Canvas 多轨道 | 100 在线 ≤200ms |
| 私信 | `ws://host/ws/chat?token=X` | 实时私信推送 | JWT 鉴权，双向通信，conversation_id 路由 | 按需 |
| 直播聊天 | `ws://host/ws/live?room_id=X&token=X` | 直播间聊天+礼物+弹幕飘屏 | 同一 WebSocket 库，消息类型(chat/gift/system/admin_warning)区分 | 单房间多观众 |

**锁定规则:** 三套 WS 复用心跳机制（30s ping/pong）；消息体统一 JSON 信封 `{type, data, timestamp}`；断线自动重连（指数退避，最大 30s）；鉴权失败立即发送 `auth_failed` → 关闭连接。

### Feed 推荐架构（当前已实施）

> 当前状态：ItemCF 协同过滤离线计算待上线，在线服务已实现 MMR 多样性重排序。

#### 召回层

```
四路并发召回:
  1. 热门召回: Redis Sorted Set（时间衰减播放量）→ 冷启动兜底
  2. 内容召回: 同分区 + 同标签匹配
  3. 社交召回: 关注 UP 主的新发布
  4. ItemCF 召回: 用户交互过的视频 → video_similarities 表（离线预计算）→ 规划中
```

#### 重排序（MMR + DPP）

```
候选集 → MMR 重排序:
  Score = Relevance(video, user) - λ × max(Sim(video, already_selected))
  λ = 0.7（多样性强度）
  → 类目打散 + 频控（Redis 曝光计数器）
  → 游标分页（limit ≤ 50, next_cursor）
```

**匿名用户策略:** Redis 缓存热门推荐（TTL 5min），减少重复计算。

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
- 热搜运营: Redis Sorted Set 热词 + `hot_search_ops` 表人工干预
- 搜索历史: Redis List per user，最大 50 条

### 风控引擎

```
用户发表内容 → resolveContentOwner(定位作者)
  → isWhitelisted? → 是 → 放行 ✅
  → isBlacklisted? → 是 → reject + 通知 ❌
  → 加载 enabled 规则(按 priority DESC)
  → 逐条匹配: keyword | regex | rate_limit
  → 命中 → 写 RiskHitLog → 按 Action 分流:
      reject       → 返回 blocked=true
      quarantine   → 评论设 approved=false
      notify_admin → WebSocket 实时推送
      auto_ban     → User.Status="banned" + BanExpiresAt → 自动解封
```

**关键设计:** 白名单优先；正则缓存 per rule；频率窗口 `risk_rate_counters` 表；后台 goroutine 每 60s 清理过期黑名单+自动解封。

### 状态管理策略

**前端状态:** Vuex 4.x 集中式状态管理。
- **服务端状态:** 管理后台数据通过 API 响应直接消费，不做客户端缓存。
- **认证状态:** JWT Token 存储在 `localStorage`，通过 Axios 拦截器自动注入 `Authorization` header。
- **路由状态:** `vue-router` 管理路由，路由守卫检查认证+权限（通过 `GET /admin/rbac/me/permissions` 获取）。
- **UI 状态:** Vuex store 管理侧边栏折叠/展开、管理员信息、权限列表。

**锁定规则:** 所有管理页面通过 `AdminLayout.vue` 统一布局；跨页面共享状态存储在 Vuex store。

---

## 3. 架构决策记录（ADR）

> 核心产出物。每个横切选择是一个 ADR。

### ADR 索引

| ADR | 标题 | 状态 | 驱动 | 日期 |
|-----|------|------|------|------|
| ADR-001 | REST + JSON 信封响应格式 | 已接受 | NFR-4 | 2026-06-25 |
| ADR-002 | MySQL 8.x + GORM AutoMigrate + Redis + RMQ + OSS | 已接受 | NFR-2, NFR-1 | 2026-06-25 |
| ADR-003 | 独立管理员 JWT 双 Token 认证 | 已接受 | NFR-3 | 2026-06-25 |
| ADR-004 | RBAC resource:action 细粒度授权（23 种权限码） | 已接受 | NFR-3 | 2026-06-25 |
| ADR-005 | 模块化单体架构 (Gin) | 已接受 | NFR-1, BC-2 | 2026-06-25 |
| ADR-006 | 全写操作自动审计日志 | 已接受 | NFR-3 | 2026-06-25 |
| ADR-007 | 统一错误码体系（errcode 包，20+ 错误码） | 已接受 | NFR-4 | 2026-06-25 |
| ADR-008 | Feature Flag FNV-1a 灰度策略 | 已接受 | NFR-6 | 2026-06-25 |
| ADR-009 | 审批流多级串行审核 | 已接受 | NFR-3 | 2026-06-25 |
| ADR-015 | SRS + flv.js 直播技术选型 | **已实施** | FR-050 | 2026-06-28 |
| ADR-016 | ItemCF 协同过滤推荐引擎 | 部分实施 | NFR-REC-1/2 | 2026-06-28 |
| ADR-017 | GORM 软删除（Video/Article 等核心实体） | 已接受 | 数据完整性 | 2026-06-28 |

### ADR-001: REST + JSON 信封响应格式

**状态:** 已接受   **驱动:** NFR-4

**Context（背景）:** SPEC NF-4 要求所有 API 使用 RESTful 风格 + 统一 JSON 响应格式。Rule R-API-1 严格定义了 `{code, msg, data}` 格式。

**Decision（决策）:**
- 所有 HTTP API 使用 RESTful 风格（GET 查询/POST 创建/PUT+PATCH 更新/DELETE 删除）
- URL 使用复数名词（`/videos`，非 `/video`），不走动词路径
- 统一 JSON 响应：`{ "code": number, "msg": string, "data": object | null }`
- code 0 = 成功，40001-50099 = 业务/认证/权限/服务器错误
- 实现：`internal/pkg/resp/resp.go` 提供 `OK(c, data)` 和 `Err(c, code)` 工厂函数

**Consequences（后果） — 对所有开发锁定:**
- 所有 handler 必须使用 `resp.OK` / `resp.Err`，严禁裸 `c.JSON`
- 新增错误码必须在 `internal/errcode/errcode.go` 注册
- 无数据时 `data` 字段返回 `null`，非空
- 变得容易: 前端统一拦截器处理错误；API 文档自动生成
- 接受的代价: 简单 GET 请求也需要嵌套结构

**替代方案:**
- **GraphQL:** 学习成本/性能难以控制，团队不熟悉，前端不需要灵活查询
- **gRPC:** 不适合浏览器直调，多一层网关复杂度

**重新审视条件:** 流量 > 5 万并发用户时评估是否需要 GraphQL 聚合查询

---

### ADR-002: MySQL 8.x + GORM AutoMigrate + Redis + RabbitMQ + OSS

**状态:** 已接受   **驱动:** NFR-2

**Context（背景）:** SPEC NF-2 要求 MySQL 主库 + Redis 热数据 + RabbitMQ 异步任务 + OSS 文件存储。Rule R-DB-1/2/3/4 严格定义了数据库安全规范。

**Decision（决策）:**
- **主存储:** MySQL 8.x + GORM v2 AutoMigrate，86 个模型自动建表
- **缓存:** Redis 7.x — 播放量 INCR（10s 落库）、弹幕冷却 SET NX EX（5s）、Token 黑名单、热搜 ZINCRBY、直播观众 SET
- **消息队列:** RabbitMQ 3.x — 视频转码任务（`task_type=transcode`），预留 `subtitle_asr`
- **文件存储:** 阿里云 OSS（`mini-bili` Bucket），目录前缀分区；本地文件系统兜底（Docker 卷 `uploads_data`）
- **ID 策略:** 自增 uint64 主键
- **索引策略:** GORM tag `index` + `uniqueIndex`，核心查询字段（play_count, created_at, user_id, video_id）必须建索引
- **事务:** 多表写操作使用 GORM 事务（如投币：INSERT coin → UPDATE balance → INSERT ledger）

**Consequences（后果） — 对所有开发锁定:**
- 严禁硬编码连接字符串（必须从环境变量读取）
- 严禁拼接 SQL（必须使用 GORM 参数化查询）
- 数据库变更必须通过 AutoMigrate（在 `internal/data/migrate.go` 注册模型）
- 严禁生产环境 `DROP TABLE` 或 `ALTER TABLE` 收缩字段
- 变得容易: 零迁移脚本维护，模型定义即数据库
- 接受的代价: 不支持复杂迁移（如列重命名）；生产缺乏版本控制

**替代方案:**
- **PostgreSQL:** 团队不熟悉，阿里云 MySQL 成本更低
- **MongoDB:** 关系型数据不适用（用户/视频/评论多表关联）
- **golang-migrate:** 1 人团队增加维护负担

**重新审视条件:** 生产部署或多人协作时切换 golang-migrate

---

### ADR-003: 独立管理员 JWT 双 Token 认证

**状态:** 已接受   **驱动:** NFR-3

**Context（背景）:** 用户端和管理端需要完全独立的认证体系。SPEC NF-3 要求双 JWT 体系隔离。

**Decision（决策）:**
- **用户 JWT:** Access Token 2h + Refresh Token **30d**（实际代码，非文档声称的 7d）
- **管理员 JWT:** Access Token 2h + Refresh Token **3d**
- 密码存储: `golang.org/x/crypto/bcrypt`（cost=12）
- 实现: `internal/pkg/jwttoken/` 包，双 Manager 实例
- Refresh Token 刷新时旧 Token 立即加入 Redis 黑名单（TTL 与 Token 有效期间）
- 管理员和用户使用独立的中间件（`AdminJWTAuth` / `JWTAuth`），路由分组隔离

**Consequences（后果） — 对所有开发锁定:**
- 用户端和管理端 API 路由必须分离（`/api/v1/admin/*` vs `/api/v1/*`）
- 严禁 Refresh Token 用于业务 API 访问
- 刷新后必须标记旧 Refresh Token 失效
- 严禁在 Access Token 中存储密码等敏感信息
- 变得容易: 无状态认证，无需 session 存储

**替代方案:**
- **统一 JWT + Role 字段:** 权限边界模糊，攻击面增大
- **Session:** 需要额外存储，不支持水平扩展

**重新审视条件:** OAuth2/OIDC 集成需求出现时重新评估

---

### ADR-004: RBAC resource:action 细粒度授权（23 种权限码）

**状态:** 已接受   **驱动:** NFR-3

**Context（背景）:** 23 个运营模块需要差异化访问控制。SPEC 定义的 23 种权限码（`resource:action` 格式）覆盖所有管理操作。

**Decision（决策）:**
- **模型:** `admin_roles` + `admin_permissions` + `role_permissions`(关联) + `admin_role_assignments` 四表
- **权限格式:** `resource:action`（如 `video:approve`, `user:ban`, `risk:manage`）
- **中间件:** `RequirePermission(db, resource, action) gin.HandlerFunc`
- **23 种权限码:**
  - 📊 数据: `dashboard:view`, `dashboard:export`
  - 📢 运营: `banner:manage`, `hotsearch:manage`, `special:manage`, `dynamic:manage`, `subtitle:manage`
  - 🛡️ 审核: `video:approve`, `article:approve`, `comment:delete`, `ticket:handle`, `copyright:handle`, `risk:manage`
  - 👤 用户: `user:ban`, `cs:manage`
  - 🤖 AI: `agent:manage`, `llm:manage`
  - ⚙️ 系统: `setting:manage`, `config:manage`, `ops:manage`, `rbac:manage`, `live:manage`
- **角色:** `super_admin`（全部权限）, `content_review`（审核组+封禁+只读）, `cs_admin`（客服组+只读）
- 前端侧边栏按 `GET /admin/rbac/me/permissions` 返回权限列表动态过滤

**Consequences（后果） — 对所有开发锁定:**
- 新增管理操作必须在 `rbac_seed.go` 注册权限码
- 路由注册时通过 `admin.Group("", RequirePermission(...))` 分组保护
- 所有写操作自动记录 `audit_logs`（ADR-006）
- 变得容易: 自建 3 表 JOIN 即可满足，无额外依赖

**替代方案:**
- **Casbin:** DSL 学习成本高，23 种简单权限无需引入额外复杂度

**重新审视条件:** 需要 ABAC（基于属性）如"仅工作日可操作"时

---

### ADR-005: 模块化单体架构 (Gin)

**状态:** 已接受   **驱动:** NFR-1, BC-2

**决策:** 单进程 Gin 应用，193 个 Go 源文件按功能拆分目录。handler → service → DB 三层。

**权衡:** 见第 9 节。

---

### ADR-006: 全写操作自动审计日志

**状态:** 已接受   **驱动:** NFR-3

**Decision（决策）:**
- 所有 admin 写操作 handler 调用 `recordAudit(db, adminID, action, resourceType, resourceID, result, c)`
- `audit_logs` 表: `id, admin_id, action, resource_type, resource_id, result, ip, created_at`
- 索引: `(admin_id, created_at)`, `(resource_type, resource_id)`

**锁定规则:** 新增 admin 写操作必须调用 `recordAudit`；审计日志 append-only，不可删除。

---

### ADR-007: 统一错误码体系

**状态:** 已接受   **驱动:** NFR-4

**Decision（决策）:**
- 20+ 错误码映射表（`internal/errcode/errcode.go`）
- 分类: 0=成功, 40001-40099 参数校验, 40100-40199 认证, 40300-40399 权限, 40400-40499 资源, 50000-50099 服务器
- `errmsg.GetMsg(code)` 获取国际化消息

---

### ADR-008: Feature Flag FNV-1a 灰度策略

**状态:** 已接受   **驱动:** NFR-6

**Decision（决策）:**
- `feature_flags` 表: `flag_key, enabled, rollout_pct, whitelist JSON`
- FNV-1a hash 分桶: `hash(user_id) % 100 < rollout_pct` → 灰度命中
- 白名单优先级高于灰度
- 配置发布流程: 模块注册 → Flag 管理 → 版本发布（快照→部署→回滚，draft→deployed→rolled_back）

---

### ADR-016: ItemCF 协同过滤推荐引擎

**状态:** 部分实施（MMR/DPP 重排序已落地，ItemCF 离线计算待上线）

**Context（背景）:** SPEC F17 定义。当前在线服务已有 MMR 多样性重排序和四路召回。

**已实施:**
- 在线服务: `GET /api/v1/feed/recommendation` + 分区推荐 + 订阅源 + 排行榜
- 重排序: MMR 算法（λ=0.7） + 类目打散 + 频控
- 四路召回: 热门 + 内容 + 社交 + ItemCF（规划中）

**待实施:**
- 离线计算: Go 定时任务构建用户-视频交互矩阵（7 种行为加权）
- Cosine 相似度 → `video_similarities` 表
- 冷启动策略: 新用户热门兜底，新视频内容相似度提权×2.0

---

## 4. 组件设计

### 组件总览

```
                     ┌──────────────────────────────────┐
                     │   Vue 3 SPA (cakecake-vue)        │
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
              │    │ 2. Gin Logger (Zap)               │  │
              │    │ 3. Trace (middleware/trace.go)    │  │
              │    │ 4. JWT Auth (Admin / User 双体系) │  │
              │    │ 5. RequirePermission (admin 路由) │  │
              │    │ 6. recordAudit (写操作 handler)   │  │
              │    └───────────────────────────────────┘  │
              └───────┬──────────────┬───────────────────┘
                      │              │
         ┌────────────▼──┐  ┌───────▼──────────┐
         │ Admin Handlers│  │  User Handlers   │
         │ (25 files)    │  │  (58 files)      │
         │  ┌──────────┐ │  │  auth/video/      │
         │  │Service   │ │  │  comment/social/  │
         │  │ Layer DI │ │  │  live/search/dm   │
         │  └──────────┘ │  └───┬──────────────┘
         └───┬───────────┘      │
             │  ┌───────────────▼──────────────┐
             │  │   Service Layer              │
             │  │   internal/service/ (19 files)│
             │  │   Services{Video,User,...}    │
             │  └───────────────┬──────────────┘
             │                  │
     ┌───────▼──────┬───────────▼───────┬──────────┐
     │    MySQL 8.x │     Redis 7.x     │ RabbitMQ │
     │ (86 模型,GORM)│   (Cache/冷却/    │ (转码)   │
     │              │    Token黑名单)   │          │
     └───────┬──────┴───────────────────┴──────────┘
             │
     ┌───────▼───────┐     ┌────────────────┐
     │  Aliyun OSS   │     │ Elasticsearch  │
     │  或本地存储    │     │ (全文搜索,可选)│
     └───────────────┘     └────────────────┘
             │
     ┌───────▼───────┐
     │   SRS 5.x     │
     │  (直播推流)   │
     └───────────────┘
```

### 核心组件详述

#### 组件: Authentication

**职责:** 用户/管理员身份验证和 JWT Token 管理
**实现:** `handler/auth.go`, `handler/admin_auth.go`, `internal/pkg/jwttoken/`
**提供的接口:** `POST /auth/login`, `POST /admin/auth/login`, `POST /auth/refresh`, `POST /admin/auth/refresh`
**需要的接口:** `gorm.DB`（User/Admin 表）, `bcrypt`, Redis（黑名单）
**拥有的数据:** `User`, `Admin`
**约束它的 ADR:** ADR-003, ADR-002
**处理的 NFR:** NFR-3（鉴权）

#### 组件: RBAC Authorization

**职责:** 基于 resource:action 的权限拦截 + 角色/权限/管理员管理
**实现:** `middleware/rbac_permission.go`, `handler/admin_rbac.go`, `data/rbac_seed.go`
**提供的接口:** `RequirePermission(db, resource, action) gin.HandlerFunc`, `/admin/rbac/*`（19 个端点）
**需要的接口:** `gorm.DB`, JWT 中间件
**拥有的数据:** `AdminRole`, `AdminPermission`, `RolePermission`, `AdminRoleAssignment`, `AdminLoginLog`
**约束它的 ADR:** ADR-004, ADR-006, ADR-009
**处理的 NFR:** NFR-3（鉴权 + 审计）

#### 组件: Video Pipeline

**职责:** 视频上传 → 转码 → 发布全链路
**实现:** `handler/video.go`, `handler/video_oss.go`, `worker/transcode.go`, `service/video_service.go`
**提供的接口:** `POST /videos`（上传）, `POST /videos/:id/publish`（发布）, `PUT /videos/:id`（更新）
**需要的接口:** RabbitMQ, FFmpeg, OSS/本地文件存储, Redis（播放计数）
**拥有的数据:** `Video`, `VideoChapter`, `VideoBitrate`
**约束它的 ADR:** ADR-001, ADR-002
**处理的 NFR:** NFR-1（异步转码解耦）

#### 组件: Danmaku Engine

**职责:** 实时弹幕发送 + WebSocket 广播 + Canvas 渲染
**实现:** `handler/danmaku.go`, `handler/ws.go`, `internal/ws/`
**提供的接口:** `POST /videos/:id/danmaku`（发送）, `GET /ws/danmaku`（WebSocket）
**需要的接口:** Redis（5s 冷却 SET NX EX + 200 条历史）, 敏感词过滤器, gorilla/websocket
**拥有的数据:** `Danmaku`, `DanmakuLike`
**约束它的 ADR:** ADR-001
**处理的 NFR:** NFR-1（100 在线 ≤200ms）

#### 组件: Comment System

**职责:** 视频/文章/动态三套独立评论表 + 3 级嵌套 + UP 主管理
**实现:** `handler/comment.go`, `handler/article_comment.go`, `handler/dynamic_comment.go`, `service/comment_service.go`
**提供的接口:** `GET /videos/:id/comments`, `POST /videos/:id/comments`, `DELETE /comments/:id`
**需要的接口:** `gorm.DB`, WebSocket Hub（评论删除广播）, 风控引擎（敏感词/隔离）
**拥有的数据:** `Comment`, `ArticleComment`, `DynamicComment`, `CommentLike`, `CommentDislike`, `CommentImage`
**约束它的 ADR:** ADR-002, ADR-004
**处理的 NFR:** NFR-3（权限校验：UP 主/本人/管理员）

#### 组件: Social System

**职责:** 关注/拉黑/私信/动态/收藏/投币
**实现:** `handler/user_follow.go`, `handler/dm.go`, `handler/user_dynamic.go`, `handler/favorite_folder.go`, `handler/coin_ledger.go`
**提供的接口:** 50+ 端点覆盖全部社交操作
**拥有的数据:** `UserFollow`, `UserBlock`, `FavoriteFolder`, `VideoFavorite`, `VideoCoin`, `CoinLedger`, `UserDynamic`, `DmConversation`, `DmMessage`
**约束它的 ADR:** ADR-002, ADR-003

#### 组件: Live Streaming

**职责:** 直播间创建/管理 + SRS 推流 + flv.js 播放 + WebSocket 聊天+礼物
**实现:** `handler/live.go`, `handler/admin_live.go`, `handler/ws.go`（直播 WS 通道）
**提供的接口:** `/live/rooms/*`, `/live/callback/*`, `/admin/live/*`
**需要的接口:** SRS RTMP 服务器, flv.js 前端播放器
**拥有的数据:** `LiveRoom`, `LiveWarnTemplate`
**约束它的 ADR:** ADR-015
**处理的 NFR:** NFR-LIVE-1/2（已达标）

#### 组件: Risk Engine

**职责:** 多层级风控规则匹配（关键词/正则/频率限制）+ 黑白名单
**实现:** `handler/admin_risk.go`, `handler/sensitive_ugc.go`
**提供的接口:** `/admin/risk/*`（10 个端点）
**拥有的数据:** `RiskRule`, `BlackWhiteList`, `RiskHitLog`, `RiskRateCounter`
**约束它的 ADR:** ADR-004, ADR-006

#### 组件: BI Reports

**职责:** 数据仪表盘 + 多维度统计 + CSV 导出
**实现:** `handler/admin_bi.go`, `handler/admin_dashboard.go`
**提供的接口:** `/admin/bi/*`（10 个端点）, `/admin/dashboard`
**拥有的数据:** `SavedReport`, `VideoDailyStat`
**约束它的 ADR:** ADR-004

#### 组件: Ops Monitoring（5合1）

**职责:** 任务队列/告警/链路追踪/健康检查/CDN 刷新/OSS 生命周期
**实现:** `handler/admin_ops.go`, `middleware/trace.go`
**提供的接口:** `/admin/ops/*`（20+ 端点）
**拥有的数据:** `TaskLog`, `AlertRule`, `AlertRecord`, `TraceRecord`, `CDNRefreshTask`, `OSSLifecycleRule`
**约束它的 ADR:** ADR-004

#### 组件: Config Management

**职责:** Feature Flag 灰度发布 + 模块注册 + 版本发布/快照/回滚
**实现:** `handler/admin_config.go`
**提供的接口:** `/admin/config/*`（11 个端点）
**拥有的数据:** `FeatureFlag`, `ReleaseRecord`
**约束它的 ADR:** ADR-008

### 前端共享组件体系（SPEC F16）

```
src/
├── components/admin/
│   ├── AdminDataTable.vue    ← 统一搜索+表格+分页（已接入 9 个 admin 页面）
│   └── AdminFormDialog.vue   ← 统一新增/编辑弹窗
├── utils/
│   └── admin-helpers.js      ← 共享 formatTime() 等工具函数
└── api/admin/                ← 17 模块模块化 API
    ├── auth.js, banner.js, video.js, comment.js,
    ├── user.js, rbac.js, cs.js, ticket.js, copyright.js,
    ├── risk.js, bi.js, ops.js, config.js, special.js,
    ├── subtitle.js, dashboard.js, dynamic.js, article.js
```

**锁定规则:** 所有新增 admin 页面通过 `@/api/admin` barrel 导入 API；列表页优先使用 `AdminDataTable`；表单弹窗优先使用 `AdminFormDialog`。

---

## 5. 数据模型

> 受 ADR-002 约束。86 个 GORM 模型，15 个业务模块。

### 核心实体（6 张）

| 实体 | 表名 | 字段数 | 关键属性 |
|------|------|--------|---------|
| `User` | `users` | 23+ | id, username, password(bcrypt), nickname, avatar_url, bio, status, coin_balance_tenths, level, cake_id, first_published_at |
| `Video` | `videos` | 29+ | id, user_id, title, description, cover_url, duration, status(processing/published/failed/pending_review/rejected), play_count, zone_id, deleted_at(gorm.DeletedAt) |
| `Article` | `articles` | 20+ | id, user_id, title, content(markdown), cover_url, category, view_count, status, tags JSON, comments_closed, comments_curated |
| `Danmaku` | `danmakus` | 10+ | id, user_id, video_id, content, position_sec, color, type, mode, font_size, like_count |
| `Comment` | `comments` | 12+ | id, user_id, video_id, content, parent_id, root_id, level, like_count, is_pinned, is_featured, approved, curated_ignored |
| `Admin` | `admins` | 8 | id, username, password_hash(bcrypt), display_name, status(active/disabled) |

### 视频互动（10 张）
`danmakus`, `comments`, `comment_likes`, `comment_dislikes`, `video_likes`, `video_coins`, `video_favorites`, `favorite_folders`, `watch_laters`, `danmaku_likes`

### 文章互动（6 张）
`articles`, `article_comments`, `article_favorites`, `article_coins`, `a_comment_likes`, `a_comment_dislikes`

### 关注社交（4 张）
`user_follows`, `user_blocks`, `user_follow_groups`, `u_follow_group_members`

### 消息通知（5 张）
`dm_conversations`, `dm_messages`, `dm_participants`, `notifications`, `like_notif_mutes`

### 动态系统（5 张）
`user_dynamics`, `user_dynamic_likes`, `dynamic_comments`, `d_comment_likes`, `d_comment_dislikes`

### 直播系统（2 张）
`live_rooms`, `live_warn_templates`

### 历史记录（6 张）
`video_view_histories`, `article_view_histories`, `live_view_histories`, `user_search_histories`, `user_daily_tasks`, `coin_ledgers`

### 运营基础（8 张）
`agent_profiles`, `agent_settings`, `home_banners`, `hot_search_ops`, `hot_search_display_layout`, `llm_configs`, `llm_providers`, `reports`

### 工单风控（7 张）
`tickets`, `ticket_messages`, `ticket_satisfactions`, `risk_rules`, `risk_hit_logs`, `black_white_lists`, `risk_rate_counters`

### 版权管理（2 张）
`copyright_complaints`, `counter_notices`

### 数据报表（2 张）
`saved_reports`, `video_daily_stats`

### 客服后台（3 张）
`cs_templates`, `cs_conversations`, `cs_messages`

### 运维监控（6 张）
`task_logs`, `alert_rules`, `alert_records`, `trace_records`, `cdn_refresh_tasks`, `oss_lifecycle_rules`

### 配置权限（12 张）
`feature_flags`, `release_records`, `admin_roles`, `admin_permissions`, `role_permissions`, `admin_role_assignments`, `admin_login_logs`, `audit_logs`, `approval_flows`, `approval_steps`, `special_pages`, `campaigns`

### 模块扩展（6 张）
`video_chapters`, `video_bitrates`, `subtitles`, `comment_images`, `scheduled_publishes`, `notification_records`

### 存储策略

- **主存储:** MySQL 8.x + GORM AutoMigrate（ADR-002）
- **缓存:** Redis 7.x — 播放量 INCR（10s 落库）/ 弹幕冷却 SET NX EX（5s）/ Token 黑名单 / 热搜 ZINCRBY + 定时 decay / 搜索历史 per-user List / 直播观众 SET
- **文件/对象:** 阿里云 OSS `mini-bili` Bucket（`videos/`/`covers/`/`avatars/`/`live-covers/`）+ 本地文件兜底（Docker 卷 `uploads_data`）
- **消息队列:** RabbitMQ — 视频转码任务队列
- **备份策略:** 当前无自动备份，生产应启 MySQL binlog + OSS 版本控制

### 关键索引设计

| 表 | 索引类型 | 字段 | 作用 |
|----|----------|------|------|
| `users` | UNIQUE | `username` | 登录名全局唯一 |
| `videos` | INDEX | `user_id`, `status`, `play_count`, `created_at` | 多维度查询排序 |
| `video_likes` | UNIQUE | `(user_id, video_id)` | 每用户每视频限赞一次 |
| `video_coins` | UNIQUE | `(user_id, video_id)` | 每用户每视频限投币一次 |
| `video_favorites` | UNIQUE | `(user_id, video_id, folder_id)` | 同视频可放多收藏夹 |
| `user_follows` | UNIQUE | `(follower_id, followee_id)` | 防重复关注 |
| `dm_conversations` | UNIQUE | `(user_low, user_high)` | 两人对话唯一 |
| `audit_logs` | INDEX | `(admin_id, created_at)`, `(resource_type, resource_id)` | 审计追溯 |

---

## 6. API 规范

> 受 ADR-001（REST+JSON）、ADR-003（JWT）、ADR-007（错误码）约束。

**协议:** REST over HTTP
**认证:** Bearer JWT（用户端 + 管理端双体系）
**版本化:** URL 路径 `/api/v1/`
**端点总计:** ~380 条路由（admin ~190 + 用户端 ~140 + 公开 ~46 + WS 3 + 回调 2）

### 6.1 认证端点

| 方法 | 路径 | 认证 | 说明 |
|------|------|:--:|------|
| POST | `/api/v1/auth/login` | 否 | 用户登录 → Access(2h) + Refresh(30d) |
| POST | `/api/v1/auth/refresh` | 否 | 刷新 Token |
| POST | `/api/v1/users` | 否 | 用户注册 |
| POST | `/api/v1/admin/auth/login` | 否 | 管理员登录 → Access(2h) + Refresh(3d) |
| POST | `/api/v1/admin/auth/refresh` | 否 | 管理员刷新 |
| GET | `/api/v1/admin/me` | Admin JWT | 当前管理员信息 |

### 6.2 用户端核心 API

```
GET    /api/v1/videos                    ← 首页/分区/排行（支持 zone/period/sort）
GET    /api/v1/videos/:id                ← 视频详情
POST   /api/v1/videos                    ← 上传视频
POST   /api/v1/videos/:id/danmaku        ← 发送弹幕
GET    /api/v1/videos/:id/comments       ← 评论列表（3级嵌套）
POST   /api/v1/videos/:id/comments       ← 发表评论
POST   /api/v1/comments/:id/like         ← 评论点赞
POST   /api/v1/comments/:id/dislike      ← 评论反对
DELETE /api/v1/comments/:id              ← 删除评论（权限校验）
POST   /api/v1/users/:id/follow          ← 关注/取关
POST   /api/v1/users/:id/block           ← 拉黑
GET    /api/v1/dm/conversations          ← 私信会话列表
POST   /api/v1/dm/conversations/:id/messages ← 发送私信
POST   /api/v1/videos/:id/coin           ← 投币
GET    /api/v1/favorites                 ← 收藏列表
POST   /api/v1/search?q=X&type=video     ← ES 全文搜索
GET    /api/v1/feed/recommendation        ← 个性化推荐
GET    /api/v1/leaderboard?by=play&period=week ← 排行榜
GET    /api/v1/live/rooms                ← 直播广场
POST   /api/v1/live/room/create          ← 创建直播间
```

### 6.3 运营后台核心 API

| 路由组 | 权限 | 端点示例 | 数量 |
|--------|------|---------|:--:|
| 管理员认证 | 无 | `POST /admin/auth/login`, `POST /admin/auth/refresh`, `GET /admin/me` | 3 |
| 数据概览 | 只读 | `GET /admin/dashboard`, `GET /admin/bi/summary`, `GET /admin/bi/*` | 10 |
| 用户管理 | `user.ban` | `GET /admin/users`, `POST /admin/users/:id/ban\|unban\|delete` | 6 |
| 视频审核 | `video.approve` | `GET /admin/videos`, `POST /admin/videos/:id/approve\|reject\|delete` | 7 |
| 专栏审核 | `article.approve` | `GET /admin/articles`, `POST /admin/articles/:id/approve\|reject` | 6 |
| 直播管理 | `live.manage` | `GET /admin/live/rooms`, `POST /admin/live/room/:id/ban\|warn` | 9 |
| 动态管理 | `dynamic.manage` | `GET /admin/dynamics`, `GET /admin/dynamics/unified`(三表UNION) | 4 |
| 评论管理 | `comment.delete` | `GET /admin/comments`, `POST /admin/comments/:id/delete` | 4 |
| 举报处理 | `ticket.handle` | `GET /admin/reports`, `POST /admin/reports/:id/handle` | 5 |
| 工单管理 | `ticket.handle` | `GET /admin/tickets`, `POST /admin/tickets/:id/assign\|close` | 10 |
| 风控管理 | `risk.manage` | `GET /admin/risk/rules`, `POST /admin/risk/rules`, CRUD + toggle | 10 |
| 版权管理 | `copyright.handle` | `GET /admin/copyright/complaints`, accept/reject/takedown | 6 |
| 客服后台 | `cs.manage` | `GET /admin/cs/conversations`, assign/message/close + templates | 9 |
| 运维监控 | `ops.manage` | `GET /admin/ops/tasks\|health\|traces`, CRUD + evaluate/sync | 20 |
| 配置发布 | `config.manage` | `GET /admin/config/feature-flags`, releases CRUD + deploy/rollback | 11 |
| 权限审计 | `rbac.manage` | `GET /admin/rbac/*`, roles/permissions/admins/audit-logs/approval-flows | 19 |
| Banner | `banner.manage` | `GET /admin/home-banners`, CRUD + upload-image | 5 |
| 热搜运营 | `hotsearch.manage` | `GET /admin/hot-search/ops`, CRUD + reorder/boost | 10 |
| AI 角色 | `agent.manage` | `GET /admin/agent-profiles`, CRUD + avatar + settings | 9 |
| 系统设置 | `setting.manage` | `GET /admin/settings`, PUT + LLM config/providers CRUD | 12 |
| 字幕管理 | `subtitle.manage` | `GET /admin/subtitles`, CRUD | 4 |
| 专题活动 | `special.manage` | `GET /admin/specials\|campaigns`, CRUD | 8 |
| 播放器高级 | `video.approve` | `GET /admin/videos/:id/chapters\|bitrates`, CRUD | 6 |

### 6.4 错误响应约定

```json
{ "code": 40300, "msg": "无操作权限: user.ban" }
```

HTTP 状态码：200 成功 / 400 参数错误 / 401 未认证 / 403 无权限 / 404 不存在 / 500 服务器错误。

---

## 7. FR/NFR 覆盖矩阵

> 必需。每项 FR 和每项 NFR 一行。状态 = 已处理 | 部分 | 推迟。

### 功能需求覆盖

| ID | 类型 | 需求 | 组件 | ADR | 状态 |
|----|------|------|------|-----|------|
| FR-001 | FR | 用户认证（注册/登录/JWT双Token） | Authentication | ADR-003 | 已处理 |
| FR-002 | FR | 视频上传+转码 Pipeline | Video Pipeline | ADR-002 | 已处理 |
| FR-003 | FR | 实时弹幕（WebSocket+5s冷却+敏感词） | Danmaku Engine | ADR-001 | 已处理 |
| FR-004 | FR | 三级嵌套评论（视频/文章/动态） | Comment System | ADR-002 | 已处理 |
| FR-005 | FR | 社交互动（关注/拉黑/私信/动态/收藏） | Social System | ADR-002/003 | 已处理 |
| FR-006 | FR | 硬币经济（每日任务+投币+账本） | Social System | ADR-002 | 已处理 |
| FR-007 | FR | ES 全文搜索+热搜+历史 | Search Engine | — | 已处理 |
| FR-008 | FR | Feed 推荐（MMR重排序+四路召回） | Feed Engine | ADR-016 | 已处理 |
| FR-009 | FR | 直播系统（SRS+flv.js+聊天+礼物+审核） | Live Streaming | ADR-015 | 已处理 |
| FR-010 | FR | 运营仪表盘 + BI 报表（9卡片+ECharts图表） | Dashboard/BI | ADR-001/002 | 已处理 |
| FR-011 | FR | 视频/专栏/动态审核 | Content Review | ADR-004/006 | 已处理 |
| FR-012 | FR | 直播审核（警告/封禁） | Live Admin | ADR-004/015 | 已处理 |
| FR-013 | FR | 举报处理+工单系统 | Ticket & Report | ADR-004/006 | 已处理 |
| FR-014 | FR | 风控引擎（keyword/regex/rate_limit+黑白名单） | Risk Engine | ADR-004/006 | 已处理 |
| FR-015 | FR | 版权投诉+反通知 | Copyright | ADR-004/006 | 已处理 |
| FR-016 | FR | 客服后台（会话+模板+快捷回复） | Customer Service | ADR-004/006 | 已处理 |
| FR-017 | FR | 运维监控5合1（队列/告警/追踪/健康/CDN） | Ops Monitoring | ADR-004 | 已处理 |
| FR-018 | FR | Feature Flag 灰度发布+模块注册+版本发布 | Config Management | ADR-008 | 已处理 |
| FR-019 | FR | RBAC 23权限码+审计+审批流+登录日志 | RBAC Management | ADR-004/006/009 | 已处理 |
| FR-020 | FR | 用户管理（列表/封禁/信息编辑） | User Management | ADR-002 | 已处理 |
| FR-021 | FR | 评论管理（跨3表联合+待审隔离） | Comment Management | ADR-002 | 已处理 |
| FR-022 | FR | Banner/热搜/专题运营 | Content Ops | ADR-002/004 | 已处理 |
| FR-023 | FR | AI 角色+LLM 配置管理 | Agent Management | ADR-004 | 已处理 |
| FR-024 | FR | 播放器高级（章节+多码率） | Player Advanced | ADR-002 | 已处理 |
| FR-025 | FR | 字幕管理（CRUD+VTT/SRT） | Subtitle Management | ADR-002/004 | 已处理 |
| FR-026 | FR | 评论增强（图片评论+举报+排序配置） | Comment Enhancement | ADR-002 | 已处理 |
| FR-027 | FR | 创作者中心（统计API+章节管理API） | Creator Center | ADR-002 | 已处理 |
| FR-028 | FR | 动态管理（三表UNION统一视图） | Dynamic Management | ADR-004 | 已处理 |

### 非功能需求覆盖

| ID | 类型 | 需求 | 组件 | ADR | 状态 |
|----|------|------|------|-----|------|
| NFR-1 | 性能 | 弹幕100在线≤200ms; 管理~50并发 | Danmaku/Arch | ADR-005 | 已处理 |
| NFR-2 | 存储 | MySQL+Redis+RabbitMQ+OSS | Data Layer | ADR-002 | 已处理 |
| NFR-3 | 鉴权 | 双JWT+RBAC 23权限码+审计 | Auth/RBAC | ADR-003/004/006 | 已处理 |
| NFR-4 | API | RESTful+JSON信封+错误码 | All Handlers | ADR-001/007 | 已处理 |
| NFR-5 | 前端 | Vue3+Vite SPA+AdminLayout | Frontend | — | 已处理 |
| NFR-6 | 配置 | .env updateEnvKeys()+Feature Flag灰度 | Config | ADR-008 | 已处理 |
| NFR-7 | 测试 | go build ./... 编译验证 | CI | — | 已处理 |
| NFR-SEC-2 | 安全 | bcrypt密码(cost=12) | Auth | ADR-003 | 已处理 |
| NFR-SEC-3 | 安全 | 输入验证(GORM tag+Gin binding) | All Handlers | ADR-001 | 已处理 |
| NFR-SEC-4 | 安全 | API 限流 | Middleware | — | 推迟 |
| NFR-OBS-1 | 可观测 | Zap 结构化日志 | Logger | — | 已处理 |
| NFR-OBS-2 | 可观测 | 链路追踪（trace_id贯穿） | Trace Middleware | — | 已处理 |
| NFR-OBS-3 | 可观测 | 系统指标采集+告警评估 | Ops Monitoring | — | 已处理 |
| NFR-REL-1 | 可靠性 | MySQL 每日备份 | 运维脚本 | ADR-002 | 待处理 |
| NFR-REL-2 | 可靠性 | 灾难恢复（RPO=24h, RTO=4h） | 运维流程 | — | 待处理 |
| NFR-REL-3 | 可靠性 | 健康检查端点 | Ops (GET /ops/health) | — | 已处理 |
| NFR-AVAIL-1 | 可用性 | 99% 正常运行时间 | 部署架构 | — | 已处理 |
| NFR-AVAIL-3 | 可用性 | 监控告警（CPU/内存/磁盘/错误率） | Ops | — | 已处理 |
| NFR-DI-1 | 数据完整 | 多表写事务 | Service Layer | ADR-002 | 已处理 |
| NFR-DI-2 | 数据完整 | 外键约束+复合唯一索引 | GORM Model | ADR-002 | 已处理 |
| NFR-DI-3 | 数据完整 | 强一致性（单MySQL实例） | Data Layer | ADR-002 | 已处理 |
| NFR-COMP-2 | 合规 | 审计日志不可篡改（append-only） | AuditLog | ADR-006 | 已处理 |
| NFR-COST-1 | 成本 | 月基础设施 ≤500 CNY | 部署架构 | — | 已处理 |
| NFR-COST-2 | 成本 | OSS 生命周期30天自动清理 | OSSLifecycleRule | — | 已处理 |
| NFR-MAINT-1 | 可维护 | go build ./... 零错误 | CI | — | 已处理 |
| NFR-MAINT-2 | 可维护 | go fmt + ESLint 代码风格 | 开发流程 | — | 已处理 |
| NFR-UA-1 | 可用性 | PC端浏览器兼容（Chrome/Firefox/Edge） | Vue3 SPA | — | 已处理 |
| NFR-UA-2 | 可用性 | 中文界面全覆盖 | 前端 | — | 已处理 |
| NFR-LIVE-1 | 性能 | 直播 FLV 延迟 ≤3s | Live Streaming | ADR-015 | 已处理 |
| NFR-LIVE-2 | 性能 | 单直播间 WebSocket 多观众并发 | Live WS | ADR-015 | 已处理 |
| NFR-REC-1 | 性能 | 推荐接口延迟 ≤50ms（Redis缓存） | Feed Engine | ADR-016 | 已处理 |
| NFR-REC-2 | 离线 | ItemCF 离线相似度计算 | Feed Engine | ADR-016 | 待处理 |

### 覆盖缺口

| ID | 需求 | 缺口 | 状态 |
|----|------|------|------|
| FR-032 | ASR 自动转写 | Worker 预留 `subtitle_asr` 但未实现 Whisper 集成 | 待实施 |
| FR-031 | 字幕编辑器前端 | 后端就绪，用户端缺字幕时间轴编辑器 UI | 待实施 |
| FR-035 | 创作者数据中心 | API 就绪（creator_center.go），缺前端独立数据中心页面 | 待实施 |
| NFR-REL-1 | 每日备份 | 无自动备份脚本 | 待处理 |
| NFR-REL-2 | 灾难恢复 | 无 DR 方案文档 | 待处理 |
| NFR-SEC-4 | API 限流 | 未实现 Token Bucket | 推迟 |
| NFR-REC-2 | ItemCF 离线计算 | 离线矩阵构建+相似度计算未上线 | 待处理 |

---

## 8. 技术栈

> 每项选择附带理由。不用"因为它流行"。

| 层级 | 选择 | 版本 | 理由（→ 驱动因素） | ADR |
|------|------|------|-------------------|-----|
| 前端框架 | Vue 3 + Vite | 3.5+ / 5.x | SPEC NF-5 约束；纯 SPA 无需 SSR；中文社区成熟 | — |
| 前端状态 | Vuex | 4.x | 管理后台状态集中管理；配合 `vue-router` 路由守卫 | — |
| 前端 UI | Element Plus | 2.x | 中文社区成熟，管理后台组件库丰富 | — |
| 前端图表 | ECharts | 5.x | BI 报表柱状/饼图/折线面积/多系列图 | — |
| 后端语言 | Go | 1.24+ | SPEC 约束；高性能并发；标准项目布局 | — |
| 后端框架 | Gin | 1.10+ | 高性能 HTTP 路由；中间件链式组合；社区成熟 | ADR-005 |
| ORM | GORM v2 | 2.x | AutoMigrate 消除 SQL 管理；预加载处理关联查询 | ADR-002 |
| 数据库 | MySQL | 8.x | 关系型数据（86 模型多表关联）；阿里云 RDS 集成 | ADR-002 |
| 缓存 | Redis | 7.x | 播放量 INCR；弹幕冷却；Token 黑名单；热搜 ZSET | ADR-002 |
| 消息队列 | RabbitMQ | 3.x | 视频转码异步解耦；死信队列 | ADR-002 |
| 文件存储 | 阿里云 OSS + 本地 | — | SPEC 约束；本地文件 Docker 卷兜底 | ADR-002 |
| 认证 | JWT (golang-jwt) | 5.x | 无状态认证；双 Token 轮换；独立管理员体系 | ADR-003 |
| 密码哈希 | bcrypt | — | SPEC 约束（R-AUTH-2）；cost=12 | — |
| 日志 | Zap | 1.x | 结构化高性能日志；JSON 格式 | — |
| 实时通信 | gorilla/websocket | 1.5+ | 三套独立 WS 通道 | — |
| 直播流媒体 | SRS + flv.js | 5.x / 1.6+ | RTMP 推流→HTTP-FLV 低延迟播放 | ADR-015 |
| 视频处理 | FFmpeg | 7.0+ | H.264 MP4 转码 + 封面截帧 | — |
| 搜索引擎 | Elasticsearch | 8.x（可选） | ik 中文分词全文搜索 | — |
| Markdown | bluemonday + goldmark | — | 文章安全渲染 | — |
| IP 定位 | ip2region | — | IP 归属地查询 | — |

**考虑的替代方案:**
- **PostgreSQL:** 团队不熟悉，阿里云 MySQL 生态更成熟
- **Pinia（替代 Vuex）:** Vuex 4 当前版本足够，切换成本不值得
- **gRPC:** 不适合浏览器直调，多一层网关复杂度
- **ZLMediaKit（替代 SRS）:** SRS 社区更大，文档更完善

---

## 9. 权衡分析

### 权衡: 模块化单体 vs 微服务

**决策:** 模块化单体（ADR-005）

| 维度 | 模块化单体 | Kratos 微服务 |
|------|-----------|--------------|
| 部署复杂度 | 1 个二进制 | 10+ 个服务 + 注册中心 + 网关 |
| 调试效率 | 单步调试 | 分布式追踪 |
| 扩缩容 | 整体扩容 | 按模块独立扩容 |
| 团队适配 | 1 人开发 | 3+ 人团队 |

**理由:** 1 人团队维护微服务不可行；当前并发（<50 管理员 + 普通用户量）无需独立扩缩容；文件级拆分已为未来过渡预留路径。

**接受:** 收益: 部署简单、开发效率高
**代价:** 无法独立扩缩容、任何模块故障影响整体
**缓解:** 严格 handler 间不调用规则；模块边界清晰；未来按 handler 直接提取为独立服务

**重新审视条件:** 团队 > 3 人或并发 > 500

### 权衡: 自建 RBAC vs Casbin

**决策:** 自建 RBAC（ADR-004）

**理由:** 23 种权限，模式简单（resource:action），自建 4 表 JOIN 即可满足。Casbin 引入 DSL 学习成本和额外依赖。

**重新审视条件:** 需要 ABAC 时评估 Casbin 迁移

### 权衡: GORM AutoMigrate vs 数据库迁移工具

**决策:** GORM AutoMigrate（ADR-002）

**理由:** 1 人开发无需 DBA 审批流程；AutoMigrate 消除版本管理负担。

**重新审视条件:** 生产环境或多人协作时切换 golang-migrate

---

## 10. 部署架构

### 环境

- **开发:** Windows 本地 — `go build` + `npm run dev`
- **容器化:** `docker-compose.yml`（MySQL + Redis + RabbitMQ + SRS + Go后端 + Nginx前端，6 服务）
- **生产:** 单台 Linux 服务器（阿里云 ECS）— systemd + Nginx

### 拓扑

```
                 Internet
                    │
              ┌─────▼─────┐
              │  Nginx    │ ← 反向代理 + 静态文件 (Vue dist/)
              │  :80      │    + API 反代 (:8080)
              └─────┬─────┘    + WebSocket 代理
                    │
        ┌───────────┼───────────┬──────────┬──────────┐
        ▼           ▼           ▼          ▼          ▼
   ┌─────────┐ ┌───────┐ ┌──────────┐ ┌──────┐ ┌──────────┐
   │ Gin App │ │ Redis │ │ RabbitMQ │ │ SRS  │ │ FFmpeg   │
   │ :8080   │ │ :6379 │ │ :5672    │ │ :1935│ │ (内置)   │
   └────┬────┘ └───────┘ └──────────┘ │ :8000│ └──────────┘
        │                             └──────┘
   ┌────▼────┐
   │  MySQL  │
   │  :3306  │
   └─────────┘
        │
   ┌────▼────────┐
   │ Aliyun OSS  │ (或本地 uploads/)
   └─────────────┘
```

### 策略

- **部署方式:** Docker Compose 一键部署（6 服务）；前端 Nginx 静态文件 + API 反代 + WS 代理
- **回滚:** 保留上一版本二进制；`systemctl restart` 即可回滚
- **扩缩容:** 当前单实例，未来水平扩展需引入 Redis session 共享 + MySQL 读写分离
- **文件存储:** 无需云存储即可运行（本地文件系统 Docker 卷）；配置 `OSS_*` 环境变量后自动切换

---

## 11. 未来考虑

### 扩展路径

```
当前容量: ~50 并发管理员，单实例，单码率，86 模型
  │
  ├── P0（当前）: 播放器增强 + 多码率 + 字幕完善 + 合集 + 创作者数据中心
  │   → 播放体验对标 B站"能看"
  │
  ├── P1: 推荐引擎 + 标签话题 + 认证 + 移动端 + 高级弹幕 + 水印
  │   → 社区生态对标 B站"好用"
  │   → 并发目标: 500 用户，需水平扩展
  │
  ├── P2: 大会员 + 充电 + 激励 + 开放平台 + AIGC
  │   → 商业化对标 B站"能赚"
  │
  └── P3: 电商 + 课堂 + 游戏 + 音频
      → 完整平台对标 B站
```

### 重新审视触发条件（汇总自 ADR）

| 触发条件 | 应重新评估的决策 |
|---------|----------------|
| 管理并发 > 200 | ADR-004: RBAC 权限缓存到 Redis |
| 团队规模 > 3 人 | ADR-005: 启动微服务拆分 |
| 每日审计日志 > 10 万条 | ADR-006: 审计日志异步写入 + 分表 |
| 流量 > 5 万并发用户 | ADR-001: 评估 GraphQL 聚合查询 |
| 需要 ABAC 访问控制 | ADR-004: 评估 Casbin 迁移 |
| 推荐系统上线 | ADR-016: ItemCF 离线任务每日凌晨重算；相似度阈值 0.15 |
| 视频量 > 10 万 | ADR-016: ItemCF 矩阵过大 → 升级 Embedding 召回 |
| 移动端流量 > 20% | 评估独立移动端 SPA 或 PWA |
| 生产环境部署 | ADR-002: 切换 golang-migrate |

---

## 附录

### 术语表

| 术语 | 定义 |
|------|------|
| Cakecake | 用户端品牌名，项目仓库名 |
| Mini-Bili / minibili | 后端 Go 模块名 |
| 模块化单体 | 单进程部署，handler 文件级模块拆分，为未来微服务预留边界的架构模式 |
| RBAC | 基于角色的访问控制 (Role-Based Access Control)，resource:action 细粒度 |
| ADR | 架构决策记录 (Architecture Decision Record) |
| Feature Flag | 功能开关，FNV-1a hash 分桶 + 白名单 + rollout_pct 三层灰度 |
| MMR | 最大边际相关性重排序算法，控制推荐多样性 |
| ItemCF | 基于物品的协同过滤推荐算法 |

### 参考代码（从代码逆向提取，非推测）

- **Go 源文件:** 193 个（`internal/` 目录）
- **GORM 模型:** 86 个（`internal/data/migrate.go` AutoMigrate 列表）
- **RBAC 权限码:** 23 种（`internal/data/rbac_seed.go`）
- **路由注册:** ~380 条（`internal/handler/router.go`）
- **Service 文件:** 19 个（`internal/service/`）
- **Handler 文件:** 83 个（`internal/handler/`，含 25 个 admin handler + 58 个用户端）

### 参考资料

- SPEC: `SPEC.md`（v2.1）
- Rule: `Rule.md`
- Skill: `Skill.md`
- 审计报告: `docs/architecture-audit-2026-06-28.md`
- 工作记忆: `.workbuddy/memory/MEMORY.md`
- README: `README.md`

### 文档历史

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-06-25 | Winston | 初始架构（从现有代码逆向提取） |
| 1.2 | 2026-06-28 | Winston | P0 修复：直播纳入范围、数据模型补全 |
| 2.0 | 2026-06-28 | Winston | 重评估：Service层+WS+ES+组件补全，NFR全面覆盖 |
| 3.0 | 2026-06-30 | Winston | 审计修正：RefreshToken时长(30d/3d)、RBAC权限(23→23保持一致)、API端点(~190 admin)、模型数(86)、ADR-016状态更新；新增 BI summary/engagement-stats、Dynamic统一端点、LLMProvider；补充风控引擎详述、MMR重排序 |


---

**文档结束** — 验证通过后即可进入下一阶段（Epic 拆分与 Sprint 规划）。
