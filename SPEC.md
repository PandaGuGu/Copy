## Mini-Bili 设计规格文档（SPEC）

**版本**：v2.1
**最后更新**：2026-06-28
**性质**：确定性需求规格

> v1.0 核心体验（F0-F10，AC-1 至 AC-16）已于 2026-05 全部达标。v2.0 在此基础上扩展运营后台、直播、社交、Service 层架构。v2.1 新增推荐引擎规划。

---

### 一、版本目标与范围

#### 1.1 v1.0（已完成）
用户认证与视频管理、实时弹幕、多级评论。详见下方"已完成的功能需求"。

#### 1.2 v2.0 新增
1. **运营后台 23 模块**：审核、举报、工单、风控、版权、BI、客服、运维、配置发布、RBAC 审计
2. **直播系统**：创建直播间、SRS 推流、flv.js 播放、WebSocket 聊天+礼物、直播历史
3. **社交体系**：关注/拉黑、私信、动态、收藏夹、硬币
4. **搜索**：ES 全文搜索、热搜、搜索历史
5. **Service 层**：internal/service/ 包，handler → service → DB 三层架构
6. **前端共享组件**：AdminDataTable / AdminFormDialog / api/admin/ 模块化

#### 1.3 明确排除（Non-Commercial License）
- 不做支付、会员、充电等任何商业化功能（LICENSE Non-Commercial）
- 不做 ML 推荐算法（当前为规则/热度排序）
- 不做移动端/小程序适配
- 不做 CDN 实际分发（仅有管理 CRUD 接口）

---

### 二、已完成的功能需求（v1.0）

#### F0：用户个人信息管理
完全实现：GET/PUT /users/me、POST /users/me/avatar、PUT /users/me/password、GET /users/me/videos

#### F1：用户认证
JWT 双 Token（Access 2h + Refresh 7d），bcrypt 密码，注册唯一性校验，独立 Admin JWT

#### F2：视频上传
multipart ≤500MB ≤30min → RabbitMQ → FFmpeg H.264 MP4 → OSS videos/{id}.mp4

#### F2-b：视频状态管理
processing / published / failed / pending_review / rejected，5 状态全部已激活使用

#### F3-F4：视频信息展示与播放
封面（默认+自定义+替换）、播放量 Redis 10s 落库、HTML5 video 播放器、倍速/PiP/章节面板

#### F5-F6：弹幕发送与实时显示
WebSocket ≤200ms、5s 冷却、敏感词过滤、Canvas 多轨道渲染、200 条历史

#### F7-F9：评论系统
3 级嵌套、UP 主权限（精选/关闭/置顶/级联删除）、点赞 toggle、聚合通知、消息中心 5 分类

#### F10：视频列表
首页 published 排序、卡片信息、Feed 推荐流、排行榜、分区

---

### 三、v2.0 新增功能需求

#### F11：运营后台（23 模块）

| 模块 | 后端文件 | 前端页面 | 状态 |
|------|---------|---------|------|
| 数据概览 | admin_dashboard.go | data/Dashboard.vue | ✅ |
| 首页轮播 | admin_banner.go | content/BannerManage.vue | ✅ |
| 热搜运营 | admin_hot_search.go | content/HotSearchManage.vue | ✅ |
| 用户管理 | admin_user.go | UserManage.vue | ✅ |
| 视频审核 | admin_video.go | review/VideoReview.vue | ✅ |
| 专栏审核 | admin_article.go | review/ArticleReview.vue | ✅ |
| 直播管理 | admin_live.go | review/LiveManage.vue | ✅ |
| 动态管理 | admin_dynamic.go | DynamicManage.vue | ✅ |
| 评论管理 | admin_comment.go | social/CommentManage.vue | ✅ |
| 系统设置 | admin_settings.go | Settings.vue | ✅ |
| 举报处理 | report.go | social/ReportManage.vue | ✅ |
| AI 角色 | admin_agent.go | AgentManage.vue | ✅ |
| LLM 配置 | admin_llm_config.go | (嵌入 AgentManage) | ✅ |
| 工单管理 | admin_ticket.go | social/TicketManage.vue | ✅ |
| 风控管理 | admin_risk.go | social/RiskManage.vue | ✅ |
| 版权管理 | admin_copyright.go | social/CopyrightManage.vue | ✅ |
| BI 报表 | admin_bi.go | data/BIReport.vue | ✅ |
| 客服后台 | admin_cs.go | social/CSManage.vue | ✅ |
| 运维监控 | admin_ops.go | ops/OpsMonitor.vue | ✅ |
| 配置发布 | admin_config.go | ops/ConfigManage.vue | ✅ |
| 权限审计 | admin_rbac.go | ops/RBACManage.vue | ✅ |
| 字幕管理 | subtitle.go | content/SubtitleManage.vue | ✅ |
| 专题活动 | admin_special.go | content/SpecialManage.vue | ✅ |

共 82 张数据表，80+ API 端点，19 种 RBAC 权限码（resource:action 格式）。

#### F12：直播系统
创建直播 → SRS 推流回调 → flv.js 播放 → WebSocket 实时聊天 + 礼物 → 观众追踪 → 直播历史记录 → 管理后台审核/警告/封禁

#### F13：社交体系
关注/取关、拉黑（双向互阻）、关注分组、多收藏夹、投币（coin_ledgers）、图文动态发布、私信 WebSocket 实时推送

#### F14：搜索与发现
ES 全文搜索、热搜运营（Redis 热词 + 管理干预）、搜索历史、Feed 推荐（规则/热度）、排行榜

