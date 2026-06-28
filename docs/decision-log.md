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
| ADR-010 | 多清晰度转码 Pipeline | 提议 | FR-029, NFR-TRANSCODE-1/2 |
| ADR-011 | 推荐系统混合架构 | 提议 | FR-036, NFR-REC-1/2 |
| ADR-012 | 视频水印 FFmpeg overlay | 提议 | FR-042 |
| ADR-013 | 移动端渐进式适配 | 提议 | FR-043, NFR-MOBILE-1/2 |
| ADR-014 | 统一支付网关抽象 | 提议 | FR-045/046/047/052, NFR-PAY-1/2 |
| ADR-015 | 直播技术选型 SRS | 已接受 | FR-050, NFR-LIVE-1/2 |
| ADR-016 | MMR/DPP 多样性重排序 + 用户画像分段 | 已接受 | FR-036, NFR-REC-1/2, 2026-06-28 |
| ADR-017 | GORM 软删除保障视频数据安全 | 已接受 | NFR-DATA-1, 2026-06-28 |

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

---

## ADR-010: 多清晰度转码 Pipeline — FFmpeg 单输入多路输出

**状态:** 提议  
**日期:** 2026-06-27  
**决策者:** 架构师 Winston  
**驱动:** FR-029 (多清晰度转码), FR-030 (清晰度切换 UI), NFR-TRANSCODE-1/2

### Context

当前系统仅转码单一 H.264 码率输出。真实 B站支持 360P~4K 多档清晰度，用户按网络状况切换。需要在不显著增加转码时长的前提下产出多档清晰度。

### Decision

- **转码策略**: FFmpeg 单次输入，多路输出（`-map` 多 output）。一次读取原始文件，同时编码 1080P/720P/480P 三路。
- **码率配置**: 1080P@6Mbps / 720P@3Mbps / 480P@1Mbps（H.264 High Profile, CRF=23）
- **存储路径**: `videos/{video_id}/1080p.mp4` / `720p.mp4` / `480p.mp4`
- **数据模型**: 已有 `VideoBitrate` 表（video_id, bitrate, url, size），复用。
- **Worker 改造**: `worker/transcode.go` 在现有转码完成后，并行编码多路清晰度。
- **清晰度切换**: 前端记录 `currentTime`，切换清晰度后 `seek` 到相同位置继续播放。
- **默认清晰度**: 根据用户网络状况自动选择（`navigator.connection.effectiveType`），默认 720P。

### Consequences

- 变得容易: 无需更换转码方案；`VideoBitrate` 表已有，前后端对齐成本低
- 接受代价: 转码时间增加约 60%（多路编码）；OSS 存储增加约 4x（3档 vs 1档）
- 锁定规则: OSS 路径 `videos/{id}/{quality}.mp4`；VideoBitrate 表的 quality 字段枚举 `1080p/720p/480p`

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 按需转码（Just-in-Time） | 首播延迟不可接受；需要转码集群 |
| H.265/AV1 编码 | 浏览器兼容性差（Safari 不支持 AV1）；编码时间长 3-5x |

### 重新审视条件

H.265 浏览器支持率 > 90% 时重新评估；引入 GPU 加速转码（NVENC）时重评 Pipeline。

---

## ADR-011: 推荐系统使用混合架构 — 协同过滤召回 + CTR 排序

**状态:** 提议  
**日期:** 2026-06-27  
**决策者:** 架构师 Winston  
**驱动:** FR-036 (推荐升级), FR-044 (相关推荐), NFR-REC-1/2

### Context

当前推荐仅为热度规则排序（播放量>时间>弹幕数）。B站核心体验依赖个性化推荐提升用户留存。团队 1 人，不能引入大规模 ML 基础设施。

### Decision

- **架构**: 召回层 + 排序层 + 重排层 三段式
- **召回层**:
  - **协同过滤 (ItemCF)**: 离线计算视频相似度矩阵（Jaccard/余弦），存入 Redis `rec:sim:{video_id}` ZSET
  - **内容召回**: 同标签/同分区/同UP主，MySQL 直接查询
  - **热度兜底**: 全站热门 TOP 200
