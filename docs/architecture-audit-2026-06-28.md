# Cakecake 项目架构真实性审计报告

> **审计日期**: 2026-06-28  
> **审计方法**: BMAD Architecture Validate 模式 — 逐项对比 docs/、working_memory 声称 vs 代码实际  
> **代码扫描范围**: 99 个 Go 源文件 + 48+ Vue 页面 + 17 个 Admin API 模块

---

## 一、总体结论

| 项目 | 评分 |
|------|------|
| **文档-代码一致性** | **85/100** — 大部分一致，存在 4 处显著偏差 |
| **架构完整性** | **92/100** — 模块化单体模式执行彻底，依赖注入清晰 |
| **RBAC 实现度** | **95/100** — 23 种权限码全覆盖，审计日志完善 |
| **技术栈一致性** | **98/100** — 与文档声明高度吻合 |

---

## 二、逐项验证清单

### 2.1 后端架构

| # | 声称项 | 来源 | 代码实际 | 结论 |
|---|--------|------|---------|------|
| 1 | 24 个 `admin_*.go` handler 文件 | PRD | **25 个**（含 `admin_special.go`、`admin_live.go` 未计入原统计） | ⚠️ 差 1 |
| 2 | "80+ API 端点"（admin） | working_memory | **约 180+** 条 `/api/v1/admin/` 路由 | 🔴 严重低估 |
| 3 | 84 张数据表 | README | **85 个** GORM AutoMigrate 模型 | ⚠️ 差 1 |
| 4 | 19 种 RBAC 权限码 | SPEC | **23 种**（rbac_seed.go） | 🔴 低估 |
| 5 | JWT Access Token = 2h | README/SPEC | `2 * time.Hour` ✅ | ✅ 一致 |
| 6 | JWT Refresh Token = 7d（用户） | README | `30 * 24 * time.Hour`（30天） | 🔴 实际为 30 天 |
| 7 | JWT Refresh Token = 3d（Admin） | Skill.md | `72 * time.Hour` ✅ | ✅ 一致 |
| 8 | GORM 软删除（Video） | ADR-017 | `gorm.DeletedAt` 字段存在 ✅ | ✅ 一致 |
| 9 | bcrypt 密码加密 | README/Rule | `golang.org/x/crypto/bcrypt` ✅ | ✅ 一致 |
| 10 | 统一 JSON 响应 `{code,msg,data}` | 多处文档 | `resp/resp.go` OK/Err 实现 ✅ | ✅ 一致 |
| 11 | Service 层三层架构 | SPEC S-015 | `handler → service → DB` 5 个 service 文件 ✅ | ✅ 一致 |
| 12 | 20 个已注册错误码 | Skill.md S-006 | `errcode.go` 含 20+ 常量 ✅ | ✅ 一致 |

### 2.2 前端架构

| # | 声称项 | 来源 | 代码实际 | 结论 |
|---|--------|------|---------|------|
| 13 | Vue 3 + Vite + Element Plus | README | `package.json` 确认 ✅ | ✅ 一致 |
| 14 | 23 个运营后台模块 | working_memory | 24 个 .vue 文件（含 AdminLayout+AdminLogin） | ✅ 一致 |
| 15 | 17 个 Admin API 文件 | 代码统计 | 17 个文件（auth/banner/hot-search/video/article/dynamic/agent/copyright/comment/user/report/settings/dashboard/rbac/cs/ticket/special） | ✅ 一致 |
| 16 | Hash 路由模式 | 代码 | `createWebHashHistory` ✅ | ✅ 一致 |
| 17 | Vuex 4（非 Pinia） | 代码 | `vuex@^4.1.0` ✅ | ✅（SPEC 未明确声明 Pinia） |

### 2.3 基础设施

| # | 声称项 | 来源 | 代码实际 | 结论 |
|---|--------|------|---------|------|
| 18 | MySQL 8.0 + GORM | README | `go.mod` 含 `gorm.io/driver/mysql` ✅ | ✅ 一致 |
| 19 | Redis 7.0+ | README | `go-redis/v9` ✅ | ✅ 一致 |
| 20 | RabbitMQ 3.12+ | README | `amqp091-go` ✅ | ✅ 一致 |
| 21 | Elasticsearch 8.x（可选） | README | `go-elasticsearch/v8` ✅ | ✅ 一致 |
| 22 | WebSocket 三通道 | architecture.md | 三套独立 WS handler ✅ | ✅ 一致 |
| 23 | SRS + flv.js 直播 | README | `docker-compose.yml` SRS 服务 + `flv.js` 依赖 ✅ | ✅ 一致 |
| 24 | FFmpeg 异步转码 | README | `worker/transcode.go` + RabbitMQ ✅ | ✅ 一致 |
| 25 | Docker Compose 6 服务 | README | 6 个服务（MySQL/Redis/RabbitMQ/SRS/backend/frontend）✅ | ✅ 一致 |
| 26 | Non-Commercial License | LICENSE | 明确禁止商用 ✅ | ✅ 一致 |

