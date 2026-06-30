# Cakecake API 文档

> **版本:** v2.1 | **生成日期:** 2026-06-30 | **来源:** `internal/handler/router.go`
>
> 统一响应格式: `{ "code": number, "msg": string, "data": object | null }`
> 认证方式: `Authorization: Bearer <token>`
> 所有端点前缀: `/api/v1`

---

## 目录

1. [约定](#约定)
2. [公开端点（无需认证）](#1-公开端点)
3. [用户认证](#2-用户认证)
4. [用户端 API（需要 JWT）](#3-用户端-api)
5. [管理员认证](#4-管理员认证)
6. [运营后台 API（需要 Admin JWT）](#5-运营后台-api)
7. [WebSocket 端点](#6-websocket-端点)
8. [直播回调](#7-直播回调)
9. [NocoBase 桥接](#8-nocobase-桥接)

---

## 约定

| 项目 | 规定 |
|------|------|
| 响应格式 | `{ "code": 0, "msg": "ok", "data": {} }` — code=0 成功 |
| 空数据 | `data: null`（非 `data: {}`） |
| 认证头 | `Authorization: Bearer <access_token>` |
| HTTP 方法 | GET=查询 POST=创建 PUT=全量更新 PATCH=部分更新 DELETE=删除 |
| URL 风格 | 复数资源名，kebab-case 多词路径 |
| 时间格式 | RFC 3339 (`2026-06-30T18:00:00+08:00`) |
| 分页 | `?page=1&page_size=20`，返回 `{ items, total, page, page_size }` |
| 管理端前缀 | `/api/v1/admin` |
| 错误码 | 见 [错误码表](#错误码表) |

### 错误码表

| code | 含义 | HTTP |
|------|------|------|
| 0 | 成功 | 200 |
| 40001 | 参数校验失败 | 400 |
| 40002 | 资源已存在 | 409 |
| 40100 | 未认证 / Token 过期 | 401 |
| 40300 | 无权限 | 403 |
| 40400 | 资源不存在 | 404 |
| 42900 | 频率限制 | 429 |
| 50000 | 服务器内部错误 | 500 |

---

## 1. 公开端点

### 健康检查

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 服务器健康检查 |

### 视频

| 方法 | 路径 | 说明 | 备注 |
|------|------|------|------|
| GET | `/videos` | 公开视频列表 | 支持 zone/period/sort/page |
| GET | `/videos/:id` | 视频详情 | OptionalJWT（登录用户获取个性化数据） |
| GET | `/videos/:id/comments` | 视频评论列表 | 3 级嵌套回复，OptionalJWT |
| GET | `/videos/:id/chapters` | 视频章节列表 | |
| GET | `/videos/:id/bitrates` | 多码率版本 | |
| GET | `/videos/:id/subtitles` | 字幕列表 | |
| GET | `/videos/:id/subtitles/:subtitleId` | 字幕详情 | |

### 专栏

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/articles` | 文章列表 |
| GET | `/articles/:id` | 文章详情 |
| GET | `/articles/:id/comments` | 文章评论列表 |

### 个人空间（公开）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/space/:userId` | 用户公开信息 |
| GET | `/space/:userId/videos` | 用户发布的视频 |
| GET | `/space/:userId/articles` | 用户发布的文章 |
| GET | `/space/:userId/dynamics` | 用户发布的动态 |
| GET | `/space/:userId/favorites` | 用户收藏夹 |
| GET | `/space/:userId/favorite-folders` | 用户收藏夹列表 |
| GET | `/space/:userId/recent-coins` | 最近投币视频 |
| GET | `/space/:userId/following` | 关注列表 |
| GET | `/space/:userId/followers` | 粉丝列表 |
| GET | `/space/:userId/article-favorites` | 文章收藏 |

### 首页

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/stats/home` | 首页统计数据 |
| GET | `/home-banners` | 首页轮播横幅 |
| GET | `/hot-search` | 热搜列表 |
| GET | `/online` | 在线人数 |

### 搜索

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/search` | 全文搜索 `?q=X&type=video\|article\|user` |
| GET | `/search/suggest` | 搜索建议 |

### 推荐 & 排行榜

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/feed/recommendation` | 个性化推荐流（MMR 重排序） |
| GET | `/leaderboard` | 排行榜 `?by=play\|coin\|fav&period=week\|month\|all` |
| GET | `/zones/:zone/recommendation` | 分区推荐 |

### Feature Flag 查询

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/config/feature-flags/:key` | 查询功能开关状态 |

### 专题页面

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/specials` | 公开专题页列表 |
| GET | `/specials/:slug` | 专题页详情 |

### 动态

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/user-dynamics/:id` | 动态详情 |
| GET | `/user-dynamics/:id/comments` | 动态评论列表 |

### 直播（公开）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/live/rooms` | 直播广场列表 |
| GET | `/live/room/my` | 我的直播间（OptionalJWT） |
| GET | `/live/room/:id` | 直播间详情 |

---

## 2. 用户认证

| 方法 | 路径 | 请求体 | 响应 |
|------|------|--------|------|
| POST | `/users` | `{ username, password, nickname? }` | `{ user, access_token, refresh_token }` |
| POST | `/auth/login` | `{ username, password }` | `{ user, access_token, refresh_token }` |
| POST | `/auth/refresh` | `{ refresh_token }` | `{ access_token, refresh_token }` |

**Token 规格:**
- Access Token: 2h
- Refresh Token: **30d**

---

## 3. 用户端 API

> 全部需要 `Authorization: Bearer <access_token>`（Access Token）

### 个人中心

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/me` | 获取个人信息 |
| PUT | `/users/me` | 修改用户名 |
| PUT | `/users/me/profile` | 修改个人资料（昵称/简介/生日/性别） |
| PUT | `/users/me/announcement` | 修改公告栏 |
| PUT | `/users/me/password` | 修改密码 `{ old_password, new_password }` |
| POST | `/users/me/avatar` | 上传头像（multipart） |
| GET | `/users/me/space-privacy` | 获取空间隐私设置 |
| PUT | `/users/me/space-privacy` | 更新空间隐私设置 |

### 账号注销

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/users/me/deletion/request` | 申请注销（7 天冷静期） |
| POST | `/users/me/deletion/revoke` | 撤销注销申请 |

### 视频操作

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/videos` | 上传视频（multipart, ≤500MB/≤30min） |
| GET | `/users/me/videos` | 我的视频列表 |
| PUT | `/videos/:id` | 修改视频信息 |
| DELETE | `/videos/:id` | 删除视频 |
| PUT | `/videos/:id/cover` | 更换封面 |
| PATCH | `/videos/:id/playback` | 更新播放进度 `{ position_sec }` |
| POST | `/videos/:id/replace-media` | 替换视频文件 |

### 视频草稿

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/videos/draft` | 保存草稿 |
| PUT | `/videos/:id/draft` | 更新草稿 |
| POST | `/videos/:id/draft` | 更新草稿（POST 别名） |
| POST | `/videos/:id/publish` | 发布草稿 |
| GET | `/users/me/videos/:id/draft-source` | 获取视频源信息 |

### 视频互动

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/videos/:id/like` | 点赞/取消 |
| POST | `/videos/:id/favorite` | 收藏/取消 |
| GET | `/videos/:id/favorite-picker` | 获取收藏夹选择器 |
| PUT | `/videos/:id/favorite-folders` | 设置收藏夹 |
| PUT | `/videos/:id/favorite-folders/move` | 移动收藏夹 |
| POST | `/videos/:id/favorite-folders/:folderId` | 添加到指定收藏夹 |
| DELETE | `/videos/:id/favorite-folders/:folderId` | 从收藏夹移除 |
| POST | `/videos/:id/coin` | 投币 `{ count: 1\|2 }` |
| POST | `/videos/:id/watch-later` | 稍后再看/取消 |

### 收藏夹管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/me/favorites` | 收藏列表 |
| GET | `/users/me/favorite-folders` | 收藏夹列表 |
| POST | `/users/me/favorite-folders` | 创建收藏夹 `{ name, is_public }` |
| PUT | `/users/me/favorite-folders/:folderId` | 修改收藏夹 |
| DELETE | `/users/me/favorite-folders/:folderId` | 删除收藏夹 |
| DELETE | `/users/me/favorite-folders/:folderId/invalid-favorites` | 清理失效收藏 |
| POST | `/users/me/favorite-folders/:folderId/batch-remove` | 批量移除 `{ video_ids }` |

### 稍后再看

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/me/watch-later` | 稍后再看列表 |
| DELETE | `/users/me/watch-later` | 清空 |
| DELETE | `/users/me/watch-later/watched` | 清除已看 |
| POST | `/users/me/watch-later/:id/watched` | 标记已看 |

### 弹幕

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/videos/:id/danmaku` | 发送弹幕 `{ content, position_sec, color?, type?, mode? }` |
| DELETE | `/danmakus/:id` | 删除弹幕（本人/UP主） |
| POST | `/danmakus/:id/like` | 弹幕点赞 |

### 评论

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/videos/:id/comments` | 发表评论 `{ content, parent_id?, root_id? }` |
| DELETE | `/comments/:id` | 删除评论 |
| POST | `/comments/:id/pin` | 置顶评论（UP主） |
| POST | `/comments/:id/approve` | 通过评论审核（UP主） |
| POST | `/comments/:id/ignore-curated` | 忽略精选 |
| POST | `/comments/:id/like` | 点赞 |
| POST | `/comments/:id/dislike` | 反对 |

### 评论增强（图片评论等）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/videos/:id/comments-with-image` | 带图评论 `{ content, images }` |
| POST | `/comments/:id/images` | 上传评论图片 |
| DELETE | `/comments/:id/images/:imageId` | 删除评论图片 |
| GET | `/comments/:id/images` | 评论图片列表 |
| GET | `/videos/:id/comments/config` | 评论排序配置 |
| POST | `/comments/:id/report` | 举报评论 |

### 文章

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/articles` | 发布文章 `{ title, content, category, tags?, cover_url? }` |
| GET | `/users/me/articles` | 我的文章列表 |
| GET | `/users/me/articles/:id` | 我的文章详情 |
| PUT | `/users/me/articles/:id` | 更新文章 |
| DELETE | `/users/me/articles/:id` | 删除文章 |
| PUT | `/users/me/articles/:id/cover` | 更换文章封面 |
| PATCH | `/users/me/articles/:id/playback` | 更新阅读进度 |
| POST | `/articles/:id/view` | 记录阅读 |
| POST | `/articles/:id/coin` | 投币 |
| POST | `/articles/:id/favorite` | 收藏/取消 |

### 文章评论

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/articles/:id/comments` | 发表评论 |
| DELETE | `/article-comments/:id` | 删除评论 |
| POST | `/article-comments/:id/like` | 点赞 |
| POST | `/article-comments/:id/dislike` | 反对 |
| POST | `/article-comments/:id/pin` | 置顶 |
| POST | `/article-comments/:id/approve` | 审核通过 |
| POST | `/article-comments/:id/ignore-curated` | 忽略精选 |

### 动态

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/me/dynamics` | 我的动态列表 |
| POST | `/users/me/dynamics` | 发布图文动态 `{ content, images?, type: "image"\|"text" }` |
| PUT | `/users/me/dynamics/:id` | 编辑动态 |
| DELETE | `/users/me/dynamics/:id` | 删除动态 |
| PATCH | `/users/me/dynamics/:id/playback` | 更新播放进度 |
| POST | `/user-dynamics/:id/like` | 点赞/取消 |

### 动态评论

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/user-dynamics/:id/comments` | 发表评论 |
| DELETE | `/dynamic-comments/:id` | 删除 |
| POST | `/dynamic-comments/:id/like` | 点赞 |
| POST | `/dynamic-comments/:id/dislike` | 反对 |
| POST | `/dynamic-comments/:id/approve` | 审核通过 |
| POST | `/dynamic-comments/:id/ignore-curated` | 忽略精选 |

### 关注 & 社交

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/users/:userId/follow` | 关注/取关 toggle |
| POST | `/users/:userId/block` | 拉黑/解除 toggle |

### 关注分组

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/me/follow-groups` | 分组列表 |
| POST | `/users/me/follow-groups` | 创建分组 `{ name }` |
| PUT | `/users/me/follow-groups/:groupId` | 修改分组 |
| DELETE | `/users/me/follow-groups/:groupId` | 删除分组 |
| GET | `/users/me/following/:followeeId/groups` | 查询某关注者的分组 |
| POST | `/users/me/follow-groups/:groupId/members` | 添加成员 |
| DELETE | `/users/me/follow-groups/:groupId/members/:followeeId` | 移除成员 |

### 私信

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/dm/conversations` | 会话列表 |
| POST | `/dm/conversations` | 创建会话 `{ participant_id }` |
| DELETE | `/dm/conversations/:id` | 删除会话 |
| POST | `/dm/conversations/:id/reset` | 重置 AI 对话上下文 |
| PATCH | `/dm/conversations/:id/settings` | 修改会话设置（置顶/免打扰） |
| GET | `/dm/conversations/:id/messages` | 消息列表 `?page=&page_size=` |
| POST | `/dm/conversations/:id/messages` | 发送消息 `{ content, content_type? }` |

### 通知

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/notifications/unread-summary` | 未读通知汇总 |
| GET | `/notifications` | 通知列表 `?category=&page=&page_size=` |
| GET | `/notifications/:id/like-likers` | 点赞人列表 |
| PATCH | `/notifications/read-by-category` | 按分类标记已读 `{ category }` |
| PATCH | `/notifications/read-batch` | 批量已读 `{ ids }` |
| PATCH | `/notifications/:id/read` | 标记单条已读 |
| POST | `/notifications/:id/mute-likes` | 免打扰 |
| POST | `/notifications/:id/comment-like` | 点赞通知评论 |
| POST | `/notifications/:id/comment-reply` | 回复通知评论 |
| DELETE | `/notifications/:id` | 删除通知 |

### 观看历史

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/me/view-history` | 历史记录列表（综合） |
| DELETE | `/users/me/view-history` | 清空所有历史 |
| DELETE | `/users/me/view-history/:videoId` | 删除视频观看记录 |
| DELETE | `/users/me/view-history/articles/:articleId` | 删除文章阅读记录 |
| DELETE | `/users/me/view-history/live/:liveRoomId` | 删除直播观看记录 |
| GET | `/users/me/view-history/settings` | 历史设置 |
| PUT | `/users/me/view-history/settings` | 更新历史设置 |
| POST | `/videos/:id/view-history` | 记录视频观看 `{ position_sec }` |

### 搜索历史

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/me/search-history` | 搜索历史 |
| PUT | `/users/me/search-history` | 更新搜索历史 |
| POST | `/users/me/search-history` | 添加搜索词 |

### 每日任务 & 硬币

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/users/me/daily-rewards` | 每日任务状态 |
| POST | `/users/me/daily-rewards/watch` | 完成观看任务 |
| GET | `/users/me/coin-ledger` | 硬币流水 |

### 创作者中心

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/videos/:id/schedule` | 定时发布 `{ publish_at }` |
| DELETE | `/videos/:id/schedule` | 取消定时发布 |
| GET | `/users/me/creator/stats` | 创作者总览统计 |
| GET | `/users/me/creator/video-stats` | 视频维度统计 |
| GET | `/users/me/creator/comments` | 所有视频下的评论 |
| GET | `/users/me/creator/danmakus` | 所有视频下的弹幕 |
| POST | `/videos/:id/chapters` | 创建章节（创作者） |
| DELETE | `/videos/:id/chapters/:chapterId` | 删除章节（创作者） |

### 字幕（用户端）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/videos/:id/subtitles` | 上传字幕文件（VTT/SRT） |
| DELETE | `/videos/:id/subtitles/:subtitleId` | 删除字幕 |
| POST | `/videos/:id/subtitles/asr` | 请求 ASR 自动转写 |

### Feed 订阅

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/feed/subscription` | 关注 UP 主的 Feed（纯时间序） |

### 举报

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/reports` | 提交举报 `{ target_type, target_id, reason, detail? }` |

### 工单（用户端）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/tickets` | 提交工单 `{ title, content, category }` |
| GET | `/users/me/tickets` | 我的工单列表 |
| GET | `/users/me/tickets/:id` | 工单详情 |
| POST | `/users/me/tickets/:id/messages` | 追加工单消息 |
| POST | `/users/me/tickets/:id/appeal` | 申诉 |
| POST | `/tickets/:id/satisfaction` | 工单满意度评价 |

### 版权

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/copyright/complaints` | 提交版权投诉 |
| GET | `/users/me/copyright/complaints` | 我的投诉列表 |

### 客服

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/cs/conversations` | 发起客服会话 |
| GET | `/users/me/cs/conversations` | 我的客服会话 |
| GET | `/users/me/cs/conversations/:id` | 会话详情 |
| POST | `/users/me/cs/conversations/:id/messages` | 发送消息 |

### 直播（用户端）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/live/room/create` | 创建直播间 |
| PUT | `/live/room/:id` | 更新直播设置 `{ title, cover_url }` |
| POST | `/live/room/:id/regenerate-key` | 重新生成推流密钥 |
| POST | `/live/room/:id/start` | 开播 |
| POST | `/live/room/:id/end` | 下播 |
| POST | `/live/room/:id/cover` | 上传直播封面 |

---

## 4. 管理员认证

| 方法 | 路径 | 请求体 | 响应 |
|------|------|--------|------|
| POST | `/admin/auth/login` | `{ username, password }` | `{ admin, access_token, refresh_token }` |
| POST | `/admin/auth/refresh` | `{ refresh_token }` | `{ access_token, refresh_token }` |

**Token 规格:**
- Access Token: 2h
- Refresh Token: **3d**

---

## 5. 运营后台 API

> 全部需要 `Authorization: Bearer <admin_access_token>`
> 🔒 标记的端点需要特定 RBAC 权限（中间件校验 `resource:action`）

### 5.1 个人

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/me` | — | 当前管理员信息 |
| GET | `/admin/rbac/me/permissions` | — | 当前管理员权限列表 |

---

### 5.2 数据概览

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/dashboard` | — | 核心指标概览 |

### BI 报表

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/bi/summary` | — | BI 总览（9 张卡片） |
| GET | `/admin/bi/article-stats` | — | 文章统计 |
| GET | `/admin/bi/engagement-stats` | — | 互动统计 |
| GET | `/admin/bi/manuscript-stats` | — | 视频稿件统计 |
| GET | `/admin/bi/zone-stats` | — | 分区统计 |
| GET | `/admin/bi/creator-stats` | — | 创作者统计 |
| GET | `/admin/bi/time-series` | — | 时序数据 |
| GET | `/admin/bi/reports` | — | 已保存报表列表 |
| POST | `/admin/bi/reports` | 🔒 `dashboard:export` | 保存报表配置 |
| DELETE | `/admin/bi/reports/:id` | 🔒 `dashboard:export` | 删除报表 |
| POST | `/admin/bi/export` | 🔒 `dashboard:export` | 导出 CSV |

---

### 5.3 用户管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/users` | — | 用户列表 `?q=&page=&page_size=` |
| GET | `/admin/users/:id` | — | 用户详情 |
| GET | `/admin/users/:id/violations` | — | 用户违规记录 |
| POST | `/admin/users/:id/ban` | 🔒 `user:ban` | 封禁用户 |
| POST | `/admin/users/:id/unban` | 🔒 `user:ban` | 解封 |
| POST | `/admin/users/:id/delete` | 🔒 `user:ban` | 删除用户 |

---

### 5.4 视频审核

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/videos` | — | 视频列表 `?status=&q=&page=` |
| GET | `/admin/videos/:id` | — | 视频详情 |
| POST | `/admin/videos/:id/approve` | 🔒 `video:approve` | 审核通过 |
| POST | `/admin/videos/:id/reject` | 🔒 `video:approve` | 驳回 |
| POST | `/admin/videos/:id/delete` | 🔒 `video:approve` | 删除 |
| DELETE | `/admin/videos/:id` | 🔒 `video:approve` | 删除（DELETE 别名） |
| POST | `/admin/videos/batch-approve` | 🔒 `video:approve` | 批量通过 `{ ids }` |

---

### 5.5 专栏审核

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/articles` | — | 文章列表 |
| GET | `/admin/articles/:id` | — | 文章详情 |
| POST | `/admin/articles/:id/approve` | 🔒 `article:approve` | 通过 |
| POST | `/admin/articles/:id/reject` | 🔒 `article:approve` | 驳回 |
| POST | `/admin/articles/:id/delete` | 🔒 `article:approve` | 删除 |
| DELETE | `/admin/articles/:id` | 🔒 `article:approve` | 删除 |

---

### 5.6 直播管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/live/rooms` | 🔒 `live:manage` | 所有直播间 |
| GET | `/admin/live/room/:id` | 🔒 `live:manage` | 房间详情 |
| POST | `/admin/live/room/:id/ban` | 🔒 `live:manage` | 封禁 |
| POST | `/admin/live/room/:id/unban` | 🔒 `live:manage` | 解封 |
| POST | `/admin/live/room/:id/warn` | 🔒 `live:manage` | 发送警告 |
| DELETE | `/admin/live/room/:id` | 🔒 `live:manage` | 删除房间 |
| GET | `/admin/live/warn-templates` | 🔒 `live:manage` | 警告模板列表 |
| POST | `/admin/live/warn-templates` | 🔒 `live:manage` | 创建模板 |
| PUT | `/admin/live/warn-templates/:id` | 🔒 `live:manage` | 更新模板 |
| DELETE | `/admin/live/warn-templates/:id` | 🔒 `live:manage` | 删除模板 |

---

### 5.7 动态管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/dynamics` | — | 动态列表 |
| GET | `/admin/dynamics/unified` | — | **统一视图**（三表 UNION: videos + articles + user_dynamics） |
| GET | `/admin/dynamics/:id` | — | 动态详情 |
| POST | `/admin/dynamics/:id/delete` | 🔒 `dynamic:manage` | 删除 |
| DELETE | `/admin/dynamics/:id` | 🔒 `dynamic:manage` | 删除 |

---

### 5.8 评论管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/comments` | — | 评论列表（跨 3 表 `?status=pending\|approved`) |
| GET | `/admin/comments/:id` | — | 评论详情 |
| POST | `/admin/comments/:id/delete` | 🔒 `comment:delete` | 删除 |
| DELETE | `/admin/comments/:id` | 🔒 `comment:delete` | 删除 |
| GET | `/admin/comment-reports` | 🔒 `comment:delete` | 评论举报列表 |
| POST | `/admin/comment-reports/:id/handle` | 🔒 `comment:delete` | 处理举报 |

---

### 5.9 举报处理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/reports` | — | 举报列表 |
| POST | `/admin/reports/:id/handle` | 🔒 `ticket:handle` | 处理 `{ action, note? }` |
| POST | `/admin/reports/batch` | 🔒 `ticket:handle` | 批量处理 `{ ids, action }` |
| DELETE | `/admin/reports/:id` | 🔒 `ticket:handle` | 删除举报 |

---

### 5.10 工单管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/tickets` | 🔒 `ticket:handle` | 工单列表 |
| GET | `/admin/tickets/:id` | 🔒 `ticket:handle` | 工单详情 |
| POST | `/admin/tickets/:id/assign` | 🔒 `ticket:handle` | 分配处理人 |
| POST | `/admin/tickets/:id/status` | 🔒 `ticket:handle` | 更新状态 |
| POST | `/admin/tickets/:id/messages` | 🔒 `ticket:handle` | 添加回复 |
| POST | `/admin/tickets/:id/auto-assign` | 🔒 `ticket:handle` | 自动分配 |
| POST | `/admin/tickets/:id/close` | 🔒 `ticket:handle` | 关闭工单 |
| POST | `/admin/tickets/:id/reopen` | 🔒 `ticket:handle` | 重新打开 |
| GET | `/admin/tickets/satisfaction-stats` | 🔒 `ticket:handle` | 满意度统计 |
| GET | `/admin/tickets/stats` | 🔒 `ticket:handle` | 工单统计 |

---

### 5.11 风控管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/risk/rules` | 🔒 `risk:manage` | 规则列表 |
| POST | `/admin/risk/rules` | 🔒 `risk:manage` | 创建规则 `{ name, type, pattern, action, priority }` |
| PUT | `/admin/risk/rules/:id` | 🔒 `risk:manage` | 更新规则 |
| DELETE | `/admin/risk/rules/:id` | 🔒 `risk:manage` | 删除规则 |
| POST | `/admin/risk/rules/:id/toggle` | 🔒 `risk:manage` | 启用/禁用 |
| GET | `/admin/risk/bw-list` | 🔒 `risk:manage` | 黑白名单 |
| POST | `/admin/risk/bw-list` | 🔒 `risk:manage` | 添加名单 `{ user_id, type, expires_at? }` |
| PUT | `/admin/risk/bw-list/:id` | 🔒 `risk:manage` | 更新名单 |
| DELETE | `/admin/risk/bw-list/:id` | 🔒 `risk:manage` | 删除名单 |
| GET | `/admin/risk/hits` | 🔒 `risk:manage` | 命中日志 |
| GET | `/admin/risk/stats` | 🔒 `risk:manage` | 风控统计 |

---

### 5.12 版权管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/copyright/complaints` | 🔒 `copyright:handle` | 投诉列表 |
| GET | `/admin/copyright/complaints/:id` | 🔒 `copyright:handle` | 投诉详情 |
| POST | `/admin/copyright/complaints/:id/accept` | 🔒 `copyright:handle` | 受理 |
| POST | `/admin/copyright/complaints/:id/reject` | 🔒 `copyright:handle` | 驳回 |
| POST | `/admin/copyright/complaints/:id/takedown` | 🔒 `copyright:handle` | 下架内容 |
| POST | `/admin/copyright/complaints/:id/restore` | 🔒 `copyright:handle` | 恢复内容 |

---

### 5.13 客服后台

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/cs/conversations` | 🔒 `cs:manage` | 会话列表 |
| GET | `/admin/cs/conversations/:id` | 🔒 `cs:manage` | 会话详情 |
| POST | `/admin/cs/conversations/:id/assign` | 🔒 `cs:manage` | 分配客服 |
| POST | `/admin/cs/conversations/:id/messages` | 🔒 `cs:manage` | 发送消息 |
| POST | `/admin/cs/conversations/:id/close` | 🔒 `cs:manage` | 关闭会话 |
| GET | `/admin/cs/templates` | 🔒 `cs:manage` | 快捷回复模板 |
| POST | `/admin/cs/templates` | 🔒 `cs:manage` | 创建模板 |
| PUT | `/admin/cs/templates/:id` | 🔒 `cs:manage` | 更新模板 |
| DELETE | `/admin/cs/templates/:id` | 🔒 `cs:manage` | 删除模板 |

---

### 5.14 运维监控

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/ops/tasks` | 🔒 `ops:manage` | 任务队列日志 |
| POST | `/admin/ops/tasks/:id/retry` | 🔒 `ops:manage` | 重试任务 |
| GET | `/admin/ops/queue-stats` | 🔒 `ops:manage` | 队列统计 |
| GET | `/admin/ops/health` | 🔒 `ops:manage` | **系统健康检查**（items[] 格式） |
| GET | `/admin/ops/traces` | 🔒 `ops:manage` | 链路追踪列表 |
| GET | `/admin/ops/traces/:id` | 🔒 `ops:manage` | 追踪详情 |
| POST | `/admin/ops/sync/trigger` | 🔒 `ops:manage` | 触发 ES/播放量同步 |

**告警**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/ops/alert-rules` | 🔒 `ops:manage` | 告警规则列表 |
| POST | `/admin/ops/alert-rules` | 🔒 `ops:manage` | 创建告警规则 |
| PUT | `/admin/ops/alert-rules/:id` | 🔒 `ops:manage` | 更新规则 |
| DELETE | `/admin/ops/alert-rules/:id` | 🔒 `ops:manage` | 删除规则 |
| POST | `/admin/ops/alert-rules/:id/toggle` | 🔒 `ops:manage` | 启用/禁用 |
| GET | `/admin/ops/alert-records` | 🔒 `ops:manage` | 告警记录 |
| POST | `/admin/ops/alert-records/:id/ack` | 🔒 `ops:manage` | 确认告警 |
| POST | `/admin/ops/alerts/evaluate` | 🔒 `ops:manage` | 立即评估告警 |

**CDN & 存储**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| POST | `/admin/ops/cdn/refresh` | 🔒 `ops:manage` | 创建 CDN 刷新任务 |
| GET | `/admin/ops/cdn/refresh` | 🔒 `ops:manage` | CDN 任务列表 |
| GET | `/admin/ops/storage/lifecycle-rules` | 🔒 `ops:manage` | 存储生命周期规则 |
| POST | `/admin/ops/storage/lifecycle-rules` | 🔒 `ops:manage` | 创建规则 |
| PUT | `/admin/ops/storage/lifecycle-rules/:id` | 🔒 `ops:manage` | 更新规则 |
| DELETE | `/admin/ops/storage/lifecycle-rules/:id` | 🔒 `ops:manage` | 删除规则 |

---

### 5.15 配置发布

**Feature Flag**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/config/feature-flags` | 🔒 `config:manage` | Flag 列表 |
| POST | `/admin/config/feature-flags` | 🔒 `config:manage` | 创建 Flag `{ key, description, rollout_pct, whitelist }` |
| PUT | `/admin/config/feature-flags/:id` | 🔒 `config:manage` | 更新 Flag |
| DELETE | `/admin/config/feature-flags/:id` | 🔒 `config:manage` | 删除 Flag |
| POST | `/admin/config/feature-flags/:id/toggle` | 🔒 `config:manage` | 启停 |

**版本发布**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/config/releases` | 🔒 `config:manage` | 发布记录列表 |
| POST | `/admin/config/releases` | 🔒 `config:manage` | 新建发布（自动快照） |
| POST | `/admin/config/releases/:id/deploy` | 🔒 `config:manage` | 部署上线 |
| GET | `/admin/config/releases/:id/export` | 🔒 `config:manage` | 下载快照 JSON |
| GET | `/admin/config/releases/:id/snapshot` | 🔒 `config:manage` | 在线查看快照 |
| POST | `/admin/config/releases/:id/rollback` | 🔒 `config:manage` | 回滚 |

**配置导出**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/config/export` | 🔒 `config:manage` | 导出当前配置 |

---

### 5.16 权限审计

**角色管理**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/rbac/roles` | 🔒 `rbac:manage` | 角色列表 |
| POST | `/admin/rbac/roles` | 🔒 `rbac:manage` | 创建角色 |
| PUT | `/admin/rbac/roles/:id` | 🔒 `rbac:manage` | 更新角色 |
| GET | `/admin/rbac/roles/:id` | 🔒 `rbac:manage` | 角色详情 |
| DELETE | `/admin/rbac/roles/:id` | 🔒 `rbac:manage` | 删除角色 |

**权限管理**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/rbac/permissions` | 🔒 `rbac:manage` | 权限码列表 |
| GET | `/admin/rbac/roles/:id/permissions` | 🔒 `rbac:manage` | 角色权限 |
| POST | `/admin/rbac/roles/:id/permissions` | 🔒 `rbac:manage` | 分配权限 |

**管理员管理**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/rbac/admins` | 🔒 `rbac:manage` | 管理员列表 |
| POST | `/admin/rbac/admins` | 🔒 `rbac:manage` | 创建管理员 |
| GET | `/admin/rbac/admins/:adminId/role` | 🔒 `rbac:manage` | 管理员角色 |
| POST | `/admin/rbac/admins/:adminId/role` | 🔒 `rbac:manage` | 分配角色 |

**审计 & 日志**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/rbac/audit-logs` | 🔒 `rbac:manage` | 审计日志列表 |
| GET | `/admin/rbac/audit-logs/:id` | 🔒 `rbac:manage` | 审计详情 |
| GET | `/admin/rbac/login-logs` | 🔒 `rbac:manage` | 登录日志 |

**审批流**

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| POST | `/admin/rbac/approval-flows` | 🔒 `rbac:manage` | 创建审批流 |
| POST | `/admin/rbac/approval-flows/:id/approve` | 🔒 `rbac:manage` | 通过某步骤 |
| POST | `/admin/rbac/approval-flows/:id/reject` | 🔒 `rbac:manage` | 驳回某步骤 |
| GET | `/admin/rbac/approval-flows` | 🔒 `rbac:manage` | 审批流列表 |

---

### 5.17 轮播横幅

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/home-banners` | — | 横幅列表 |
| POST | `/admin/home-banners` | 🔒 `banner:manage` | 创建 `{ image_url, link_url?, sort, start_at?, end_at? }` |
| POST | `/admin/home-banners/upload-image` | 🔒 `banner:manage` | 上传图片 |
| POST | `/admin/home-banners/:id/image` | 🔒 `banner:manage` | 更新图片 |
| PUT | `/admin/home-banners/:id` | 🔒 `banner:manage` | 更新 |
| DELETE | `/admin/home-banners/:id` | 🔒 `banner:manage` | 删除 |

---

### 5.18 热搜运营

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/hot-search/ops` | 🔒 `hotsearch:manage` | 运营干预列表 |
| GET | `/admin/hot-search/dashboard` | 🔒 `hotsearch:manage` | 热搜仪表盘 |
| GET | `/admin/hot-search/preview` | 🔒 `hotsearch:manage` | 预览热搜 |
| POST | `/admin/hot-search/ops` | 🔒 `hotsearch:manage` | 创建干预项 |
| POST | `/admin/hot-search/quick-op` | 🔒 `hotsearch:manage` | 快捷操作（pin/block） |
| POST | `/admin/hot-search/reorder` | 🔒 `hotsearch:manage` | 重新排序 |
| POST | `/admin/hot-search/display-order/reset` | 🔒 `hotsearch:manage` | 重置排序 |
| POST | `/admin/hot-search/redis/remove` | 🔒 `hotsearch:manage` | 从 Redis 移除热词 |
| POST | `/admin/hot-search/redis/boost` | 🔒 `hotsearch:manage` | 加权提升热词 |
| PUT | `/admin/hot-search/ops/:id` | 🔒 `hotsearch:manage` | 更新干预 |
| DELETE | `/admin/hot-search/ops/:id` | 🔒 `hotsearch:manage` | 删除干预 |

---

### 5.19 AI 角色

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/agent-settings` | 🔒 `agent:manage` | 全局 Agent 配置 |
| PUT | `/admin/agent-settings` | 🔒 `agent:manage` | 更新全局配置 |
| POST | `/admin/agent-settings/avatar` | 🔒 `agent:manage` | 上传 Agent 头像 |
| GET | `/admin/agent-profiles` | 🔒 `agent:manage` | 角色列表 |
| POST | `/admin/agent-profiles` | 🔒 `agent:manage` | 创建角色 `{ name, prompt, welcome_message }` |
| PUT | `/admin/agent-profiles/:id` | 🔒 `agent:manage` | 更新角色 |
| DELETE | `/admin/agent-profiles/:id` | 🔒 `agent:manage` | 删除角色 |
| POST | `/admin/agent-profiles/:id/avatar` | 🔒 `agent:manage` | 上传角色头像 |

---

### 5.20 系统设置 & LLM 配置

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/settings` | 🔒 `setting:manage` | 系统设置 |
| PUT | `/admin/settings` | 🔒 `setting:manage` | 更新设置（同步写入 .env） |
| GET | `/admin/llm-config` | 🔒 `setting:manage` | LLM 全局配置 |
| PUT | `/admin/llm-config` | 🔒 `setting:manage` | 更新 LLM 配置 |
| GET | `/admin/llm-config/providers` | 🔒 `setting:manage` | LLM 提供商列表 |
| POST | `/admin/llm-config/providers` | 🔒 `setting:manage` | 添加提供商 |
| PUT | `/admin/llm-config/providers/:id` | 🔒 `setting:manage` | 更新提供商 |
| DELETE | `/admin/llm-config/providers/:id` | 🔒 `setting:manage` | 删除提供商 |
| POST | `/admin/llm-config/providers/:id/set-default` | 🔒 `setting:manage` | 设为默认 |

---

### 5.21 字幕管理

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/subtitles` | 🔒 `subtitle:manage` | 字幕列表 |
| POST | `/admin/subtitles` | 🔒 `subtitle:manage` | 创建字幕 |
| PUT | `/admin/subtitles/:id` | 🔒 `subtitle:manage` | 更新字幕 |
| DELETE | `/admin/subtitles/:id` | 🔒 `subtitle:manage` | 删除字幕 |

---

### 5.22 播放器高级

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/videos/:id/chapters` | 🔒 `video:approve` | 章节列表 |
| POST | `/admin/videos/:id/chapters` | 🔒 `video:approve` | 添加章节 `{ title, start_sec }` |
| DELETE | `/admin/videos/:id/chapters/:chapterId` | 🔒 `video:approve` | 删除章节 |
| GET | `/admin/videos/:id/bitrates` | 🔒 `video:approve` | 码率列表 |
| POST | `/admin/videos/:id/bitrates` | 🔒 `video:approve` | 添加码率 `{ label, url, bitrate }` |
| DELETE | `/admin/videos/:id/bitrates/:bitrateId` | 🔒 `video:approve` | 删除码率 |

---

### 5.23 专题活动

| 方法 | 路径 | 权限 | 说明 |
|------|------|------|------|
| GET | `/admin/specials` | 🔒 `special:manage` | 专题页列表 |
| POST | `/admin/specials` | 🔒 `special:manage` | 创建专题页 |
| POST | `/admin/specials/upload-cover` | 🔒 `special:manage` | 上传封面 |
| PUT | `/admin/specials/:id` | 🔒 `special:manage` | 更新专题页 |
| DELETE | `/admin/specials/:id` | 🔒 `special:manage` | 删除专题页 |
| GET | `/admin/campaigns` | 🔒 `special:manage` | 活动列表 |
| POST | `/admin/campaigns` | 🔒 `special:manage` | 创建活动 |
| PUT | `/admin/campaigns/:id` | 🔒 `special:manage` | 更新活动 |
| DELETE | `/admin/campaigns/:id` | 🔒 `special:manage` | 删除活动 |

---

## 6. WebSocket 端点

| 路径 | 用途 | 鉴权 | 协议 |
|------|------|:--:|------|
| `/api/v1/ws/danmaku` | 弹幕实时推送 | token query | `?video_id=X&token=X` |
| `/api/v1/ws/chat` | 私信实时推送 | token query | `?token=X` |
| `/api/v1/ws/live` | 直播聊天 + 礼物 | token query | `?room_id=X&token=X` |

### 弹幕 WS 消息格式

```
C → S: { "content": "弹幕文字", "position_sec": 12.5, "color": "#FFFFFF", "type": "scroll" }
S → C: { "type": "danmaku", "danmaku": { ... }, "timestamp": 1719700000000 }
S → C: { "type": "history", "danmakus": [...] }
S → C: { "type": "error", "msg": "发送频率过快" }
```

### 直播 WS 事件类型

| 方向 | type | 说明 |
|------|------|------|
| S → C | `user_info` | 连接时发送自身信息+观众列表 |
| S → C | `audience` | 观众列表更新 |
| S → C | `system` | 系统消息（进场/离场/关注） |
| C → S | (raw) | `{ "content": "文字" }` 聊天 |
| C → S | (raw) | `{ "gift": "rose\|rocket\|star\|... }"` 礼物 |
| C → S | (raw) | `{ "follow": true }` 关注主播 |
| S → C | `gift` | 礼物广播 |
| S → C | `admin_warning` | 管理员警告覆盖层 |
| S → C | `admin_ban` | 管理员封禁通知 |

---

## 7. 直播回调

> SRS / node-media-server 推流回调，无需认证。

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/live/callback/on_publish` | 推流开始回调（校验 stream_key） |
| POST | `/api/v1/live/callback/on_done` | 推流结束回调（更新房间状态） |

---

## 8. NocoBase 桥接

> 内部 API，需要 `X-Internal-API-Key` header。

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/v1/internal/takedown/video/:id` | 下架视频 |
| POST | `/api/v1/internal/restore/video/:id` | 恢复视频 |
| POST | `/api/v1/internal/ban/user/:id` | 封禁用户 |
| POST | `/api/v1/internal/unban/user/:id` | 解封用户 |
| GET | `/api/v1/internal/user/:id` | 查询用户信息 |
| GET | `/api/v1/internal/video/:id` | 查询视频信息 |
| POST | `/api/v1/internal/nocobase-webhook` | 通用 Webhook |

---

### 静态资源

| 路径 | 说明 |
|------|------|
| `/live-hls/*` | HLS 直播文件（`data/live/`） |
| `/uploads/*` | 本地文件上传（`data/uploads/`，当未配置 OSS 时使用） |

---

## RBAC 权限码索引

| 分组 | code | resource | action | 用途 |
|------|------|----------|--------|------|
| 📊 数据 | `dashboard:view` | dashboard | view | 查看仪表盘 |
| 📊 数据 | `dashboard:export` | dashboard | export | 导出报表 |
| 📢 运营 | `banner:manage` | banner | manage | 轮播图管理 |
| 📢 运营 | `hotsearch:manage` | hotsearch | manage | 热搜管理 |
| 📢 运营 | `special:manage` | special | manage | 专题活动管理 |
| 📢 运营 | `dynamic:manage` | dynamic | manage | 动态管理 |
| 📢 运营 | `subtitle:manage` | subtitle | manage | 字幕管理 |
| 🛡️ 审核 | `video:approve` | video | approve | 视频审核 |
| 🛡️ 审核 | `article:approve` | article | approve | 专栏审核 |
| 🛡️ 审核 | `comment:delete` | comment | delete | 删除评论 |
| 🛡️ 审核 | `ticket:handle` | ticket | handle | 举报/工单处理 |
| 🛡️ 审核 | `copyright:handle` | copyright | handle | 版权处理 |
| 🛡️ 审核 | `risk:manage` | risk | manage | 风控管理 |
| 👤 用户 | `user:ban` | user | ban | 封禁用户 |
| 👤 用户 | `cs:manage` | cs | manage | 客服管理 |
| 🤖 AI | `agent:manage` | agent | manage | AI角色管理 |
| 🤖 AI | `llm:manage` | setting | manage | LLM配置 |
| ⚙️ 系统 | `setting:manage` | setting | manage | 系统设置 |
| ⚙️ 系统 | `config:manage` | config | manage | 配置发布 |
| ⚙️ 系统 | `ops:manage` | ops | manage | 运维监控 |
| ⚙️ 系统 | `rbac:manage` | rbac | manage | 权限审计 |
| ⚙️ 系统 | `live:manage` | live | manage | 直播管理 |

---

### 预置角色

| 角色 | 权限数 | 权限范围 |
|------|:--:|------|
| `super_admin` | 23 | 全部权限 |
| `content_review` | 8 | video:approve, article:approve, comment:delete, ticket:handle, copyright:handle, risk:manage, user:ban, dashboard:view |
| `cs_admin` | 4 | user:ban, ticket:handle, cs:manage, dashboard:view |

---

### 默认管理员账号

| 用户名 | 角色 | 密码 |
|--------|------|------|
| `superadmin` | super_admin | `admin123` |
| `content_review` | content_review | `review123` |
| `cs_admin` | cs_admin | `cs123` |

---

> **维护说明:** 此文档与 `internal/handler/router.go` 同步。新增路由后请同步更新本文档。