- **排序层**: 轻量 CTR 预估 — 特征：用户历史互动率 + 视频 CTR + 发布时间衰减 → 加权打分
- **重排层**: 打散（同UP主间隔 ≥ 3）、去重、多样性提升
- **离线计算**: 每日凌晨 Cron 重算 ItemCF 矩阵（Go goroutine）；增量每小时更新
- **在线服务**: `GET /api/v1/feed/recommendation` → Redis 取召回集 → 排序 → 返回 Top 20
- **AB 实验**: Feature Flag `rec_algo_v2`（ADR-008）控制新老算法分流

### Consequences

- 变得容易: 纯 Go 实现无需 Python/Spark 依赖；Redis 天然适合存储相似度矩阵
- 接受代价: ItemCF 冷启动问题（新视频无交互数据 → 依赖内容召回+热度兜底）；无深度语义理解
- 锁定规则: 推荐接口延迟 ≤ 100ms；召回候选集 ≥ 500

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 深度学习（DNN/Transformer） | 需 GPU + 训练 Pipeline + 特征平台，1 人团队不可维护 |
| 纯内容推荐 | 冷启动好但缺乏个性化，CTR 远低于协同过滤 |
| 第三方推荐服务 | 成本 + 数据外泄风险 |

### 重新审视条件

用户量 > 10 万或团队 > 3 人时，引入向量召回（Milvus/Faiss）+ Two-Tower 模型。

---

## ADR-012: 视频水印使用 FFmpeg overlay 转码时叠加

**状态:** 提议  
**日期:** 2026-06-27  
**决策者:** 架构师 Winston  
**驱动:** FR-042

### Context

B站视频在右下角有用户ID水印，防止盗搬。需要在转码阶段叠加，而非前端 CSS 覆盖（前端水印可被 F12 去除）。

### Decision

- **实现方式**: FFmpeg `drawtext` 滤镜，在转码输出时叠加文字水印。
- **水印内容**: `@{username}` + 上传时间戳（半透明白色，右下角，字号为视频高度的 3%）
- **控制开关**: Feature Flag `video_watermark_enabled`（ADR-008），管理员可关闭
- **性能影响**: `drawtext` 滤镜增加约 5% 转码时间，可接受
- **字体**: 使用系统默认无衬线字体（Linux: DejaVu Sans, Windows: Arial）

### Consequences

- 变得容易: FFmpeg 原生支持，无需额外依赖
- 接受代价: 转码时间微增；水印无法在已转码视频上追加（需重新转码）
- 锁定规则: 水印仅加在转码阶段；Feature Flag 控制全局开关

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 前端 CSS 水印 | F12 即可去除，无防盗意义 |
| DRM 加密 | 实现复杂度极高，需 Widevine/FairPlay 许可 |

---

## ADR-013: 移动端渐进式适配 — CSS 响应式优先

**状态:** 提议  
**日期:** 2026-06-27  
**决策者:** 架构师 Winston  
**驱动:** FR-043, NFR-MOBILE-1/2

### Context

当前 Vue SPA 以 PC 端（1920px）设计为主，未做移动端适配。B站移动端流量占比 > 70%。1 人团队无法同时维护两套前端。

### Decision

- **策略**: 渐进式适配，非一次重构。
- **阶段 1（P0）**: 核心页面 CSS 响应式 — 首页视频网格、播放页布局、导航栏汉堡菜单（Tailwind `sm/md/lg` 断点）
- **阶段 2（P1）**: 播放器移动端组件 — 手势控制（双击暂停、左右滑动快进、上下滑动音量/亮度）；全屏横屏适配
- **阶段 3（P2）**: 管理后台最小可用 — 表格横向滚动、表单堆叠布局
- **不自建移动端 App**: 优先 PWA（`manifest.json` + Service Worker），后期评估 React Native/Flutter
- **测试**: Chrome DevTools 设备模拟 + 真机测试（iPhone + Android 各一）

### Consequences

