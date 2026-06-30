# cakecake

<p align="center">
  <b>仿 B 站的全栈视频社交平台</b><br>
  <sub>视频上传/异步转码 · 弹幕 WebSocket · 直播 SRS · 评论/私信/硬币 · 23 模块运营后台(全前后端对齐) · Docker 一键部署</sub>
</p>

<p align="center">
  <b>A Bilibili-like Full-Stack Video Social Platform</b><br>
  <sub>Upload + Async Transcode (FFmpeg + RabbitMQ) · Danmaku via WebSocket · Live Streaming (SRS) · Comments / DMs / Coin Economy · 23 Admin Modules · Docker One-Click</sub>
</p>

<p align="center">
  <a href="#docker-一键部署"><img src="https://img.shields.io/badge/Docker-ready-2496ED?logo=docker&style=flat-square" alt="Docker"></a>
  <img src="https://img.shields.io/badge/Go-1.25-00ADD8?logo=go&style=flat-square" alt="Go">
  <img src="https://img.shields.io/badge/Vue-3.5-4FC08D?logo=vuedotjs&style=flat-square" alt="Vue">
  <img src="https://img.shields.io/badge/license-MIT-lightgrey?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/tables-86-orange?style=flat-square" alt="86 Tables">
  <img src="https://img.shields.io/badge/admin_modules-23-blueviolet?style=flat-square" alt="23 Admin Modules">
  <img src="https://img.shields.io/badge/RBAC_perms-23-important?style=flat-square" alt="23 RBAC Perms">
</p>

仿 B 站核心链路的全栈视频社交平台（用户端品牌 **cakecake**），后端 Go 模块名 `minibili`。