### 2.4 文档内部矛盾

| # | 矛盾点 | 文档 A | 文档 B | 实际代码 |
|---|--------|--------|--------|---------|
| 27 | Refresh Token 时长 | README: 7d | Skill.md: 3d | **30d**（用户）/ **3d**（Admin） |
| 28 | 表数量 | README: 84 | SPEC: 82 | **85** |
| 29 | RBAC 权限码数 | SPEC: 19 | architecture.md: 19 | **23** |
| 30 | Admin 端点数 | MEMORY: 80+ | PRD: 多 | **~180** |

---

## 三、架构决策记录（ADR）状态核对

| ADR | 标题 | 文档状态 | 代码落地 | 备注 |
|-----|------|---------|---------|------|
| ADR-001 | REST + JSON 信封 | 已接受 | ✅ `resp/resp.go` | |
| ADR-002 | MySQL + GORM + Redis + RMQ + OSS | 已接受 | ✅ 全部实现 | |
| ADR-003 | 独立 Admin JWT | 已接受 | ✅ `jwttoken/admin.go` | |
| ADR-004 | RBAC resource:action | 已接受 | ✅ 23 种权限码 | 代码比文档多 4 种 |
| ADR-005 | 模块化单体 | 已接受 | ✅ handler 文件级拆分 | |
| ADR-006 | 全写操作审计日志 | 已接受 | ✅ AuditLog 模型 + 中间件 | |
| ADR-007 | 统一错误码 | 已接受 | ✅ `errcode.go` | |
| ADR-008 | FNV-1a Hash 灰度 | 已接受 | ✅ FeatureFlag + rollout | |
| ADR-009 | 审批流多级串行 | 已接受 | ✅ ApprovalFlow + ApprovalStep | |
| ADR-015 | SRS 直播 | 已实施 | ✅ LiveRoom + SRS callback | |
| ADR-016 | ItemCF 推荐引擎 | 设计中 | 🟡 MMR/DPP 已实现 | 超预期部分完成 |
| ADR-017 | GORM 软删除 | 已接受 | ✅ Video.DeletedAt | |

---

## 四、主要发现与建议

### 🔴 严重偏差（需立即修正文档）

1. **Admin API 端点严重低估**: working_memory 声称 "80+"，实际约 180+。建议更新为 "180+" 或按模块精确统计。

2. **Refresh Token 时长混乱**: README 说 7 天，Skill.md 说 3 天，实际代码用户端 30 天。建议统一为实际值：用户 30 天，Admin 3 天。

3. **RBAC 权限码数量**: 文档统一写 19 种，实际代码 23 种。新增的 `live:manage`、`special:manage`、`subtitle:manage`、`dynamic:manage` 未反映在文档中。

### ⚠️ 小偏差（建议修正）

4. **表数量**: README 写 84，实际 85（差 `LiveViewHistory` 的遗漏）。建议统查后更新。

5. **Admin Handler 文件**: PRD 说 24 个，实际 25 个。`admin_special.go` 是遗漏项。

6. **SPEC v2.1 推荐引擎**: ADR-016 标 "设计中" 但 MMR/DPP 重排序已实现。建议将 ADR-016 状态更新为 "部分实施"。

### ✅ 高可信赖项

以下声称经过验证完全准确：
- 技术栈（Go+Gin+Vue3+Vite+Element Plus+MySQL+Redis+RabbitMQ）
- Docker Compose 一键部署
- WebSocket 三通道架构
- GORM AutoMigrate 自动建表
- bcrypt 密码哈希
- 统一 JSON 响应格式
- 弹幕 Redis 冷却校验
- 敏感词过滤器
- Markdown 安全渲染（bluemonday）
- ip2region IP 定位

---

## 五、审计方法论

本审计遵循 BMAD Architecture Validate 流程：

1. 从 `docs/prd.md`、`docs/architecture.md`、`docs/decision-log.md` 提取所有可验证声称
2. 从 `SPEC.md`、`README.md`、`Skill.md`、`Rule.md` 提取一致性约束
3. 逐项对比 `internal/` 下 99 个 Go 文件和 `cakecake-vue/` 下所有 Vue/JS 文件
4. 对歧义点（如端点数量、JWT 时长）直接读取源代码确认
5. 报告缺口并给出修正建议
