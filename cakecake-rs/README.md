# cakecake-rs

Cakecake 移动端客户端：**纯 Rust + egui 自绘**，直连 Go 后端（与 Web 端同一套 REST / WebSocket）。

> 完整文档（架构、页面、输入法/播放实现、构建与已知限制）见 **[docs/APP.md](../docs/APP.md)**。
> 这是 2026-11 起取代 uni-app 客户端的现行路线，旧工程 `cakecake-vue/cakecake-app/` 已停更。

## 快速开始

```bash
# 桌面窗口（开发用，JNI 相关走空实现）
cargo run

# Android APK（真机；模拟器用 x86_64-linux-android）
bash scripts/build-apk.sh aarch64-linux-android
adb install -r target/release/apk/cakecake-rs.apk
```

打包脚本会自动探测本机局域网 IP 并烘焙进 APK（`CAKECAKE_LAN_IP=192.168.x.x` 可手动指定，
`CAKECAKE_LAN_API=` 可关闭），产物为已重签的 `target/release/apk/cakecake-rs.apk`。

## 目录

| 路径 | 说明 |
|------|------|
| `src/app.rs` | 全局状态机：路由、消息泵、登录态、布局 |
| `src/api.rs` | 后端端点封装 + JWT 刷新 + 流式上传 |
| `src/video.rs` | JNI：MediaPlayer / 输入法 / 系统栏 / 相册与文件选择 / 视频首帧 |
| `src/pages/` | 页面 |
| `android/java/` | Java 薄壳（返回键、隐藏输入框、相册、视频首帧） |
| `scripts/build-apk.sh` | 编译 + javac/d8 + 重签 |

## 版本

发版说明见 `RELEASE-v*.md`；当前 **v0.4.0**。