> 项目灵感来源于 [earthcake2233/cakecake](https://github.com/earthcake2233/cakecake)，在此基础上完成了数据库重构、运营后台全面扩建（86 张数据表，23 个后台模块，23 种 RBAC 权限码），形成了可投入生产级使用的完整系统。维护仓库：[PandaGuGu/Copy](https://github.com/PandaGuGu/Copy)。

---

## 系统概述

| 子系统 | 核心能力 |
|--------|----------|
| 用户认证 | 注册（唯一性校验）→ 登录（JWT 双 Token）→ 密码修改 → 个人信息维护 → 账号注销（7 天冷静期） |
| 视频管理 | 上传（≤500MB, ≤30min）→ 异步转码（FFmpeg→H.264 MP4→OSS）→ 状态机 → Redis 播放计数 |
| 弹幕系统 | WebSocket 长连接 → 历史弹幕推送 → 5s 冷却 + 敏感词过滤 → Canvas 多轨道渲染 |
| 评论系统 | 视频/文章/动态三套独立表 → 3 级嵌套 → UP 主管理（精选/关闭/置顶）→ 点赞/反对 → 聚合通知 |
| 社交互动 | 关注/取关 → 拉黑（双向互阻）→ 关注分组 → 多收藏夹 → 投币（coin_ledgers）→ 动态发布 |
| 私信聊天 | 一对一配对 → WebSocket 实时推送 → 未读计数 → 置顶/免打扰 → AI 对话 |
| 通知系统 | 点赞聚合 → 评论回复 → 消息中心 5 分类 → 未读角标 → 免打扰 |
| 搜索模块 | ES 全文搜索 → 搜索历史 → 观看/阅读/直播历史追踪 → 每日任务 |
| 直播系统 | 创建直播间 → UUID 推流密钥 → SRS RTMP 推流 → flv.js HTTP-FLV 播放 → WebSocket 聊天 + 6 种礼物 + 弹幕飘屏 → 管理员警告/封禁 |
| 推荐系统 | 协同过滤召回 → MMR 多样性重排序（λ=0.7）→ 游标分页 → 分区推荐 + 排行榜 |
| 风控引擎 | 白名单优先 → 黑名单拦截 → 多层级规则匹配（关键词/正则/频率限制）→ 4 种分流（拦截/隔离/告警/封禁）→ 到期自动解封 |
| 运营后台 | 23 模块全前后端对齐（见下方简表），[完整 API 文档](./docs/API.md) |

> 架构细节、数据流图、算法详述见 [bmad-output/architecture.md](./bmad-output/architecture.md)。

---

## 技术栈

| 层次 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 前端 | Vue 3 + Vite + Element Plus + ECharts | 3.5+ | SPA，路由懒加载，AdminLayout 统一布局 |
| 后端 | Go + Gin + GORM | 1.25 | RESTful API，JWT 鉴权，193 个 Go 源文件 |
| 数据库 | MySQL 8.0 | 8.0+ | 86 模型，GORM AutoMigrate 自动建表 |
| 缓存 | Redis | 7.0+ | 播放计数、弹幕冷却、Token 黑名单、热搜 ZSET |
| 消息队列 | RabbitMQ | 3.12+ | 视频转码异步任务队列 |
| 文件存储 | 阿里云 OSS / 本地 | — | 视频/封面/头像，Docker 卷本地兜底 |
| 实时通信 | WebSocket（gorilla） | — | 弹幕/私信/直播聊天 3 套独立通道 |
| 直播 | SRS + flv.js | 5.0+ | RTMP 推流 → HTTP-FLV 低延迟播放 |
| 视频处理 | FFmpeg | 7.0+ | H.264 转码、封面截帧 |
| 搜索 | Elasticsearch（可选） | 8.x | ik 中文分词全文搜索 |
| 鉴权 | JWT 双 Token | — | 用户 Access(2h)+Refresh(30d)，管理员 Access(2h)+Refresh(3d) |

---

## 数据库设计

86 张数据表，15 个业务模块，GORM AutoMigrate 首次启动自动建表。

### 数据库架构总览

<table>
  <tr>
    <td align="center"><b>核心模块（upper）</b><br><img src="docs/images/db-arch-bento-top.png" alt="数据库架构-上" width="600"/></td>
  </tr>
  <tr>
    <td align="center"><b>扩展模块（lower）</b><br><img src="docs/images/db-arch-bento-bottom.png" alt="数据库架构-下" width="600"/></td>
  </tr>
</table>

### E-R 图

| E-R 图 | 说明 |
|--------|------|
| ![完整 ER 图](docs/images/er-diagram-full.png) | **86 实体 · Crow's Foot 标注 · Figma 风格** — [交互版](docs/cakecake_er_figma-diagram.html) |
| ![扩展模块 ER](docs/images/er-diagram-admin-ext.png) | Admin Extensions 扩展模块 ER — [交互版](docs/AdminER_Diagram.html) |
| [cakecake_er_bento.html](docs/cakecake_er_bento.html) | **Bento 风格** 数据库架构总览 |

### 核心实体（6 张）

| 表名 | 中文名 | 字段数 | 说明 |
|------|--------|--------|------|
| `users` | 用户表 | 23 | JWT 双 Token、bcrypt 密码、经验值等级（Lv1~Lv6）、硬币余额、注销冷静期 |
| `videos` | 视频表 | 29 | H.264 转码状态机、Redis 播放计数 10s 落库 |
| `articles` | 专栏文章表 | 20 | Markdown 正文、标签 JSON |
| `danmakus` | 弹幕表 | 10 | WebSocket 实时推送、5s 冷却 + 敏感词过滤 |
| `comments` | 视频评论表 | 12 | 3 级嵌套、精选模式审核 |
| `admins` | 管理员表 | 8 | 独立 JWT 登录运营后台 |

### 业务模块表分布

| 模块 | 表数 | 包含表 |
|------|:--:|------|
| 视频互动 | 10 | danmakus, comments, comment_likes, comment_dislikes, video_likes, video_coins, video_favorites, favorite_folders, watch_laters, danmaku_likes |
| 文章互动 | 6 | articles, article_comments, article_favorites, article_coins, a_comment_likes, a_comment_dislikes |
| 关注社交 | 4 | user_follows, user_blocks, user_follow_groups, u_follow_group_members |
| 消息通知 | 5 | dm_conversations, dm_messages, dm_participants, notifications, like_notif_mutes |
| 动态系统 | 5 | user_dynamics, user_dynamic_likes, dynamic_comments, d_comment_likes, d_comment_dislikes |
| 直播系统 | 2 | live_rooms, live_warn_templates |
| 历史记录 | 6 | video_view_histories, article_view_histories, live_view_histories, user_search_histories, user_daily_tasks, coin_ledgers |
| 运营基础 | 8 | coin_ledgers, agent_profiles, agent_settings, home_banners, hot_search_ops, hot_search_display_layout, llm_configs, llm_providers, reports |
| 工单风控 | 7 | tickets, ticket_messages, ticket_satisfactions, risk_rules, risk_hit_logs, black_white_lists, risk_rate_counters |
| 版权管理 | 2 | copyright_complaints, counter_notices |
| 数据报表 | 2 | saved_reports, video_daily_stats |
| 客服后台 | 3 | cs_templates, cs_conversations, cs_messages |
| 运维监控 | 6 | task_logs, alert_rules, alert_records, trace_records, cdn_refresh_tasks, oss_lifecycle_rules |
| 配置权限 | 12 | feature_flags, release_records, admin_roles, admin_permissions, role_permissions, admin_role_assignments, admin_login_logs, audit_logs, approval_flows, approval_steps, special_pages, campaigns |
| 模块扩展 | 6 | video_chapters, video_bitrates, subtitles, comment_images, scheduled_publishes, notification_records |

### 用户角色与权限矩阵

| 操作 | 普通用户 | UP 主 | 管理员 | AI 助手 |
|------|:--:|:--:|:--:|:--:|
| 浏览视频/文章 | ✓ | ✓ | ✓ | ✗ |
| 上传视频/文章 | ✓ | ✓ | ✗ | ✗ |
| 发送弹幕 | ✓ | ✓ | ✓ | ✗ |
| 点赞/投币/收藏 | ✓ | ✓ | ✓ | ✗ |
| 关注/拉黑 | ✓ | ✓ | ✓ | ✗ |
| 删除他人评论 | ✗ | ✓（自己视频下） | ✓ | ✗ |
| 审核视频/文章 | ✗ | ✗ | ✓ | ✗ |
| 运营配置 | ✗ | ✗ | ✓ | ✗ |

---

## 运营后台（23 模块）

| # | 模块 | 功能 |
|---|------|------|
| 1 | 数据概览 | 9 张概览卡片 + ECharts 图表 |
| 2 | 首页轮播 | 横幅 CRUD + 排序 + 起止时间 |
| 3 | 热搜运营 | 关键词 pin/block/manual 干预 + 布局排序 |
| 4 | 用户管理 | 用户列表、封禁、信息编辑 |
| 5 | 视频审核 | 审核 publish/reject → 记录审核人 + 时间 |
| 6 | 专栏审核 | 文章内容审核 |
| 7 | 动态管理 | 三表 UNION 统一视图 + 类型过滤 |
| 8 | 评论管理 | 跨 3 表联合查询 + 待审隔离筛选 |
| 9 | 系统设置 | 全局配置管理，同步更新 .env + 内存 |
| 10 | 举报处理 | 多类型举报受理与批量处理 |
| 11 | AI 角色 | Agent Profile CRUD + LLM 提供商管理 |
| 12 | 工单管理 | 提交→处理→对话→满意度→SLA 自动升级 |
| 13 | 风控管理 | 规则引擎（keyword/regex/rate） + 黑白名单 + auto_ban |
| 14 | 版权管理 | 投诉→审核→下架/驳回 + 反通知 |
| 15 | 数据报表 | ECharts 多维度统计 + CSV 导出 |
| 16 | 客服后台 | 模板管理 + 会话管理 + 快捷回复 |
| 17 | 运维监控 | 任务队列/告警/链路追踪/健康检查/CDN/存储 |
| 18 | 配置发布 | Feature Flag（FNV-1a Hash 灰度）+ 版本发布（快照→部署→回滚） |
| 19 | 权限审计 | RBAC 23 权限码 + 审计日志 + 审批流 + 登录日志 |
| 20 | 播放器高级 | 视频章节标记 + 多码率版本管理 |
| 21 | 字幕管理 | 字幕 CRUD + VTT/SRT 格式 |
| 22 | 评论增强 | 图片评论 + 举报 + 排序配置 |
| 23 | Feed 推荐 | 个性化推荐(MMR) + 订阅源 + 排行榜 |

> 完整 API（~380 端点 + 权限码索引 + WS 协议）见 [docs/API.md](./docs/API.md)。

---

## 仓库结构

```
minibili/
├── cmd/mini-bili/             # Go 入口
├── internal/
│   ├── handler/               # 83 个 handler 文件（含 25 个 admin handler）
│   ├── service/               # 19 个 service 文件（业务逻辑层）
│   ├── middleware/             # 横切关注点（认证/授权/追踪）
│   ├── model/                 # 86 个 GORM 模型
│   ├── data/                  # 数据层（AutoMigrate + RBAC seed）
│   ├── worker/                # RabbitMQ 消费者（转码 Worker）
│   └── ws/                    # WebSocket Hub（弹幕/私信/直播）
├── configs/                   # sensitive_words.txt
├── deploy/                    # Nginx、systemd 模板
├── scripts/                   # 工具脚本
├── go.mod                     # module minibili
├── cakecake-vue/
│   └── bilibili-vue/          # Vue 3 + Vite 前端（独立依赖隔离）
├── docs/                      # 截图、ER 图、API 文档、部署手册等
└── bmad-output/               # BMAD 架构分析产出
```

---

## 5 分钟本地联调

### 1. 后端

```bash
cp .env.example .env          # 填写 JWT_SECRET、MYSQL_DSN、REDIS_*、RABBITMQ_URL、OSS_* 等
go mod tidy
go build -o ./bin/mini-bili ./cmd/mini-bili/
./bin/mini-bili               # 默认 :8080；健康检查 GET /api/v1/health
```

MySQL 需先建库（如 `minibili`）；**表由首次启动时 GORM AutoMigrate 自动创建**（86 张表，含全部索引与约束），无需手动执行 SQL。

### 2. 前端

```bash
cd cakecake-vue/bilibili-vue
npm install
cp .env.example .env.local    # 至少 VITE_MINIBILI_API=true
npm run dev                   # http://localhost:8888
```

### 3. 验证

- 首页能打开，接口走 `/api/v1`（Vite 代理到 `127.0.0.1:8080`）
- 登录 / 注册：`#/minibili/login`、`#/minibili/register`
- 无效路径或不存在的视频 → `#/404`

前端详细说明见 [bilibili-vue/README.md](./cakecake-vue/bilibili-vue/README.md)。

---

## Docker 一键部署

### 前置条件

- **Docker** + **Docker Compose** v2
- **Node.js 18+**（仅用于构建前端，一次性）

### 3 步启动

```bash
# 1. 构建前端
cd cakecake-vue/bilibili-vue && npm install && npm run build && cd ../..

# 2. 配置环境变量
cp .env.example .env
# 编辑 .env：只需设置 JWT_SECRET（任意长随机串），其余留空即可

# 3. 启动全部服务
docker compose up -d
```

访问 `http://localhost` 即见首页。管理员默认账号 `admin / change-me-admin`。

### 包含的服务

| 服务 | 端口 | 说明 |
|------|------|------|
| 前端 Nginx | `80` | SPA 静态文件 + API 反代 + WebSocket 代理 + 文件服务 |
| Go 后端 | `8080` | REST API + WebSocket |
| MySQL 8.0 | `3306` | 数据库（首次启动自动建表） |
| Redis 7 | `6379` | 缓存 / Token 存储 |
| RabbitMQ | `5672` (15672 管理后台) | 视频转码队列 |
| SRS | `1935` (RTMP) / `8000` (FLV) | 直播推流/播放 |

### 文件存储说明

本项目**无需任何云存储**即可完整运行。后端默认使用本地文件系统（Docker 卷 `uploads_data`）。配置 `OSS_*` 环境变量后自动切换阿里云 OSS。

### 注意事项

- **直播功能**：主播 OBS 推流地址为 `rtmp://<你的IP>:1935/live/<stream_key>`（stream_key 在"我要开播"页面获取）
- **搜索功能**：Elasticsearch 默认不启动（可选）
- **数据持久化**：`docker compose down` 不会丢失数据（Docker 卷）

---

## 数据流

<table>
  <tr>
    <td align="center"><b>Context Diagram（顶层）</b><br><img src="docs/images/dataflow-context.png" alt="顶层数据流图" width="500"/></td>
    <td align="center"><b>Level-0 FFD（分解）</b><br><img src="docs/images/dataflow-level0.png" alt="0层数据流图" width="500"/></td>
  </tr>
</table>

| 流程 | 详述 |
|------|------|
| 视频上传与转码 | 用户上传 → FFmpeg 转码 → OSS → 发布 |
| 弹幕实时推送 | WebSocket → 5s冷却 + 敏感词 → 广播 |
| 私信 | 创建会话 → INSERT → WS 推送 → 未读计数 |
| 硬币投币 | INSERT coins → UPDATE balance → INSERT ledger（事务） |
| 直播推流 | OBS RTMP → SRS 回调 → flv.js 播放 → WS 聊天 |
| 风控检测 | 白名单→黑名单→keyword/regex/rate→Action 分流 |

> 完整数据流图、架构详图、算法细节见 [bmad-output/architecture.md](./bmad-output/architecture.md)。

---

## HTTP API 约定

- 前缀：`/api/v1`
- 响应：`{ "code": number, "msg": string, "data": object | null }`
- 认证：`Authorization: Bearer <access_token>`
- 运营后台：`/api/admin/*`，独立 admin JWT，RBAC 23 权限码控制

> 完整 API 文档（~380 端点）见 [docs/API.md](./docs/API.md)。

---

## 界面截图

<table>
  <tr>
    <td align="center"><b>首页</b><br><img src="docs/images/homepage.png" alt="首页" width="400"/></td>
    <td align="center"><b>视频播放（含弹幕）</b><br><img src="docs/images/video-player.png" alt="视频播放" width="400"/></td>
  </tr>
  <tr>
    <td align="center"><b>搜索</b><br><img src="docs/images/search.png" alt="搜索" width="400"/></td>
    <td align="center"><b>个人中心</b><br><img src="docs/images/profile.png" alt="个人中心" width="400"/></td>
  </tr>
  <tr>
    <td align="center"><b>个人空间</b><br><img src="docs/images/personal-space.png" alt="个人空间" width="400"/></td>
    <td align="center"><b>动态</b><br><img src="docs/images/dynamic.png" alt="动态" width="400"/></td>
  </tr>
  <tr>
    <td align="center"><b>排行榜</b><br><img src="docs/images/ranking-list.png" alt="排行榜" width="400"/></td>
    <td align="center"><b>运营后台 BI</b><br><img src="docs/images/admin-bi.png" alt="运营后台BI" width="400"/></td>
  </tr>
</table>

---

## 文档索引

| 文档 | 内容 | 对象 |
|------|------|------|
| **本文** | 项目概述、启动、部署 | 所有人 |
| [docs/API.md](./docs/API.md) | **完整 API 文档**（~380 端点、权限码、WS 协议、预置账号） | 开发 |
| [bmad-output/architecture.md](./bmad-output/architecture.md) | **架构分析 v3.0**（ADR、FR/NFR 矩阵、数据流、算法详述） | 架构师/开发 |
| [SPEC.md](./SPEC.md) | 功能与验收规格 | 开发 |
| [Rule.md](./Rule.md) | 工程红线 | 开发 |
| [Skill.md](./Skill.md) | 标准操作（迁移、Token、WS 等） | 开发 |
| [cakecake-vue/bilibili-vue/README.md](./cakecake-vue/bilibili-vue/README.md) | 前端安装/构建 | 前端 |
| [deploy/DEPLOY.md](./deploy/DEPLOY.md) | 生产部署 | 运维 |
| [docs/ai-gateway.md](./docs/ai-gateway.md) | AI 助手配置 | 运维 |
| [docs/manual-video-ingest.md](./docs/manual-video-ingest.md) | 关闭上传时的视频入库 | 运维 |
| [docs/cakecake_er_figma-diagram.html](./docs/cakecake_er_figma-diagram.html) | ER 图交互版 | 开发 |

---

## 其他

- 勿提交 `.env`、密钥与数据库密码。
- 实现与 SPEC / Rule 冲突时，以 SPEC / Rule 为准。
- 后端基于 [earthcake2233/cakecake](https://github.com/earthcake2233) 二次开发，遵循开源协议。
