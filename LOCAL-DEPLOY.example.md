# cakecake / MiniBili 本地部署与验收（模板）

> 这是**脱敏模板**：真实的本地手册含口令、内网 IP、本机路径，已加入 `.gitignore`（`LOCAL-DEPLOY.md`），
> 只留在本机。需要一份自己的：`cp LOCAL-DEPLOY.example.md LOCAL-DEPLOY.md` 再填。
> 容器化部署看 [README.md](./README.md#docker-一键部署)，客户端构建看 [docs/APP.md](./docs/APP.md)。

---

## 一、服务与端口（Windows 原生运行示例）

| 服务 | 端口 | 说明 |
|---|---|---|
| MySQL 8 | 3306 | 库名 `minibili`，账号密码填自己的 |
| Redis | 6379 | 播放计数、弹幕冷却、限流 |
| RabbitMQ | 5672 | 转码队列 |
| Go 后端 | 8080 | REST + WebSocket + 静态 `/uploads`、`/live-hls` |
| 直播服务器 | 1935 / 8000 | RTMP 推流 + HTTP-FLV / HLS |
| PC 前端（Vite） | 8888 | 用户端 + 运营后台 |
| 移动端 App（Rust） | — | 打包时把局域网地址烘焙进 APK，见 §四 |

## 二、账号

口令**不要写进仓库**。本地记录建议放在 `.env`（已被 `.gitignore` 排除）或密码管理器：

```dotenv
# .env（本地，不入库）
MYSQL_DSN=root:<本机口令>@tcp(127.0.0.1:3306)/minibili?charset=utf8mb4&parseTime=True&loc=Local
JWT_SECRET=<随机长串>
ADMIN_SEED_PASSWORD=<设置一个强口令>
```

- 运营后台入口 `<http://localhost:8888/admin/login>`，用户端 `<http://localhost:8888>`
- 测试账号自行在后端注册，或首次启动时用 `ADMIN_SEED_PASSWORD` 建管理员

## 三、启停

```powershell
# 仓库自带的交互式启动器（会自动定位仓库根目录，不再依赖固定路径）
powershell -ExecutionPolicy Bypass -File scripts\cakecake-run.ps1

# 或 docker compose（推荐）
docker compose up -d
```

## 四、移动端打包

```bash
cd cakecake-rs
# 1) 签名口令放本地（不入库）：二选一
#    export CAKECAKE_KS_PASS=<口令>
#    echo <口令> > .keystore-pass
# 2) 打包：脚本自动探测局域网 IP 并烘焙进 APK
bash scripts/build-apk.sh aarch64-linux-android
adb install -r target/release/apk/cakecake-rs.apk
```

签名密钥 `cakecake-rs/cakecake.keystore` 与口令都**不入库**（`.gitignore` 已排除）。
换机器时把这两个文件拷过去即可；口令丢了只能重新生成密钥（届时签名指纹变化，已安装的 App 需卸载重装）。

## 五、直播推流

- 服务器 `rtmp://127.0.0.1:1935/live`，推流密钥在直播间详情里（`stream_key`），**不要贴到公开文档里**
- 开播后房间状态由 `/api/v1/live/callback/on_publish` 回调置为 `live`
- 需要多清晰度：`deploy/srs-docker.conf` 里已开一路 480p 转码，客户端会自动探测 `<stream_key>_sd.m3u8`

## 六、验收清单（本地）

```bash
curl -s http://127.0.0.1:8080/api/v1/health          # 后端存活
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8888   # 前端
curl -s http://127.0.0.1:8000/live/<stream_key>.m3u8 | head -3    # 直播 HLS
```

App 侧：登录 → 首页能出封面 → 详情能播 → 直播间能看、能发弹幕 → 创作中心能投稿/存草稿/发布。

## 七、已知本机差异（按需保留）

- 无 Docker（未启用 WSL2/Hyper-V）时用原生 MySQL/Redis/RabbitMQ；
- `node-media-server` 需 2.7.x（4.x 移除了内置 FFmpeg 转 HLS）；
- HLS 切片路径不要带盘符冒号（ffmpeg `tee` muxer 会误切）。
