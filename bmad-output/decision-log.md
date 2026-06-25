# 决策日志 — Mini-Bili 运营中心

> 每个重大决策一条 ADR。绝不删除记录——用废弃标记。
> 状态：提议 / 已接受 / 被 ADR-{NNN} 废弃

---

| ADR | 标题 | 状态 | 驱动 |
|-----|------|------|------|
| ADR-001 | REST + JSON 信封响应格式 | 已接受 | NFR-PERF-3, FR-001~025 |
| ADR-002 | MySQL + GORM AutoMigrate 持久化 | 已接受 | NFR-DATA-1, FR-001~025 |
| ADR-003 | 独立管理员 JWT 双 Token 认证 | 已接受 | NFR-AUTH-1/2 |
| ADR-004 | RBAC resource:action 细粒度授权 | 已接受 | NFR-AUTH-3 |
| ADR-005 | 模块化单体架构 (Gin) | 已接受 | NFR-PERF-1, NFR-EXT-1 |
| ADR-006 | 全写操作自动审计日志 | 已接受 | NFR-AUTH-4 |
| ADR-007 | 统一错误码体系 | 已接受 | NFR-PERF-3 |
| ADR-008 | Feature Flag FNV-1a 灰度策略 | 已接受 | NFR-CONFIG-3 |
| ADR-009 | 审批流多级审核 | 已接受 | NFR-AUTH-5 |

---

## ADR-001: 所有 HTTP API 使用 REST + JSON 信封响应格式

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-PERF-3

### Context

运营中心需要为 25 个模块提供统一的 API 风格，前端团队（Vue3 SPA）需要一致的响应格式来处理成功和错误状态。所有管理端接口使用 `/api/v1/admin` 路由前缀，公共接口使用 `/api/v1` 前缀。

### Decision

- **协议**: REST over HTTP，不使用 GraphQL 或 gRPC。
- **响应信封**: 所有响应遵循 `{code: int, msg: string, data: any}` 格式。
- **错误码**: 使用 `internal/errcode` 包定义的统一错误码（如 `40006` = 用户名存在，`40301` = 密码错误，`50000` = OSS 未配置）。
- **HTTP 状态码映射**: 401 = 未认证，403 = 无权限，400 = 参数错误，404 = 资源不存在，500 = 服务器错误。
- **路由约定**: 资源名使用复数小写（`/users`、`/videos`、`/comments`），操作通过 HTTP 方法区分（GET=查询, POST=创建, PUT=全量更新, PATCH=部分更新, DELETE=删除）。
- **版本化策略**: URL 路径版本化 `/api/v1/`，不做 Header 版本化。
- **管理端前缀**: 所有管理接口统一 `/api/v1/admin/` 前缀。

### Consequences

- 变得容易: 前端统一 `axios` 拦截器处理错误码；API 文档生成工具可直接解析
- 接受的代价: REST 的 over-fetching/under-fetching 问题；复杂聚合查询（如仪表盘）需要后端专门设计
- 锁定规则: 所有 API handler 必须使用 `resp.Ok(c, data)` / `resp.Err(c, httpStatus, errCode)` 返回

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| GraphQL | 管理后台无需灵活查询能力，增加学习成本和复杂度 |
| gRPC | 浏览器原生不支持，需要 gRPC-Web 代理层，不符合 Vue SPA 直接调用需求 |

### 重新审视条件

当前端需要实时推送能力（如运营看板自动刷新），评估是否引入 WebSocket 子协议。

---

## ADR-002: 持久化使用 MySQL + GORM AutoMigrate，缓存使用 Redis

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-DATA-1/2/3/4

### Context

运营中心需要持久化用户数据、视频元数据、风控规则、审计日志、Feature Flag 配置等。同时需要 Redis 缓存热数据和 RabbitMQ 处理异步任务。

### Decision

