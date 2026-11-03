# cakecake-rs v0.2.0 — App 内直播播放

纯 Rust（egui）Android 客户端首次具备**在 App 内观看直播**的能力，真机验收通过。

## 本次重点

### 1. 直播播放（新功能）

链路：`SRS HLS(.m3u8)` → 系统 `MediaPlayer`（SoC 硬解）→ `SurfaceTexture`（OES 外部纹理）→ egui 的 glow 绘制回调自绘。

- 视频是页面里的一个矩形，跟着布局滚动，不是浮在上面的独立窗口；
- 离开房间立即 `release()` 解码器，不会后台继续出声；
- 未开播的房间仍显示封面 + 文案提示，不占解码器；
- 播放失败时点视频区域可重连。

### 2. Docker 部署改动（重要，升级需重建容器）

| 文件 | 改动 | 原因 |
|---|---|---|
| `docker-compose.yml` | srs 服务新增 `command: ["./objs/srs", "-c", "conf/srs.conf"]` | `ossrs/srs:5` 镜像默认 CMD 是 `conf/docker.conf`，挂载进去的 `conf/srs.conf` **从未生效**，`http_hooks` 不触发，房间永远停在「未开播」 |
| `deploy/srs-docker.conf` | 补齐 `hls {}` 段并设置 `hls_ctx off` | 让 `:8000/live/<stream_key>.m3u8` 直接返回媒体播放列表；开启 hls_ctx 时会重定向到带会话参数的主列表，部分播放器解析不稳 |
| `Dockerfile` | `mkdir -p /app/data/tmp /app/data/uploads /app/logs` | `uploads_data` 是具名卷，镜像里没有该目录时 Docker 会以 `root:root` 创建挂载点，而容器以 `uid=1000(appuser)` 运行 → 封面上传 `mkdir data/uploads/live-covers: permission denied` → 500 |
| `internal/handler/live.go` | 新增 `srsHookStreamKey()` | SRS 回调发的是 JSON body，原实现只读 query/form 的 `stream` → 返回 400 → **SRS 会直接拒绝推流** |

**升级步骤**

```bash
# 1. 重建后端（含上传目录修复 + 回调解析修复）
docker compose build backend && docker compose up -d backend

# 2. 让 SRS 用上新配置（compose 里改了 command，需要重建容器）
docker compose up -d --force-recreate srs

# 3. 已存在的 uploads 卷如果还是 root 属主，补一次（新卷不需要）
docker exec -u 0 cake-backend chown -R 1000:1000 /app/data/uploads
```

验证：`docker logs cake-srs | grep conf:` 应显示 `conf:conf/srs.conf`；OBS 开播后 `GET /api/v1/live/room/<id>` 的 `status` 应变成 `live`。

### 3. 客户端其它修复

- 自建 GL VAO：在 egui 已绑定的 VAO 上改顶点属性会破坏它的布局，导致整帧 UI 只剩视频（这个 bug 花了几轮才定位）；
- 通过 JNI 读取 `status_bar_height` / `navigation_bar_height` 补内衬，修复返回键与底部 tab 被系统手势区挡住点不到。

## 安装

- 架构：**arm64-v8a**（Android 7.0 / API 24+）
- 签名与 0.1.0 一致，可直接覆盖安装；`versionCode 16777728`
- 开发期连后端：App 内烘焙地址为 `http://127.0.0.1:8080`，真机调试用
  `adb reverse tcp:8080 tcp:8080 && adb reverse tcp:8000 tcp:8000`；
  走 Wi-Fi 需带 `CAKECAKE_API=http://<电脑IP>:8080` 重新编译。

```bash
adb install -r cakecake-rs-0.2.0-arm64-v8a.apk
```

SHA256：`8983291815274a46a3fb2e9a4b855c97933eec719582d3caa23ac43a7202b899`

## 已知限制

- HLS 为 4 秒切片，端到端延迟约 6–9 秒（不是 FLV 的 1 秒级）；
- 仅构建 arm64 单架构；
- 播放依赖后端 `/api/v1/live/room/:id` 返回的 `stream_key`。

## 模拟器验收修复（2026-11-03）

安卓模拟器（1080×2400 / Android 14 / 三键导航）实测发现并修复三个问题：

| 问题 | 现象 | 修复 |
|---|---|---|
| 系统返回键无效 | 三键导航栏的返回键被 winit 吃掉（映射成 `Key::BrowserBack`），egui 无处理，页面不动也不退出 | `App::ui` 拦截 `BrowserBack`/`Esc`：二级页回上一页，根页面 `ViewportCommand::Close` 退出；同时新增路由历史栈，从直播房间返回回直播 tab、从搜索进详情返回回搜索页 |
| 首页无顶部搜索栏 | 头部只有 `cakecake` + 后端地址 + 刷新 | 头部改为「左侧头像 + 中间搜索胶囊 + 右侧刷新」；点搜索进新增的搜索页 |
| 首页左侧无头像 | 完全没有头像位 | 未登录画灰底小人，登录后取 `/users/me` 的 `avatar_url`（解码失败回退首字占位），点击进「我的」 |

顺带：新增 `Route::Search` 与 `pages/search.rs`，接 `GET /api/v1/hot-search`、`GET /api/v1/search`（读 `result.video`），
输入框聚焦时通过 `ViewportCommand::IMEAllowed(true)` 拉起软键盘；底部导航与返回栏按 JNI 读取的系统栏高度做内缩，不再被三键导航条盖住。

