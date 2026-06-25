# 运营后台模块闭环审计

> 2026-06-25 | 检查维度：Model → Route → Handler → Vue Page → 业务闭环

---

## 审计结果总览

| # | 模块 | Model | Route | Handler | Vue | 闭环 | 缺口 |
|---|------|-------|-------|---------|-----|------|------|
| 1 | 用户管理 | ✅ User | ✅ 6条 | ✅ user.go | ✅ UserManage | ✅ | — |
| 2 | 视频审核 | ✅ Video | ✅ 6条 | ✅ video.go | ✅ VideoReview | ✅ | — |
| 3 | 专栏审核 | ✅ Article | ✅ 6条 | ✅ article.go | ✅ ArticleReview | ✅ | — |
| 4 | 动态管理 | ✅ Dynamic | ✅ 3条 | ✅ dynamic.go | ✅ DynamicManage | ✅ | — |
| 5 | 评论管理 | ✅ Comment | ✅ 3条 | ✅ comment.go | ✅ CommentManage | ✅ | — |
| 6 | 举报处理 | ✅ Report | ✅ 3条 | ✅ report.go | ✅ ReportManage | ✅ | — |
| 7 | AI角色 | ✅ AgentProfile | ✅ 9条 | ✅ agent.go | ✅ AgentManage | ✅ | — |
| 8 | 工单管理 | ✅ Ticket | ✅ 7条 | ✅ ticket.go | ✅ TicketManage | 🟡 | 缺SLA定时器、满意度 |
| 9 | 风控管理 | ✅ RiskRule | ✅ 8条 | ✅ risk.go | ✅ RiskManage | 🟡 | 缺命中日志前端、行为分析 |
| 10 | 版权管理 | ✅ CopyrightComplaint | ✅ 6条 | ✅ copyright.go | ✅ CopyrightManage | 🟡 | 缺反通知、通知链路 |
| 11 | 数据报表 | ✅ SavedReport | ✅ 7条 | ✅ bi.go | ✅ BIReport | 🟡 | 缺导出文件生成、定时推送 |
| 12 | 客服后台 | ✅ CSConversation | ✅ 8条 | ✅ cs.go | ✅ CSManage | 🟡 | 缺排队分配、会话质检 |
| 13 | 运维监控 | ✅ TaskLog等5表 | ✅ 15条 | ✅ ops.go | ✅ OpsMonitor | 🟡 | 缺CPU/内存采集、大屏 |
| 14 | 配置发布 | ✅ FeatureFlag | ✅ 6条 | ✅ config.go | ✅ ConfigManage | 🟡 | 缺审批流、A/B实验 |
| 15 | 权限审计 | ✅ AdminRole等5表 | ✅ 12条 | ✅ rbac.go | ✅ RBACManage | 🟡 | 缺操作二次确认 |
| 16 | 专题活动 | ✅ SpecialPage | ✅ 8条 | ✅ special.go | ✅ SpecialManage | 🟡 | 缺可视化搭建器 |
| 17 | 字幕管理 | ✅ Subtitle | ✅ 2条 | ✅ subtitle.go | ✅ SubtitleManage | 🟡 | 不缺前后端，缺审核队列 |

---

## 闭环详解（✅已闭环）

### 1-7: 核心审核模块（全部闭环）
- **数据流**: 用户发内容 → 触发审核 → 管理员通过/拒绝 → 内容状态变更
- **DB**: User/Video/Article/Dynamic/Comment/Report/AgentProfile 字段完整
- **API**: 列表/详情/操作（通过/拒绝/删除/封禁）齐全
- **前端**: 对应 Vue 页面可操作

---

## 半闭环详解（🟡缺业务逻辑）

### 8. 工单管理
```
✅ 现有: 创建 → 查看 → 指派 → 状态变更(处理中/已解决/已关闭) → 留言
❌ 缺口: 
  - 没有定时扫描 open→超时升级
  - 关闭后没有满意度评分入口
  - 用户前端没有"我的工单"查看页（只有admin端）
```
**前后端**: 全有，差的是一个 `time.Ticker` goroutine 扫描超时工单 + 一个满意度表

