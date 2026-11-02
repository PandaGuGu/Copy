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