验收件：`verify/01-home-header-search.png`、`02-search-page.png`、`03-back-to-search.png`、`04-search-results.png`。

## 视频详情页重做（2026-11-03）

原来的详情页只有封面 + 弹幕列表 + 评论，没有播放器，看起来就是「一张图片」。现在按 B 站详情页的结构重做：

| 区块 | 实现 |
|---|---|
| 播放器 | `video_url` → 系统 MediaPlayer（H.264 硬解）→ SurfaceTexture → egui 自绘，**真的有画面**；暂停/继续、点时间轴跳转、当前进度/总时长 |
| 弹幕 | 读 `/videos/:id/danmaku`，按 `video_time` 跟播放进度从右往左滚，多泳道、黑边描边 |
| 弹幕输入 | 简介/评论同一行右侧的输入框，聚焦拉起软键盘，回车 `POST /videos/:id/danmaku`（带当前 `video_time`） |
| 弹幕开关 | 输入框最右的「弹」按钮，关闭后不绘制弹幕（灰色 + 斜杠），再点恢复 |
| tab | 简介 / 评论（显示已加载条数），粉色下划线 |
| UP 主区 | 头像 + 昵称 + 粉丝数/视频数 + 关注按钮（`POST /users/:id/follow`） |
| 标题区 | 两行标题 + 播放/弹幕/发布时间/在看人数 |
| 操作栏 | 点赞 / 投币 / 收藏，计数与已操作态来自详情接口，点击走 authd 接口并静默刷新；未登录给提示 |
| 简介 | 标签 + 描述 |
| 相关推荐 | `/feed/recommendation` 去掉当前视频，点一条直接切视频（播放器换源） |

另外修了两个生命周期问题：

- 根页面按返回改为 `moveTaskToBack`，不再 `finish` Activity——否则同进程重建 Activity 时 winit 报 `EventLoop can't be recreated`，App 再也起不来；
- 同一个进程里被系统多开一个实例时，第二次 `android_main` 直接把自己那个 Activity finish 掉，不再黑屏。

验收件：`verify/05-detail-video-page.png`、`06-danmaku-off.png`、`07-danmaku-on.png`、`08-danmaku-input.png`、`09-playback-progress.png`。

**还没做**：分享、不喜欢、充电、三连弹窗、清晰度切换（后端有 `/videos/:id/bitrates`，客户端还没接）、播放结束自动重播。

## 动态页 + 直播间聊天（2026-11-03）

### 动态（关注）tab

底部导航从 3 个变 4 个：首页 / **动态** / 直播 / 我的。

| 区块 | 数据来源 |
|---|---|
| 我关注的人（横向头像条，显示人数） | `GET /users/me/followings` |
| 全部 / 视频 子 tab | 本地过滤：视频只看投稿，全部是投稿+动态按时间归并 |
| 投稿卡（头像 +「投稿了视频」+ 封面 + 时长/播放/弹幕 + 标题 + 评论/点赞） | `GET /feed/subscription`（返回 `model.Video`，字段是 Go 驼峰名 `ID/Title/CoverURL/DurationSec/...`，客户端单独建模解析） |
| 图文动态卡（头像 + 时间 + 正文 + 最多 3 张图 + 评论/点赞） | `GET /dynamics/following` |
| 点投稿卡 | 直接进视频详情（播放器自动起播） |

未登录时整页是「登录后查看关注动态 → 去登录」，点了跳「我的」。

### 直播间聊天

后端只有 `GET /api/v1/ws/live?room_id=&token=` 一条 WS 通道，所以客户端自己写了个最小 WS：
`src/livechat.rs` 负责握手（`Sec-WebSocket-Key` + 校验 101）→ 拆帧/组帧（含掩码、126/127 扩展长度、ping/pong）→ 把
`{"type":"message"|"system"|"audience"|"user_info"}` 转成界面事件。没有引入第三方 WS 库（避免给安卓交叉编译再拉依赖），
读超时 120ms + 自己缓冲半包，所以「发送」和「收消息」在同一条线程里也不会卡。

房间页因此重排：播放器 + 房间信息在上，底部固定一块聊天面板（`Panel::bottom`，232pt）：
- 头部「聊天」+ 在线人数（来自 `audience` / `user_info` 的 count）
- 消息列表（系统消息灰色、用户消息「昵称：内容」，自动滚到底，最多留 200 条）
- 输入框 + 发送（回车也能发），离开房间自动断开 WS

顺手修了两处会被系统栏挡住/看不见输入的问题：

- 二级页（详情/搜索/直播间）之前没有为三键导航栏留内衬 → 底部控件被压住，现在统一补 `bottom_inset`；
- Android 15 强制 edge-to-edge 后 `adjustResize` 失效，键盘是盖在画布上的 → 通过 JNI 读
  `WindowInsets.Type.ime()` 高度，输入框聚焦时把内容整体顶到键盘上方。

验收件：`verify/10-dynamic-need-login.png`、`11-dynamic-feed.png`、`12-dynamic-video-tab.png`、`13-live-chat.png`、`14-live-chat-sent.png`。

**还没做**：动态点赞/评论、发布动态、直播间礼物、聊天历史（后端只推实时消息，没有历史接口）。
