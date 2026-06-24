# 运营后台 & 23 模块扩展状态

> 最后更新：2026-06-24
> 状态：P0-P2 已完成，23 模块扩展后端+前端已实施

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
| 14 | 风控管理 | ✅ | ✅ | `admin_risk.go` / `RiskManage.vue` |
| 15 | 版权管理 | ✅ | ✅ | `admin_copyright.go` / `CopyrightManage.vue` |
| 16 | BI报表 | ✅ | ✅ | `admin_bi.go` / `BIReport.vue` |
| 17 | 客服后台 | ✅ | ✅ | `admin_cs.go` / `CSManage.vue` |
| 18-22 | 运维5合1 | ✅ | ✅ | `admin_ops.go` / `OpsMonitor.vue` |
| 21 | 配置发布 | ✅ | ✅ | `admin_config.go` / `ConfigManage.vue` |
| 23 | RBAC审计 | ✅ | ✅ | `admin_rbac.go` / `RBACManage.vue` |

新增模型文件：`internal/model/module_extend.go`（20+ 模型）
新增路由：80+ API 端点（admin 60+ / auth 20+ / public 5+）

---

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
