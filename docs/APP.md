# Cakecake 移动端（Rust 版）文档

> 移动端客户端 = `cakecake-rs/`：**纯 Rust + egui 自绘**，直连 Go 后端（同 Web 端同一套 REST / WebSocket）。
> 现行版本 **v0.4.0**，Android arm64，真机为 vivo V2180GA（Android 11）。
> 历史文档（已弃用的 uni-app 版）见 [legacy/APP-uniapp.md](./legacy/APP-uniapp.md)。

---

## 1. 为什么弃用 uni-app

2026-08 的第一版移动端是 uni-app（Vue3 + TS，`cakecake-vue/cakecake-app/`），跑通 20 个页面后停在这里，
换成 Rust 的三个直接原因：

| 痛点 | uni-app 下的表现 | Rust 版的处理 |
|------|------------------|---------------|
| 播放器不可控 | 视频/直播只能走 `<video>` 组件，清晰度切换、弹幕渲染层、全屏转屏都要靠平台能力，nvue 与 vue 双份实现 | 直接用系统 `MediaPlayer`（硬解）+ glow 把帧画成 OES 纹理，进度、暂停、清晰度、弹幕层全部自己说了算 |
| 输入法难缠 | H5/WebView 的输入走 DOM，键盘遮挡、候选词、返回键在各家 ROM 上都要单独打补丁 | 一个隐藏 `EditText` 当输入宿主，Rust 侧只读文本，键盘开合自己控制 |
| 运行时太重 | 需要 uni 运行时 + WebView + 插件市场依赖，包体和启动都偏大 | 单个 12MB 的 APK，native 渲染，冷启动直接进首页 |

结论：移动端不再维护 uni-app 工程，`cakecake-vue/` 里只保留 **Web 端（Vue3 + Element Plus 后台）**。

> 移动端和 Web 端复用同一套后端端点，没有为 App 单独开后门接口。

## 2. 技术栈