### 9. 风控管理
```
✅ 现有: 规则CRUD + 黑白名单CRUD + 规则开关 + RiskHitLog 模型
❌ 缺口:
  - RiskManage.vue 没有"命中日志"Tab（后端API已有 /risk/hits）
  - 命中规则后没有自动执行动作（comment.go/video.go 没有调用 ScanContentRisk）
```
**前后端**: 全有，后端的 `ScanContentRisk` 函数已写但**没人调用**。前端差一个 Tab。

### 10. 版权管理
```
✅ 现有: 投诉提交 → 查看 → 受理/驳回 → 下架/恢复
❌ 缺口:
  - 没有"通知被诉方"环节（受理后应发站内信）
  - 没有"反通知"机制（被下架方可申诉）
  - 没有 DMCA 标准 7 天等待期
```
**前后端**: 全有，差通知链路（调一下站内信 API 即可）

### 11. 数据报表
```
✅ 现有: 分区统计 + 创作者统计 + 时序数据 + SavedReport CRUD
❌ 缺口:
  - /bi/export 路由存在但 handler 只返回 JSON，没生成真实文件
  - 没有"定时推送日报/周报"
```
**前后端**: 全有，export 端点需要改写成生成 CSV/Excel 文件流

### 12. 客服后台
```
✅ 现有: 会话列表 → 接入 → 发消息 → 关闭 + 模板CRUD
❌ 缺口:
  - 没有排队/轮询分配（assign 是手动）
  - 没有用户画像侧栏（查历史投诉/评论）
```
**前后端**: 全有，差分配算法（改 assign 从手动→自动取最少负载的客服）

### 13. 运维监控
```
✅ 现有: TaskLog + AlertRule + AlertRecord + Trace + CDN + OSS 生命周期
❌ 缺口:
  - 没有实际的 CPU/内存/QPS 采集（AlertRule 的阈值无数据源）
  - 没有可视化大屏
```
**前后端**: 全有，差一个 `/ops/metrics` 端点返回 runtime.MemStats + QPS

### 14. 配置发布
```
✅ 现有: FeatureFlag CRUD + Release CRUD + toggle + rollback
❌ 缺口:
  - publish 动作没有审批流（谁都能按发布）
  - 前端 toggle 按钮没有二次确认
```
**前后端**: 全有，差一个发布审批接入已有 ApprovalFlow

### 15. 权限审计
```
✅ 现有: Role + Permission + Assignment + AuditLog + LoginLog + ApprovalFlow
❌ 缺口:
  - 前端没有展示登录日志 Tab
  - 删除视频/封号没有弹出二次确认
```
**前后端**: 全有，差前端 Tab + 一个确认弹窗

### 16. 专题活动
```
✅ 现有: SpecialPage CRUD + Campaign CRUD + 时间字段
❌ 缺口:
  - 内容块编辑是裸 JSON 文本框，无可视化
  - 活动没有参与统计
```
**前后端**: 全有，可视化编辑器是大工程

### 17. 字幕管理
```
✅ 现有: 列表 + 删除 + 创作者端上传/编辑/预览
❌ 缺口:
  - 没有审核队列（用户在 SubtitleEdit 上传后直接生效）
```
**前后端**: 全有，差一个 `status=pending` 中间状态

---

## 核心问题归纳

**好消息**: 所有 17 个模块的 Model/Route/Handler/Vue 四层都存在，不存在"缺胳膊少腿"。

**坏消息**: 9 个模块是"能看不能跑"——页面能打开，数据能列出来，但业务流程走不完。根源是：

1. **事件驱动缺失** — 风控命中后没自动执行、工单没超时扫描
2. **通知链路断裂** — 版权受理后不通知被诉方、客服分配靠手动
3. **前端最后一公里** — 后端 API 有了，前端没接（命中日志/登录日志 Tab）

**修正成本**: 除了专题可视化编辑器是大工程，其余 8 个模块的缺口都是 1-2 个函数/1 个 Tab 的事。
