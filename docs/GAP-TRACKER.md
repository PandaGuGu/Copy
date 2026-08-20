# 覆盖缺口跟踪表（GAP-TRACKER）

> 来源：`bmad-output/architecture.md` §7「覆盖缺口」（FR/NFR 需求 vs 实现）。
> 用途：需求黑洞的状态看板，替代文档里散落的"待实施/待处理/推迟"标注。
> 更新约定：每次实施/决策后在此登记状态与日期。

## 状态图例

| 状态 | 含义 |
|------|------|
| 待实施 | 已排期或认可必要性，未动工 |
| 部分实施 | 后端/前端有一侧就绪 |
| 推迟 | 明确暂缓，记录原因 |
| 已取消 | 决策不做，记录原因 |
| 已完成 | 验收通过 |

## 缺口清单

| ID | 需求 | 缺口描述 | 优先级 | 状态 | 备注 / 决策记录 | 最后更新 |
|----|------|----------|--------|------|-----------------|----------|
| FR-032 | ASR 自动转写 | Worker 预留 `subtitle_asr`，未集成 Whisper | P2 | 待实施 | 无外部 ASR 依赖，可本地 whisper.cpp | 2026-08-20 |
| FR-031 | 字幕编辑器前端 | 后端就绪，用户端缺字幕时间轴编辑器 UI | P2 | 待实施 | 与播放器增强同批（P0 路线） | 2026-08-20 |
| FR-035 | 创作者数据中心 | `creator_center.go` API + `CreatorDashboard.vue`（stats/video-stats/7日趋势/稿件表）均已就绪 | P1 | 已完成 | 2026-08-20 复核：前端组件与路由（creatorDashboard）已存在并接真实 API，标记完成 | 2026-08-20 |
| NFR-REL-1 | 每日备份 | 新增 `scripts/backup.sh`（uploads 冷备 + mysqldump 逻辑备份 + 保留 N 份） | P1 | 部分实施 | 2026-08-20 脚本就绪；定时执行待用户挂 Windows 计划任务 | 2026-08-20 |
| NFR-REL-2 | 灾难恢复 | `migrations/` 版本化迁移基线 + `backup.sh` 恢复说明 | P2 | 部分实施 | 随 NFR-REL-1 一并推进 | 2026-08-20 |
| NFR-SEC-4 | API 限流 | Token Bucket 中间件 `ratelimit.go` + config 7 项 + 路由接入 + 错误码 42900 均已落地 | P1 | 已完成 | 2026-08-20 复核：全链路已存在，标记完成（默认 Guest60/User300/Admin1000 per min） | 2026-08-20 |
| NFR-REC-2 | ItemCF 离线计算 | `internal/service/itemcf.go`（7 行为加权 + Cosine + 阈值 0.15）+ scheduler 每日任务 + feed 在线召回接入 | P2 | 部分实施 | 2026-08-20 离线计算 + 在线召回已实现；`video_similarities` 表随 AutoMigrate 建表 | 2026-08-20 |
| ADR-018 | 状态机治理 | `internal/pkg/statemachine`（8 域转移表 + Can/Transition 校验 + 审计钩子）+ 6 域接入 + 3 时间驱动执行器（定时发布/SLA/自动解封） | P1 | 已完成 | 2026-08-20 落地：视频/工单/文章/版权/审批接入状态机校验；补定时发布消费者（原无消费者）；SLA 改按 sla_deadline 驱动；自动解封走状态机；3 项测试通过 | 2026-08-20 |

## 变更记录

| 日期 | 变更 |
|------|------|
| 2026-08-20 | 建表；NFR-REL-1/2 标记推迟（用户决策：不需要备份） |
| 2026-08-20 | 复核修正：NFR-SEC-4 限流与 FR-035 创作者中心标记已完成（此前文档过时）；新增 backup.sh（NFR-REL-1/2 部分实施）；新增 ItemCF 离线计算 + 在线召回（NFR-REC-2 部分实施）；新增 migrations/ 版本化迁移（DB_MIGRATE_TOOL 开关） |
| 2026-08-20 | ADR-018 状态机治理落地：statemachine 包（8 域）+ 6 域接入 + 定时发布消费者（修复无消费者缺口）+ SLA 按 sla_deadline 驱动 + 自动解封走状态机；main.go 旧 SLA/unban 逻辑收敛入 scheduler |