- **主存储**: MySQL 8.x，通过 GORM v2 ORM 操作。
- **Schema 管理**: GORM `AutoMigrate` 自动建表，无需手动 SQL。所有模型定义在 `internal/model/` 目录下。
- **表命名**: 使用 snake_case 复数表名（`video_chapters`、`risk_rules`、`audit_logs`）。
- **主键**: 使用自增 `uint64`（`gorm:"primaryKey"`），不使用 UUID。
- **时间戳**: 标准 `created_at` / `updated_at`，类型 `time.Time`。
- **索引**: 通过 GORM tag `gorm:"index:idx_name"` 声明。
- **缓存**: Redis 存储播放量热数据（`INCR` 命令），每 10 秒刷新到 MySQL；弹幕实时通道中转；Token 黑名单。
- **消息队列**: RabbitMQ 用于视频转码等耗时异步任务。
- **文件存储**: 阿里云 OSS `mini-bili` Bucket，路径规则 `{type}/{id}.{ext}`。

### Consequences

- 变得容易: GORM AutoMigrate 消除数据库版本管理负担；Redis 大幅降低播放量计数器对 MySQL 的写压力
- 接受的代价: GORM 在复杂 JOIN 查询时可能产生 N+1 问题（评论管理跨 3 表联合查询需手动优化）；自增 ID 不利于分布式扩展
- 缓解: 评论管理使用预加载 (`Preload`) 处理关联查询；后续微服务拆分时可引入分布式 ID 生成器
- 锁定规则: 所有模型必须定义 `TableName()` 方法；禁止在 handler 中直接写 SQL 字符串，必须通过 GORM 链式调用

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| PostgreSQL | MySQL 更符合团队现有经验；阿里云 RDS MySQL 集成成熟 |
| MongoDB | 运营数据以关系型为主（用户-视频-评论-工单多表关联），文档型反范式化增加维护成本 |
| UUID 主键 | 自增 ID 性能更优，当前单体架构无分布式需求 |

### 重新审视条件

微服务拆分时重新评估主键策略（分布式 ID 如 Snowflake）。

---

## ADR-003: 管理员端使用独立 JWT 双 Token 认证体系

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-AUTH-1, NFR-AUTH-2

### Context

管理员认证需要与用户端认证完全隔离，防止用户 Token 越权访问管理接口。同时需要 refresh token 机制支持长会话。

### Decision

- **独立 Token 体系**: 管理员 JWT 与用户 JWT 使用不同的签名密钥和 Claims 结构。
- **端点**: 登录 `POST /api/v1/admin/auth/login`，刷新 `POST /api/v1/admin/auth/refresh`。
- **Token 类型**: 使用 `jwttoken.Manager` 包的 `ParseAdminAccess()` 方法解析，Claims 中包含 `AdminID`（`uint64`）。
- **中间件链**: `middleware.AdminJWTAuth(jwtm)` → 设置 `c.Set("admin_id", aid)` → `middleware.RequirePermission(db, resource, action)`。
- **Token 黑名单**: 登出/刷新时将旧 Token 加入 Redis 黑名单。
- **前端存储**: JWT 存储在 `localStorage`，请求时通过 `Authorization: Bearer <token>` header 传递。
- **管理员模型**: `Admin` 表（id, username, password_hash(bcrypt), display_name, status, last_login_at）。

### Consequences

- 变得容易: 管理员端和用户端安全边界清晰；可以独立配置 Token 过期时间（管理员通常更长）
- 接受的代价: 需要维护两套 JWT 验证逻辑；admin 注册通过 RBAC 管理接口手动创建，无公开注册
- 锁定规则: 管理员登录只能通过 `/admin/auth/login`；`AdminJWTAuth` 中间件检查 Claims 中的 `admin_id`

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| Session + Cookie | RESTful API 架构下前后端分离部署不便；跨域 Cookie 配置复杂 |
| 用户/管理员共用 JWT | 安全风险：用户 Token 泄漏后可访问管理接口；Claims 结构混杂 |

### 重新审视条件

引入 OAuth2/OIDC 统一认证时重新评估。

---

## ADR-004: RBAC 使用 resource:action 细粒度权限模型

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-AUTH-3

### Context

25 个管理模块需要差异化访问控制。简单的角色枚举（admin/moderator）无法满足如"允许审核视频但禁止封禁用户"、"允许管理 BI 报表但禁止修改风控规则"等需求。

### Decision

- **权限模型**: `resource:action` 格式（如 `video:approve`、`user:ban`、`rbac:manage`）。
- **表结构**: 
  - `admin_roles` — 角色定义（name, description）
  - `admin_permissions` — 权限定义（resource, action）
  - `role_permissions` — 角色-权限多对多关联
  - `admin_role_assignments` — 管理员-角色一对一关联
