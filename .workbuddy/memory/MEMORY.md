# 运营后台 & 23 模块扩展状态

> 最后更新：2026-06-29
> 状态：统一动态视图上线 + UserDynamic Type 字段新增

---

## 已实施模块（P0/P1/P2 基础）

| 模块 | 状态 | 后端文件 | 前端页面 |
|------|------|---------|---------|
| 用户管理 | ✅ | `admin_user.go` | `UserManage.vue` |
| 评论管理 | ✅ | `admin_comment.go` | `CommentManage.vue` |
| 数据仪表盘 | ✅ | `admin_dashboard.go` | `Dashboard.vue` |
| 系统设置 | ✅ | `admin_settings.go` | `Settings.vue` |
| 举报处理 | ✅ | `report.go` | `ReportManage.vue` |
| LLM 配置 | ✅ | `admin_llm_config.go` | 嵌入 `AgentManage.vue` |

---

## 23 模块扩展（2026-06-24 新增）

| # | 模块 | 后端 | 前端 | 文件 |
|---|------|------|------|------|
| 2 | 播放器高级 | ✅ | 🟡 | `admin_player.go` / `feed.go` |
| 3 | 字幕管理 | ✅ | 🟡 | `subtitle.go` |
| 4 | 评论增强 | ✅ | 🟡 | `comment_enhance.go` |
| 5 | 创作者中心 | ✅ | ✅ | `creator_center.go` |
| 7 | Feed推荐 | ✅ | 🟡 | `feed.go` |
| 13 | 工单系统 | ✅ | ✅ | `admin_ticket.go` / `TicketManage.vue` |
| 14 | 风控管理 | ✅ | ✅ | `admin_risk.go` / `RiskManage.vue` — 引擎重写：黑白名单+正则+频率限制+auto_ban真封号 |
| 15 | 版权管理 | ✅ | ✅ | `admin_copyright.go` / `CopyrightManage.vue` |
| 16 | BI报表 | ✅ | ✅ | `admin_bi.go` / `BIReport.vue` |
| 17 | 客服后台 | ✅ | ✅ | `admin_cs.go` / `CSManage.vue` |
| 18-22 | 运维5合1 | ✅ | ✅ | `admin_ops.go` / `OpsMonitor.vue` |
| 21 | 配置发布 | ✅ | ✅ | `admin_config.go` / `ConfigManage.vue` — 模块注册(一键)、功能开关、版本发布三 tab |
| 23 | RBAC审计 | ✅ | ✅ | `admin_rbac.go` / `RBACManage.vue` |

新增模型文件：`internal/model/module_extend.go`（20+ 模型）
新增路由：180+ API 端点（admin ~150 / auth ~20 / public ~10）

---

## 运维监控 5 合 1 生产化（2026-06-25）

| 模块 | 状态 | 生产者 | 说明 |
|------|------|--------|------|
| 任务队列 | ✅ | `worker/transcode.go` finishTaskLog | 转码任务生命周期写入 TaskLog |
| 告警 | ✅ | `POST /ops/alerts/evaluate` | 指标采集+阈值比较+告警记录 |
| 链路追踪 | ✅ | `middleware/trace.go` 全局中间件 | 自动记录 TraceRecord |
| 系统健康 | ✅ | `GET /ops/health` 增强 | items[] 格式 + 延迟/详情 |
| CDN / 存储 | 🟡 | CRUD 前后端已对齐 | 未对接真实 CDN/OSS 服务；字段/路径已于 2026-06-25 对齐 |
| 告警级别 | ✅ | 阈值偏离度分级 | critical (≥2x) / warning (≥1.3x) / info，2026-06-25 修复 |
| 生命周期 | ✅ | CRUD + PUT | 路径对齐 `/ops/storage/lifecycle-rules`，字段对齐前端 |
| report_export TaskLog | ✅ | admin_bi.go AdminExportReport | 服务端导出自动写 TaskLog 生命周期 |
| sync TaskLog | ✅ | admin_ops.go AdminTriggerSync | POST /ops/sync/trigger 触发 ES/播放量同步，写 TaskLog |
| BI 总览 | ✅ | GET /bi/summary | 9 张概览卡片（用户/视频/文章/评论/播放/弹幕等） |
| 文章统计 | ✅ | GET /bi/article-stats | 分类分布、热门文章 TOP20、时序 |
| 互动统计 | ✅ | GET /bi/engagement-stats | 评论/弹幕/点赞/收藏/投币/关注时序+累计值 |

## 侧边栏顺序（更新）

1. 数据概览 | 2. 首页轮播 | 3. 热搜运营 | 4. 用户管理
5. 视频审核 | 6. 专栏审核 | 7. 动态管理 | 8. 评论管理
9. 系统设置 | 10. 举报处理 | 11. AI 角色 | 12. 工单管理
13. 风控管理 | 14. 版权管理 | 15. 数据报表 | 16. 客服后台
17. 运维监控 | 18. 配置发布 | 19. 权限审计

---

## 关键架构约定

- `.env` 写入模式：`updateEnvKeys()` 逐行更新，追加缺失 key
- 系统设置/LLM 配置保存时同步更新内存 Cfg + `.env` 文件
- 评论管理跨 3 表联合查询
- Admin 中心自动建表（GORM AutoMigrate），无需手动 SQL
- GitHub 推送：`git push https://TOKEN@github.com/PandaGuGu/Copy.git main`
- 所有写操作 handler 自动记录 AuditLog（admin_rbac.go 共享 recordAudit）
- Feature Flag 灰度：FNV-1a hash 分桶 + whitelist + rollout_pct
- 配置发布流程：
  - 模块注册 tab：一键填表 → 自动创建 Flag+发布+部署
  - 功能开关 tab：管理 Flag → 版本发布 tab：新建发布（自动快照）→ 部署上线（apply 快照到 live DB）
  - 状态流转: draft → deployed → rolled_back
  - `POST /admin/config/releases/:id/deploy` → 解析快照 → 逐条 UPDATE feature_flags 表
  - `GET /admin/config/releases/:id/export` → 下载 JSON 快照
  - `GET /admin/config/releases/:id/snapshot` → 在线查看快照
  - ReleaseRecord 新增 Title / Type(canary/full/hotfix) / Notes / Snapshot / ReleasedAt 字段
- 动态管理（2026-06-29）：
  - `UserDynamic.Type`: `image`（图文）/ `text`（纯文字），创建/编辑时自动判定
  - `GET /api/v1/admin/dynamics/unified`: 三表 UNION（videos+articles+user_dynamics），支持 `user_id` / `kind`(video|article|image|text) / `q` 过滤
  - 前端 `DynamicManage.vue` 对接统一端点，类别标签用不同颜色：视频(primary) / 专栏(warning) / 图文(success) / 文字(info)
  - 删除仅支持图文动态（kind=image/text），video/article 需走对应审核模块
- BI 报表（2026-06-29）：
  - 图表库：ECharts 5，通用封装 `BiChart.vue`（深监听 option + resize）
  - 卡片：`BiCard.vue`（label/value/trend/color，左侧彩色边框）
  - BIReport.vue：全部手写 SVG → ECharts（柱状/饼图/折线面积/多系列），代码量减少 ~40%