| 项 | 选择 |
|----|------|
| 语言/UI | Rust 2021 + [egui 0.37](https://github.com/emilk/egui)（eframe，wgpu/glow 双后端） |
| Android 壳 | `android-activity`（NativeActivity）+ 一层 Java 薄壳 `MainBack.java` |
| 网络 | `ureq`（阻塞 + `{code,msg,data}` 信封解包）· `serde` |
| 实时 | 自写 WebSocket 客户端（弹幕 / 直播聊天 / 私信三通道） |
| 播放 | Android `MediaPlayer`（JNI，硬解）+ `egui_glow` 绘制回调贴 OES 纹理 |
| 图片 | 后台线程解码（`image` crate）→ 主线程上传纹理，本地有文件优先读盘 |
| 包名/签名 | `com.cakecake.rust` · `cakecake.keystore`（alias `cakecake`） |

## 3. 代码结构

```
cakecake-rs/
├── src/
│   ├── main.rs        # 桌面窗口入口（开发时直接 cargo run）
│   ├── lib.rs         # Android 入口（android_main）+ 日志初始化
│   ├── app.rs         # 全局状态机：路由、消息泵、登录态、面板布局、轻提示
│   ├── api.rs         # 所有后端端点 + JWT 刷新 + 流式上传
│   ├── video.rs       # JNI：MediaPlayer / 输入法 / 系统栏 / 相册 / 文件选择 / 视频首帧
│   ├── livechat.rs    # 直播间聊天（WS + 本地状态）
│   ├── wsclient.rs    # 最小 WebSocket 客户端
│   ├── images.rs      # 图片缓存（线程池 + 纹理池）
│   ├── theme.rs       # 颜色/字号/时钟格式化
│   ├── widgets.rs     # 自绘控件：封面、头像、蒙层、原生输入框代理…
│   ├── subtitle.rs    # 字幕解析（srt/vtt）
│   ├── zones.rs       # 分区图标与配色
│   └── pages/         # 19 个文件 / 30+ 个页面（more.rs 聚合了创作中心、治理、客服等子页）
├── android/java/…/MainBack.java   # 返回键、隐藏输入框、相册/文件选择、视频首帧
└── scripts/build-apk.sh           # 编译 + javac/d8 + 重签
```

## 4. 页面清单

路由定义在 `app.rs::Route`，页面实现在 `src/pages/`。

| 分组 | 路由 → 页面 |
|------|-------------|
| 4 个 Tab | `Tab(Home)` 首页 · `Tab(Dynamic)` 动态 · `Tab(Live)` 直播 · `Tab(Mine)` 我的 |
| 播放 | `Detail` 视频详情（含弹幕/评论/简介）· `LiveRoom` 直播间（竖屏+横屏全屏） |
| 发现 | `Search` 搜索 · `Zones` 全部分区 · `Rank` 排行榜 · `Specials` 专题 |
| 内容 | `ArticleList` 专栏 · `Article` 文章详情 · `DynamicDetail` 动态详情 |
| 个人 | `Space` 他人主页 · `WatchLater` 稍后再看 · `History` 观看历史 · `Favorites` 收藏 · `ArticleFavorites` 图文收藏 |
| 消息 | `Notifications` 消息中心 · `Dm` 私信列表 · `DmChat` 私信会话 |
| 创作 | `Creator` 创作中心 · `Upload` 视频投稿 · `VideoEdit` 稿件编辑 · `MyArticles` 文章编辑 · `LiveHost` 直播开播 · `CreatorComments`/`CreatorDanmakus` |
| 其它 | `Settings` 设置 · `Ledger` 硬币账本 · `Governance`/`Ticket`/`Appeal`/`CsChat` 治理与客服 · `CommentDetail` 评论详情 |

## 5. 关键实现

### 5.1 网络与登录态

- `api.rs::Client` 持有 `base` 与 token；`send()` 统一解包 `{code,msg,data}`，401 自动用 refresh token 换新（后端会轮换）。
- 令牌**落盘**到 `<files>/auth.txt`（与 `server.txt` 同目录），重装/重启不用重登。
- 服务器地址优先级：运行期 `CAKECAKE_API` > 编译期烘焙的局域网地址（`build-apk.sh` 自动探测本机 LAN IP）> `server.txt` 里记住的 > 模拟器网关探测。

### 5.2 视频与直播

- VOD：`/videos/:id` 拿 HLS，`MediaPlayer` 硬解；进度、清晰度、倍速、弹幕层自绘。
- 直播：`/live/{stream_key}.m3u8`；开启 SRS 转码后额外探测 `<key>_sd.m3u8`，探到才在播放器里露出「流畅」档（不做假选项）。
- 横屏全屏：请求横屏 + 沉浸式，系统栏 inset 用 `WindowInsets` 实取（隐藏时不留黑边，未隐藏时控件按实际宽度让位）。
- 弹幕：聊天消息当弹幕从右往左飘（9 秒一条，系统提示不飘），竖屏聊天列表与横屏弹幕是同一份数据。

### 5.3 输入法（Android 必踩）

NativeActivity 的窗口没有输入连接，egui 的 `TextEdit` 收不到中文（软键盘只吐数字）。做法：

1. Java 壳挂一个 1×1 透明的 `EditText` 到 DecorView；
2. 点自绘输入框 → `beginInput`/`beginInputMulti`（单行/多行两种）→ `SHOW_FORCED` 拉键盘；
3. Rust 每帧 `ime_text()` 把内容镜像回业务字段，`ime_close()` 收起；
4. 横屏额外关掉 `IME_FLAG_NO_EXTRACT_UI`，否则会被输入法接管成全屏编辑页。

聊天、评论、弹幕、文章标题/正文、投稿信息、工单全部走这条链路。

### 5.4 图片

`images.rs` 维护 URL → 纹理的缓存：本地 `data/` 目录有文件先读盘，没有就 HTTP 拉 `/uploads/...`；
评论配图、文章插图、封面、头像共用。点评论配图会开全屏查看（按原始比例居中）。

## 6. 构建与安装

```bash
# 依赖：Rust + NDK + Android SDK（build-tools 里有 d8/apksigner/zipalign）
cd cakecake-rs
bash scripts/build-apk.sh aarch64-linux-android     # 真机
# bash scripts/build-apk.sh x86_64-linux-android    # 模拟器

# 产物：target/release/apk/cakecake-rs.apk（已重签，可直接安装）
adb install -r target/release/apk/cakecake-rs.apk
```

- 局域网地址自动烘焙：脚本会挑「已连接且有默认网关」的物理网卡 IP；也可 `CAKECAKE_LAN_IP=192.168.x.x bash scripts/build-apk.sh` 手动指定，或用 `CAKECAKE_LAN_API=` 关掉。
- 桌面调试：`cargo run`（不带 Android 目标时 `video.rs` 走空实现，页面照常打开）。
- 一键自检脚本与截图产物在 `scripts/`，历史发版说明见 `cakecake-rs/RELEASE-v*.md`。

## 7. 已知限制

- 横屏输入依赖 `SHOW_FORCED` + `NO_EXTRACT_UI`，个别 ROM 仍可能把键盘放到屏幕外；
- HLS 3 秒切片，直播端到端延迟 6~9 秒（与 Web 端一致）；
- 文章编辑器暂不支持插图与字号（Web 端有），Markdown 文本可直接粘贴；
- 只做了 arm64 与 x86_64 两个 Android 目标，没有 iOS。

## 8. 与后端的对应

移动端复用**用户端端点**：认证、视频/弹幕/评论、动态、专栏、私信/通知、搜索、直播、创作中心、治理工单。
端点清单与字段以 [API.md](./API.md) 为准，App 侧封装在 `api.rs`（一个方法对一条端点）。
后台管理（23 个模块）只在 Web 端，App 不做。