- **中间件**: `RequirePermission(db, resource, action)` — 通过 JOIN 查询 `admin_role_assignments → role_permissions → admin_permissions` 判断权限。
- **调试头**: 每个请求响应携带 `X-RBAC-Check` header，格式 `admin:{id} resource:{r} action:{a} count:{n}`。
- **前端点**: `GET /api/v1/admin/rbac/me/permissions` — 返回当前管理员的所有权限列表，用于侧边栏显示控制。

### Consequences

- 变得容易: 运营团队可灵活组合权限；新增模块只需在 `admin_permissions` 表插入新记录
- 接受的代价: 每次请求都执行 3 表 JOIN 查询（性能可接受——管理员并发 ≤ 50）
- 锁定规则: 所有管理端写操作路由必须包裹在 `RequirePermission` 中间件组中；读操作可免权限（所有已认证管理员）

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 固定角色枚举 | 无法满足 25 模块差异化权限需求 |
| Casbin | 引入额外依赖和 DSL 学习成本；当前规模下自建 RBAC 足够 |

### 重新审视条件

管理员并发超过 200 时，评估是否缓存权限到 Redis 以减少 DB JOIN。

---

## ADR-005: 使用模块化单体架构，为微服务拆分预留边界

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-PERF-1, NFR-EXT-1

### Context

项目目前由 1 人全栈开发（PandaGuGu），运营中心并发 ≤ 50 管理员。SPEC v1.0 明确要求"代码架构必须支持未来平滑拆分为 Kratos 微服务"（BC-2）。

### Decision

- **当前模式**: Go Gin 单体应用，所有 handler 在同一进程。
- **文件拆分**: 每个模块一个 handler 文件（`admin_dashboard.go`、`admin_risk.go` 等），对应未来微服务的 API 层。
- **共享层**: `internal/model/` — 所有模型；`internal/middleware/` — 认证/授权/日志中间件；`internal/pkg/` — 工具包。
- **路由注册**: `internal/handler/router.go` — 集中注册所有路由，但按权限组拆分路由组。
- **拆包原则**: handler 文件之间不互相调用；通过 `API` 结构体共享 `DB`、`Log`、`Cfg` 依赖；数据模型通过 `model` 包共享。
- **前端**: 每个管理页面为独立 Vue SFC，路由注册在 `router/index.js`。

### Consequences

- 变得容易: 部署简单（单个二进制文件）；开发调试效率高；handler 文件边界清晰
- 接受的代价: 无法独立扩缩容（一个模块高负载影响整体）；代码耦合风险（handler 通过共享 DB 间接耦合）
- 缓解: 严格执行"handler 间不直接调用"规则；所有跨模块数据访问通过 model 层
- 锁定规则: 禁止在 handler A 中调用 handler B 的函数；共享逻辑抽到 `internal/service/` 或 `internal/pkg/`

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 微服务 (Kratos) | 1 人团队维护 10+ 服务的运维负担过重；当前并发无需扩容 |
| 纯单体无拆分规划 | 违反 BC-2 向前兼容要求；未来重构成本极高 |

### 重新审视条件

团队规模 > 3 人或并发管理员 > 500 时，启动微服务拆分评估。

---

## ADR-006: 所有写操作自动记录审计日志

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-AUTH-4

### Context

运营中心涉及封禁用户、删除内容、修改权限等敏感操作，需要完整的操作审计追踪以满足安全合规要求。

### Decision

- **共享函数**: `(a *API) recordAudit(c, adminID, action, resource, targetID, detail)` — 定义在 `admin_ops.go`，所有 admin handler 调用。
- **记录字段**: `admin_id`、`action`（`create_role`、`ban_user`、`delete_video` 等）、`resource`（`role`、`user`、`video` 等）、`target_id`、`detail`（JSON 字符串）、`ip`、`user_agent`、`timestamp`。
- **触发时机**: 每个 POST/PUT/DELETE handler 在业务操作成功后立即调用 `recordAudit`。
- **查询接口**: `GET /api/v1/admin/rbac/audit-logs`（列表 + 详情）、`GET /api/v1/admin/rbac/login-logs`（登录审计）。
- **存储**: `audit_logs` 表，保留周期由 `AdminGetAuditLog` 中的时间范围查询控制。