#### F15：Service 层架构
```
handler（HTTP 请求处理）
  → service（业务逻辑）
    → gorm.DB（数据访问）
```
`internal/service/` 包：Services 容器 → VideoService / UserService / CommentService
通过 `handler.Dependencies.Svcs` DI 注入

#### F16：前端共享组件体系
- `components/admin/AdminDataTable.vue` — 统一搜索+表格+分页
- `components/admin/AdminFormDialog.vue` — 统一新增/编辑弹窗
- `utils/admin-helpers.js` — 共享 formatTime()
- `api/admin/` — 18 模块模块化 API（auth / banner / video / comment / user / rbac / cs / ticket / copyright / ...）
- admin 页面按分组子目录：data / review / ops / content / social

#### F17：推荐引擎（v2.1 规划中）

> 当前：F14 Feed 推荐为规则/热度排序（`ORDER BY play_count DESC`），所有用户看到同一份榜单。
> v2.1 计划：引入 ItemCF 协同过滤实现个性化推荐。

**一期 — ItemCF 协同过滤核心引擎：**

| 阶段 | 内容 | 技术路线 |
|------|------|---------|
| ① 离线计算 | 每日凌晨 Go 定时任务构建用户-视频交互矩阵 | 7 种行为加权（点赞1.0/投币3.0/收藏2.0/观看0.5/评论1.5/弹幕1.0）→ Cosine 相似度 → `video_similarities` 表 |
| ② 多路召回 | ItemCF + 内容(同zone/标签) + 热门(时间衰减) + 社交(关注UP主) | Go service 层，四路并发召回 → 加权融合 |
| ③ 粗排 | 加权打分公式 | `like×120 + coin×200 + fav×90 + dm×85 + play×1.2 + 时间衰减` |
| ④ 重排 | 类目打散 + 频控 | MMR 多样性 + Redis 曝光计数器 |
| ⑤ 在线服务 | `GET /api/v1/feed/recommendation` 个性化 | Redis 缓存 top-200 + MySQL 相似度表 |
| ⑥ 冷启动 | 新用户/新视频专门策略 | 新用户 → 热门兜底；新视频 → 内容相似度提权×2.0 |

**二期 — 排序模型升级：**
- LR/GBDT 排序替代加权公式
- AB 实验框架（分流+埋点+统计检验）
- 多目标优化（CTR + 停留时长 + 互动率）

**新增数据表：**
- `video_similarities` — 视频相似度矩阵
- `rec_exposure_log` — 推荐曝光日志

**评估指标：**
- 离线：Precision@K, Recall@K, NDCG@K
- 在线：CTR, 人均播放数, 7日留存

---

### 四、验收标准

#### v1.0 AC（16 项全部通过）
AC-1 至 AC-16：用户注册登录、视频上传转码、弹幕实时、评论嵌套、点赞通知、封面管理、删除权限 — 全部达标 ✅

#### v2.0 AC

| 编号 | 验收项 | 达标标准 |
|------|--------|---------|
| AC-17 | 管理员认证闭环 | /admin/login → JWT → /admin/me → RBAC 权限校验 → 侧边栏按权限过滤 |
| AC-18 | 视频审核全链路 | 上传 → pending_review → 管理员通过/驳回 → published/rejected |
| AC-19 | 工单全生命周期 | 用户提交 → 管理员分配 → 处理人回复 → 用户确认 → 关闭 → 申诉 |
| AC-20 | Feature Flag 灰度 | 创建 Flag → FNV-1a hash 分桶 → whitelist + rollout_pct → 非白名单用户返回 false |
| AC-21 | 直播审核 | 直播中 → 管理员警告 → WebSocket 广播到直播间 → 封禁频道 |
| AC-22 | Service 层编译 | go build ./... 零错误，internal/service/ 4 文件 |
| AC-23 | 前端 API 归队 | 所有 admin 页面通过 @/api/admin barrel 导入，零裸 adminHttp 调用 |
| AC-24 | 共享组件覆盖 | AdminDataTable 接入 9 个 admin 页面 |
| AC-25 | ItemCF 召回可用 | `GET /api/v1/feed/recommendation` 返回个性化结果（非全局热门），离线相似度计算成功入表 |

---

### 五、兼容要求

| 编号 | 类型 | 规格 |
|------|------|------|
| BC-1 | 向后兼容 | v1.0 用户端 API 不受 v2.0 管理端改动影响；管理端独立路由 /api/v1/admin |
| BC-2 | 向前兼容 | 模块化单体（Go Gin），代码架构支持未来按 module 拆分为独立服务 |
| BC-3 | 前端兼容 | AdminLayout.vue 统一布局，路由前缀 /admin/* 独立于用户端 |

---

### 六、非功能需求

| 编号 | 类型 | 规格 |
|------|------|------|
| NF-1 | 并发 | 弹幕 100 人在线 ≤200ms；运营后台 ~50 并发管理员 |
| NF-2 | 存储 | MySQL 主库 + Redis 热数据 + RabbitMQ 异步任务 + OSS 文件 |
| NF-3 | 鉴权 | 用户 JWT + Admin JWT 双体系隔离；RBAC resource:action 细粒度；全写操作审计 |
| NF-4 | API | RESTful，JSON 信封 {code, msg, data}，统一错误码 errcode 包 |
| NF-5 | 前端 | Vue 3 + Vite SPA，AdminLayout 统一后台布局 |
| NF-6 | 配置 | .env 文件 updateEnvKeys() 逐行更新；Feature Flag FNV-1a hash 灰度 |
| NF-7 | 测试 | go build ./... 编译验证；handler 层可集成测试 |