- 变得容易: 复用现有 Vue 组件，不引入新框架
- 接受代价: CSS 响应式无法达到原生 App 体验；复杂管理后台页面在手机上体验较差
- 锁定规则: 新组件必须同时考虑 PC + Mobile 布局；使用 Tailwind 响应式前缀

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| React Native / Flutter 独立 App | 1 人无法维护 3 套代码（PC + iOS + Android） |
| 独立移动端 SPA | 代码分叉，维护成本翻倍 |

---

## ADR-014: 统一支付网关抽象 — 支付宝/微信双通道

**状态:** 提议  
**日期:** 2026-06-27  
**决策者:** 架构师 Winston  
**驱动:** FR-045 (大会员), FR-046 (充电), FR-047 (激励提现), FR-052 (付费课程), NFR-PAY-1/2

### Context

P2 阶段引入多个支付场景（大会员订阅、充电打赏、课程购买、收益提现），需要统一的支付抽象避免每个场景对接一次。

### Decision

- **支付网关接口**: Go interface `PaymentGateway` 定义 `CreateOrder()` / `QueryOrder()` / `Refund()` / `VerifyCallback()`
- **双通道实现**: `AlipayGateway` + `WechatPayGateway`，通过配置切换
- **回调处理**: 统一回调端点 `POST /api/v1/payment/callback/{channel}` — 验签 → 幂等（order_id 去重）→ 更新订单状态 → 触发业务回调
- **订单模型**: `PaymentOrder` 表（id, user_id, order_no, channel, amount_cent, subject, status, paid_at）
- **对账**: 每日凌晨 Cron Job 拉取支付宝/微信账单 → 比对 `PaymentOrder` → 差异告警（`alert_rules` 复用 ADR 告警系统）
- **安全**: HTTPS 强制；回调签名验证；金额以 `int64` 分(cent) 存储避免浮点精度

### Consequences

- 变得容易: 新增支付场景只需调用 `gateway.CreateOrder()`；对账自动发现异常
- 接受代价: 支付宝/微信 SDK 依赖；需要商户账号和审核
- 锁定规则: 金额统一用分存储；支付回调必须幂等

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 仅支持一种支付方式 | 用户覆盖不全 |
| Stripe | 主要面向海外，支付宝/微信国内更普适 |

---

## ADR-015: 直播技术选型 — SRS + RTMP/HLS/WebRTC

**状态:** 提议  
**日期:** 2026-06-27  
**决策者:** 架构师 Winston  
**驱动:** FR-050, NFR-LIVE-1/2

### Context

P3 阶段引入直播功能。需要流媒体服务器支持推流、转码、分发。需要与现有 WebSocket 弹幕系统无缝集成。

### Decision

- **流媒体服务器**: **SRS**（Simple Realtime Server）— 国产开源，社区活跃，支持 RTMP/WebRTC/HLS/FLV
- **推流协议**: RTMP（OBS 兼容）→ SRS 接收
- **播放协议**: 
  - 低延迟场景: WebRTC（延迟 ≤ 1s）
  - 兼容性场景: HLS（延迟 3-5s，iOS Safari 原生支持）
  - 回退: HTTP-FLV
- **转码**: SRS FFmpeg 集成 → 多码率输出（同 ADR-010 策略）
- **录制**: SRS DVR → 自动转点播视频（复用现有转码 Pipeline）
- **弹幕集成**: 直播间 WebSocket 端点 `/ws/live/{room_id}`，复用现有 danmaku WebSocket 架构
- **数据模型**: `LiveRoom` 表（id, user_id, title, cover, status: idle/live/ended, stream_key, viewer_count）

### Consequences

- 变得容易: SRS 单二进制部署；与 Nginx 同机部署；HTTP Callback 对接业务系统
- 接受代价: 新增基础设施（SRS 进程）；WebRTC 需 TURN/STUN 服务器（复杂网络环境）
- 锁定规则: 推流密钥 `stream_key` 每个用户唯一；SRS HTTP Callback `on_publish`/`on_play` 对接认证+计数

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| ZLMediaKit | 功能更强但配置复杂；C++ 生态不如 SRS 对 Go 友好 |
| 云直播服务（阿里云/腾讯云） | 成本高（按流量计费）；无法自控 |

### 重新审视条件

并发直播间 > 100 或单房间 > 10000 人时，评估 SRS 集群 + Edge 节点分发。