### Consequences

- 变得容易: 统一审计格式；`recordAudit` 签名简单，handler 调用成本低
- 接受的代价: `audit_logs` 表随操作量线性增长（1 万条/月 ≈ 5MB，可接受）；fire-and-forget 写入可能丢失（未使用事务）
- 缓解: 审计日志写入失败不阻塞主业务（仅记录 error log）；定期归档旧日志
- 锁定规则: 所有 admin handler 的写操作必须在业务成功后在 return 前调用 `recordAudit`

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 中间件自动审计 | 无法获取业务语义（如 target_id、detail JSON），审计日志失去可读性 |
| 消息队列异步写入 | 增加延迟和复杂度；当前写入量无需解耦 |

### 重新审视条件

每日审计日志 > 10 万条时，切换为消息队列异步写入 + 按时序分表。

---

## ADR-007: 统一错误码体系（errcode 包）

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-PERF-3

### Context

前端需要根据后端返回的错误码做差异化处理（如用户名重复显示特定提示、token 过期跳转登录页），不能仅依赖 HTTP 状态码。

### Decision

- **错误码格式**: 5 位数字，`HTTP状态码前缀 + 序号`（如 `40006` = 用户名重复，`40301` = 密码错误）。
- **定义位置**: `internal/errcode/errcode.go` — 集中定义所有错误码常量。
- **响应函数**: `resp.Err(c, http.StatusXXX, errcode.CodeXxx)` — 统一错误响应封装。
- **分类规范**:
  - `40100` = 未认证 / Token 无效
  - `40300` = 通用无权限 / `40301` = 密码错误
  - `40000` = 通用参数错误 / `40006` = 业务参数（如用户名存在）
  - `50000` = 服务器错误（OSS 未配置等）
- **前端处理**: Axios 拦截器根据 `code` 字段做自动 token 刷新和错误提示。

### Consequences

- 变得容易: 前后端通过错误码约定消除沟通歧义
- 接受的代价: 错误码定义需要维护文档；新增业务错误需同时更新 errcode 包 + 前端
- 锁定规则: 禁止在 handler 中直接 `c.JSON(400, gin.H{"msg": "xxx"})`，必须使用 `errcode` 常量

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 仅 HTTP 状态码 | 无法区分同状态码下的不同业务错误 |
| 字符串错误码 | 拼写错误风险，国际化困难 |

### 重新审视条件

国际化需求出现时，将错误码映射到 i18n key。

---

## ADR-008: Feature Flag 使用 FNV-1a Hash + 白名单 + 比例灰度

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-CONFIG-3

### Context

新功能（如新的播放器 UI、字幕 ASR 功能）需要灰度发布，支持对特定用户白名单开启 + 按比例随机分配。

### Decision

- **灰度策略**: 三层递进判断：
  1. 检查用户是否在 `whitelist` 中 → 若在，返回 true
  2. 检查 `rollout_pct`（0-100）→ 若为 0 全部关闭，100 全部开启
  3. FNV-1a hash(user_id) % 100 < rollout_pct → 返回 true/false
- **数据模型**: `FeatureFlag` 表（key, description, enabled, rollout_pct, whitelist JSON）。
- **公开检查端点**: `GET /api/v1/config/feature-flags/:key` — 无需认证，前端按需调用。
- **管理 CRUD**: `GET/POST/PUT /api/v1/admin/config/feature-flags` + `toggle` — 权限 `config.manage`。

### Consequences

- 变得容易: 前端通过一个 API 调用即可判断功能可用性；运营人员通过 UI 调整灰度比例
- 接受的代价: FNV-1a 分桶非严格随机（有微小偏差）
- 锁定规则: 所有新功能引入前必须在 `FeatureFlag` 表注册 key；前端必须检查 Flag 后才渲染新 UI

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| LaunchDarkly / 第三方服务 | 成本和外部依赖；当前规则简单无需 SaaS |
| 仅白名单 | 无法做比例灰度，大型功能无法渐进放量 |

### 重新审视条件

