# docs 索引

> 按「现行 / 历史快照」分开看：现行文档随代码更新，历史快照**只读**，用来追溯当时的决策与结论。

## 现行文档

| 文档 | 内容 | 什么时候看 |
|------|------|-----------|
| [API.md](./API.md) | 用户端 + 管理端全部端点、字段、错误码 | 接接口、对字段 |
| [APP.md](./APP.md) | **移动端（Rust + egui）**架构、页面清单、输入法/播放/构建 | 改 App、打 APK |
| [SECURITY.md](./SECURITY.md) | 安全基线与审计流程（常驻） | 改动鉴权/上传/权限前 |
| [GAP-TRACKER.md](./GAP-TRACKER.md) | 需求缺口看板（FR/NFR vs 实现） | 排期、找"为什么没做" |
| [RUN-CLI.md](./RUN-CLI.md) | 一键启停全栈的交互式 CLI | 本地起整套环境 |
| [ai-gateway.md](./ai-gateway.md) | 消息中心 AI 网关 | 改助手/消息中心 |
| [manual-video-ingest.md](./manual-video-ingest.md) | 服务器无法转码时的手动发布流程 | 生产排障 |
| [GIT-PUSH-GUIDE.md](./GIT-PUSH-GUIDE.md) | 提交/推送规范与常见坑 | 提交前 |

顶层另外三份：仓库总览 [../README.md](../README.md)、本地部署 [../LOCAL-DEPLOY.md](../LOCAL-DEPLOY.md)、
研发制度 [../Rule.md](../Rule.md) / [../Skill.md](../Skill.md)、架构与需求快照 [../bmad-output/architecture.md](../bmad-output/architecture.md)。

## 历史快照（只读）

| 文档 | 时间 | 说明 |
|------|------|------|
| [legacy/APP-uniapp.md](./legacy/APP-uniapp.md) | 2026-08 | **已弃用**的 uni-app 移动端（v0.1.0），仅作历史留存 |
| [architecture-audit-2026-06-28.md](./architecture-audit-2026-06-28.md) | 2026-06 | 架构审计结论 |
| [security-audit-2026-08-20.md](./security-audit-2026-08-20.md) | 2026-08 | 首次全量安全审计报告 |
| [PERF-REPORT.md](./PERF-REPORT.md) | 2026-08 | 单机性能与治理件验证 |

## 客户端路线

移动端 = **Rust（`cakecake-rs/`）**，2026-11 起取代 uni-app。
`cakecake-vue/` 只保留 Web 端（用户 H5 + 运营后台）；`cakecake-vue/cakecake-app/` 已停更，
原因与替代方案见 [APP.md §1](./APP.md#1-为什么弃用-uni-app)。
