# Cakecake 项目部署记录

## 项目概况
- 仓库: https://github.com/earthcake2233/cakecake
- 后端: Go (Gin + GORM + JWT + Redis + RabbitMQ)
- 前端: Vue 3 + Vite + Element Plus (端口 8888)
- 数据库: MySQL (root:123456, 数据库 minibili)
- 模块名: minibili

## 环境部署 (2026-06-10)

### 已部署服务
- **Go 1.24.5**: `C:\Program Files\Go\bin\go.exe` (go.mod 中 `go 1.25.0` 改为 `go 1.24.0`)
- **Go 代理**: `GOPROXY=https://goproxy.cn,direct`
- **Go 工具链**: `GOTOOLCHAIN=local` (防止自动下载 go 1.25)
- **MySQL 9.6**: root/123456, 数据库 `minibili` (utf8mb4)
- **Redis 3.0.504**: Windows 版本, `C:\Program Files\Redis\redis-server.exe`, 端口 6379
- **FFmpeg 8.1.1**: winget 安装 (Gyan.FFmpeg.Essentials), ffprobe/ffmpeg 可用
- **Erlang OTP 26.2.5**: `C:\Program Files\Erlang OTP`, ERLANG_HOME 已设置, ERTS 14.2.5
- **RabbitMQ 4.3.0**: `C:\rabbitmq\rabbitmq_server-4.3.0`, AMQP 5672, 管理界面 15672
  - epmd.exe 在 `erts-14.2.5\bin`（非主 `bin` 目录，PATH 需额外添加）
  - 插件: rabbitmq_management 已启用
  - 队列: `mini_bili_transcode` (由后端自动创建)
  - Erlang 29 不兼容 → 降级到 Erlang 26.2.5 解决

### 代码修改
1. `go.mod`: `go 1.25.0` → `go 1.24.0`, `golang.org/x/time` `v0.15.0` → `v0.5.0`
2. `cmd/mini-bili/main.go`: RabbitMQ 连接失败不 fatal，改为 warn；transcode consumer 条件启动

### 环境变量 (.env)
- `APP_ENV=development`, `HTTP_ADDR=127.0.0.1:8080`
- `MYSQL_DSN=root:123456@tcp(127.0.0.1:3306)/minibili`
- `REDIS_ADDR=127.0.0.1:6379`
- `ADMIN_SEED_USERNAME=admin`, `ADMIN_SEED_PASSWORD=admin123`
- `VIDEO_UPLOAD_DISABLED=false`, `VIDEO_REVIEW_REQUIRED=false`
- `OSS_ACCESS_KEY_ID=LTAI5t64NLkuYedJA581bTLp`, Bucket=`pandagugu`, Endpoint=`oss-cn-hongkong.aliyuncs.com`
- `ELASTICSEARCH_URL` 空 (无 ES)
- `DEEPSEEK_API_KEY` 空 (无 AI)

### 前端 .env.development
- `VITE_MINIBILI_API=true`
- `VITE_REMOTE_API_BASE=http://127.0.0.1:8080`

### 启动命令
```bash
# 后端
cd cakecake-project && ./mini-bili.exe

# 前端
cd cakecake-project/cakecake-vue/bilibili-vue && npx vite --host 0.0.0.0 --port 8888

# Redis
redis-server --port 6379
```

### 前端路由
- Vue Router 使用 **Hash 模式** (`createWebHashHistory`)
- 所有页面路径需带 `#/` 前缀，如 `http://127.0.0.1:8888/#/admin`
- 运营后台: `http://127.0.0.1:8888/#/admin`, 登录 `http://127.0.0.1:8888/#/admin/login`

### 未完成 / TODO
- Elasticsearch 未配置 → 全文搜索不可用
- DeepSeek AI 未配置 → AI 助手不可用
- 管理员账号 (admin/admin123) 首次启动自动创建
- 排行榜链接未迁回导航栏（从搜索区域移除后暂未重新添加）

## 前端重构记录 (2026-06-12)

### 版本历史 (GitHub: PandaGuGu/Copy)
- v0.0.0: B站风格登录弹窗全面重构
- v0.0.1: 同 v0.0.0（初始发布）
- v0.0.2: primary-menu 从 header 移至首页底部
- v0.0.3: 搜索框从 head-banner 移至顶部导航栏

### 搜索框迁移 (v0.0.3)
- 搜索框从 `header.vue` head-banner 区迁移至 `navMenu.vue` 导航栏
- 导航栏结构：左链接 → 搜索框 → 右上传/用户区
- `navMenu.vue` 使用 `this.$store.state.header.*` 跨命名空间访问
- App.vue 清理 ~189 行废弃全局样式
- header.vue 精简为仅导航栏+banner渲染

### 分类导航迁移 (v0.0.2)
- `primary-menu` 从 `header.vue` 移至 `home/index.vue` 页面底部
- 子菜单向上展开（`top: auto; bottom: 44px`）
- header.vue 仍保留 `setMenuIcon()` 初始化 Vuex 数据