---

## ADR-016: MMR/DPP 多样性重排序 + 用户画像分段

**状态:** 已接受  
**日期:** 2026-06-28  
**决策者:** PandaGuGu + WorkBuddy  
**驱动:** FR-036, NFR-REC-1/2

### Context

Feed 推荐此前仅为 SQL `ORDER BY play_count DESC`，无多样性控制，导致首页连续推荐同分区/同UP主/同类视频。需要在不大规模引入 ML 基础设施的前提下提升推荐多样性和个性化。

### Decision

- **算法**: 同时实现 **MMR**（Maximal Marginal Relevance）和 **DPP**（Determinantal Point Process）两种重排序算法
- **MMR 公式**: `argmax[ λ×relevance(i) − (1−λ)×max sim(i,j) ]`
- **评分维度**: 加权质量分（play×1+like×10+coin×20+fav×5+danmaku×3）× 时间衰减 e^(-0.01·days)
- **相似度**: Jaccard 标签 × 0.5 + Zone 匹配 × 0.3 + 同 UP 主惩罚 × 0.2
- **用户画像**: 从观看/点赞/投币/收藏/搜索 5 表提取 zone 亲和度 + tag 亲和度 → 自适应 λ（0.5~0.9） → 四段分类（seg_anime/game/tech/life/mix）
- **缓存策略**: 
  - 候选池 Redis ZSET 每 60s 预热（Top 300）
  - 匿名用户结果 Redis 缓存 30s（确定性输出，命中率高）
  - 用户画像 Redis Hash TTL 7d
- **代码位置**: `internal/service/rerank.go` + `user_profile.go` + `feed_service.go`

### Consequences

- 变得容易: 纯 Go 实现无外部依赖；用户 λ 自适应无需人工调参
- 接受代价: 首次请求需实时计算（~5-10ms），匿名用户预热后命中缓存即 <2ms
- 锁定规则: 候选池 ≤ 300；MMR 默认 λ=0.7；用户画像异步构建不阻塞请求

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| ML 协同过滤 | 需离线训练 + 特征工程，1 人团队不可维护 |
| 纯随机打散 | 牺牲相关性 |

---

## ADR-017: GORM 软删除保障视频数据安全

**状态:** 已接受  
**日期:** 2026-06-28  
**决策者:** PandaGuGu + WorkBuddy（事故驱动）  
**驱动:** NFR-DATA-1, 2026-06-28 误删事故

### Context

2026-06-28 因批量删除脚本逻辑错误，全部 29 个视频（含用户稿件）被级联硬删除，无备份无法恢复。原有 `deleteVideoCascade()` 使用 GORM `Delete()` 物理清除行。需要防止同类事故。

### Decision

- **软删除**: `model.Video` 新增 `DeletedAt gorm.DeletedAt \`gorm:"index"\`` 字段
- **行为变化**: GORM 自动将 `tx.Delete(&model.Video{})` 转为 `UPDATE SET deleted_at = NOW()`，所有默认查询自动附加 `WHERE deleted_at IS NULL`
- **误删恢复**: `UPDATE videos SET deleted_at = NULL WHERE id = ?`（一行 SQL 恢复）
- **子表行为**: `VideoLike` / `VideoCoin` / `VideoFavorite` / `Danmaku` 等子表保持硬删除（GORM 无软删除字段），仅 `videos` 主表软删除
- **查询兼容**: 所有现有 `WHERE status = 'published'` 查询无需修改（GORM 自动过滤软删行）

### Consequences

- 变得容易: 误删可秒级恢复；无需改业务代码
- 接受代价: `deleted_at` 索引轻微写放大；子表关联数据不可恢复（仅恢复视频元数据）
- 锁定规则: 所有视频删除操作必须通过 GORM（不可用 `Exec("DELETE FROM videos")`）

### 替代方案

| 方案 | 为何拒绝 |
|------|---------|
| 数据库备份 | 需运维配置，恢复窗口 ≥ 分钟级 |
| 回收站表 | 需新建表 + 修改所有 handler，工程量大 |
| 变更前手动备份 | 依赖人记忆，不可靠 |