需要多维度 targeting（地区/设备/版本）时评估引入 OpenFeature SDK。

---

## ADR-009: 审批流使用多级串行审核

**状态:** 已接受  
**日期:** 2026-06-25（回溯）  
**决策者:** 架构师 Winston（从代码中提取）  
**驱动:** NFR-AUTH-5

### Context

高风险操作（如删除热门视频、封禁大 V 用户）需要多人审批才能执行，防止单人误操作。

### Decision

- **审批流模型**: `ApprovalFlow`（resource_type, resource_id, current_step, status, created_by）+ `ApprovalStep`（step_order, approver_id, verdict, comment）。
- **状态机**: `pending → approved`（所有步骤通过）或 `pending → rejected`（任一步骤拒绝）。
- **流程**: 创建审批流 → 审批人按步骤顺序审批 → 全部通过后操作执行。
- **API**: `POST /api/v1/admin/rbac/approval-flows` (创建) + `POST approve/reject` (审批) + `GET list` (查看)。
- **权限**: `rbac.manage`。

### Consequences

- 变得容易: 多级审批确保高风险操作有制衡
- 接受的代价: 审批流引入操作延迟；审批人离线时阻塞流程
- 锁定规则: 高风险操作需先创建 ApprovalFlow，在所有步骤 approved 后才能执行实际业务操作

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 并行审批 | 运营场景下串行审批更符合层级审批流程 |
| 仅记录日志 | 缺乏"阻止"能力，事后审计无法防止已发生的误操作 |

### 重新审视条件

审批流平均等待时间 > 24 小时时，引入自动提升（escalation）机制。

---

## 附录 A: RBAC 权限-路由映射表

| resource | action | 路由组 | 覆盖功能 |
|----------|--------|--------|---------|
| `user` | `ban` | `/admin/users/:id/ban` / `unban` / `delete` | FR-004, FR-008 |
| `video` | `approve` | `/admin/videos/:id/approve` / `reject` / `delete` + chapters/bitrates | FR-002, FR-020 |
| `comment` | `delete` | `/admin/comments/:id/delete` + comment-reports | FR-009 |
| `ticket` | `handle` | `/admin/tickets/*` + `/admin/reports/*` | FR-003 |
| `article` | `approve` | `/admin/articles/:id/approve` / `reject` / `delete` | FR-002 |
| `dynamic` | `manage` | `/admin/dynamics/:id/delete` | FR-002 |
| `banner` | `manage` | `/admin/home-banners/*` | FR-010 |
| `hotsearch` | `manage` | `/admin/hot-search/*` | FR-011 |
| `agent` | `manage` | `/admin/agent-*` | FR-012 |
| `setting` | `manage` | `/admin/settings` + `/admin/llm-config` | FR-013 |
| `subtitle` | `manage` | `/admin/subtitles/*` | FR-021 |
| `risk` | `manage` | `/admin/risk/*` | FR-004 |
| `copyright` | `handle` | `/admin/copyright/*` | FR-005 |
| `dashboard` | `export` | `/admin/bi/*` | FR-006 |
| `cs` | `manage` | `/admin/cs/*` | FR-007 |
| `ops` | `manage` | `/admin/ops/*` | FR-014/015/016/018 |
| `config` | `manage` | `/admin/config/*` | FR-017 |
| `rbac` | `manage` | `/admin/rbac/*` | FR-019 |
| `special` | `manage` | `/admin/specials` + `/admin/campaigns` | FR-025 |

---

## 附录 B: 关键代码位置

| 关注点 | 文件 |
|--------|------|
| 路由注册 | `internal/handler/router.go` |
| Admin JWT 中间件 | `internal/middleware/admin_auth.go` |
| RBAC 权限中间件 | `internal/middleware/rbac_permission.go` |
| 审计日志函数 | `internal/handler/admin_ops.go` — `recordAudit()` |
| 错误码定义 | `internal/errcode/errcode.go` |
| 响应封装 | `internal/pkg/resp/resp.go` |
| JWT 管理 | `internal/pkg/jwttoken/` |
| 数据模型 | `internal/model/module_extend.go` + 基础模型 |
| 前端管理布局 | `cakecake-vue/bilibili-vue/src/pages/admin/AdminLayout.vue` |
