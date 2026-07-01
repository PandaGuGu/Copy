# Cakecake 免费公网部署方案

> 月费：¥0 | 适用：Demo 展示 / 小范围测试（< 50 DAU）
>
> 最后更新：2026-07-01

---

## 架构总览

```
用户浏览器
    │
    ▼
┌──────────────────┐
│  Netlify / Vercel │  ← Vue 3 SPA 静态前端（完全免费）
│  (cakecake.xyz)   │
└────────┬─────────┘
         │ API 调用
         ▼
┌──────────────────┐
│  Fly.io           │  ← Go 后端（免费 3 VM，256MB × 3）
│  (api.cakecake.xyz)│     无 RabbitMQ（同步转码）
└──┬────┬────┬─────┘     无 Elasticsearch（MySQL LIKE 搜索）
   │    │    │
   ▼    ▼    ▼
┌────┐┌─────┐┌──────────┐
│PS  ││Ups  ││Cloudflare│
│    ││tash ││R2        │
│5GB ││1GB  ││10GB      │
└────┘└─────┘└──────────┘

❌ RabbitMQ → 禁用（视频上传关闭或同步转码）
❌ ES       → 禁用（搜索退化为 MySQL LIKE）
❌ SRS 直播 → 免费 VM 跑不动，标记"即将上线"
```

## 免费服务注册清单（10 分钟）

| 步骤 | 服务 | 注册地址 | 额度 |
|:--:|------|---------|:--:|
| 1 | **Netlify** | netlify.com | 100GB/月，自动 HTTPS |
| 2 | **Fly.io** | fly.io | 3 个 256MB VM，30GB 出站 |
| 3 | **PlanetScale** | planetscale.com | 5GB，1B 行读/月 |
| 4 | **Upstash Redis** | upstash.com | 1GB，10K 命令/天 |
| 5 | **Cloudflare R2** | cloudflare.com | 10GB，每月 10M 操作 |
| 6 | **域名**（可选）| Namecheap/Porkbun | $0-10/年，或用服务自带子域名 |

> 全部注册无需信用卡。

---

## 分步部署

### 第一步：准备前端（Netlify）

```bash
# 1. 构建前端
cd cakecake-vue/bilibili-vue
npm install
npm run build

# 2. 创建 Netlify 配置文件
cat > netlify.toml << 'EOF'
[build]
  publish = "dist"

[[redirects]]
  from = "/api/*"
  to = "https://api.cakecake.xyz/api/:splat"
  status = 200

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
EOF

# 3. 登录 Netlify CLI 部署
npx netlify-cli deploy --prod --dir=dist
```

### 第二步：数据库（PlanetScale）

```bash
# 1. 安装 CLI
npm i -g planetscale

# 2. 创建数据库
pscale database create cakecake --region ap-southeast
pscale branch create cakecake main

# 3. 获取连接字符串
pscale connect cakecake main --port 3309
# 输出: mysql://...  → 记下来

# 4. 首次启动后端时会 GORM AutoMigrate 自动建表
```

### 第三步：Redis（Upstash）

1. 注册 → Create Database → 选 Region（新加坡/东京离 Fly.io 最近）
2. 复制 `REDIS_URL`（格式 `rediss://:password@host:port`）

### 第四步：文件存储（Cloudflare R2）

```bash
# 1. Cloudflare Dashboard → R2 → Create Bucket "cakecake"
# 2. 创建 API Token: Manage R2 → 选 bucket → 复制 Access Key ID + Secret
# 3. 设置公开访问 → Bucket Settings → Public Access → Allow
# 4. 记录 Public URL: https://<bucket>.<account>.r2.cloudflarestorage.com
```

### 第五步：Go 后端（Fly.io）

```bash
# 1. 安装 Fly CLI
curl -L https://fly.io/install.sh | sh

# 2. 登录
fly auth signup

# 3. 创建应用
fly launch --name cakecake-api --region sin
# 选择不部署（先配环境变量）

# 4. 设置环境变量
fly secrets set \
  JWT_SECRET="$(openssl rand -hex 32)" \
  APP_ENV=production \
  MYSQL_DSN="<PlanetScale DSN>" \
  REDIS_ADDR="<Upstash host>" \
  REDIS_PASSWORD="<Upstash password>" \
  REDIS_DB=0 \
  RABBITMQ_URL="" \
  ELASTICSEARCH_URL="" \
  TEMP_UPLOAD_DIR=/tmp \
  VIDEO_UPLOAD_DISABLED=false \
  ADMIN_SEED_USERNAME=admin \
  ADMIN_SEED_PASSWORD="$(openssl rand -hex 12)"

# 5. 设置 R2 存储
fly secrets set \
  OSS_ACCESS_KEY_ID="<R2 Access Key>" \
  OSS_ACCESS_KEY_SECRET="<R2 Secret Key>" \
  OSS_BUCKET=cakecake \
  OSS_ENDPOINT="https://<account>.r2.cloudflarestorage.com" \
  OSS_PUBLIC_URL_PREFIX="https://<bucket>.<account>.r2.cloudflarestorage.com"

# 6. 部署
fly deploy
```

### 第六步：验证

```bash
# 健康检查
curl https://api.cakecake.fly.dev/api/v1/health

# 前端访问
open https://cakecake.netlify.app

# 管理后台（使用上面生成的 ADMIN_SEED_PASSWORD）
curl -X POST https://api.cakecake.fly.dev/api/v1/admin/auth/login \
  -d '{"username":"admin","password":"<生成的密码>"}'
```

---

## 免费功能矩阵

| 功能 | 状态 | 说明 |
|------|:--:|------|
| 首页浏览 | ✅ | 完全正常 |
| 用户注册/登录 | ✅ | JWT 双 Token |
| 视频上传 | ✅ | 需 > 256MB RAM 转码小视频 (< 3min) |
| 视频转码 | ⚠️ | 同步转码，大视频会超时。建议：限制上传 ≤30MB/3min |
| 弹幕 | ✅ | WebSocket 走 Fly.io 自带支持 |
| 评论/社交 | ✅ | MySQL 完全支撑 |
| ES 全文搜索 | ❌ | 退化为 MySQL LIKE，够用但不精确 |
| Feed 推荐 | ✅ | MMR 规则排序，不依赖 ES |
| 直播 | ❌ | 无免费 SRS，前端隐藏入口 |
| 运营后台 | ✅ | 23 模块全部可用 |
| HTTPS | ✅ | Netlify + Fly.io 自动提供 |
| 自定义域名 | ✅ | 绑定你自己的域名 |

---

## 免费额度警戒线

| 资源 | 免费额度 | 炸了会怎样 |
|------|---------|-----------|
| Fly.io 出站流量 | 30GB/月 | 超额按 $0.02/GB 计费 |
| PlanetScale 写入 | 10M 行/月 | 超额拒绝写入 — **弹幕多发会炸** |
| PlanetScale 存储 | 5GB | 超额拒绝写入 |
| Upstash 命令 | 10K/天 | 超额限流 — **弹幕+播放计数走 Redis，日均 > 10K 请求会炸** |
| R2 存储 | 10GB | 超额按 $0.015/GB 计费 |
| R2 操作 | 10M/月 | 超额按 $0.36/百万次计费 |
| Netlify 带宽 | 100GB/月 | 超额按 $20/100GB 计费 |

---

## 省钱建议

1. **PlanetScale 写入配额** → 弹幕 5s 冷却已经帮你省了一大笔。如果还是超，调 Redis 播放计数从 10s → 60s 落库。
2. **Upstash 命令配额** → 关掉 Token 黑名单（用短 Access Token 2h 自然过期代替 Redis 黑名单）。
3. **R2 存储** → 设置 7 天自动过期（运维后台 → 存储生命周期规则）。
4. **Fly.io VM** → 如果视频上传关闭（`VIDEO_UPLOAD_DISABLED=true`），256MB 完全够跑。

> 按 50 DAU 估算，所有配额都在安全线内。唯一风险是有人恶意发弹幕刷 Redis 命令配额——但你的 API 限流中间件会兜底。

---

## 从免费到付费的路标

| 日活 | 需要升级什么 | 月费估算 |
|:--:|-------------|:--:|
| < 50 | 什么都不用 | ¥0 |
| 50-200 | Upstash → Redis Cloud 30MB 付费版 | ¥30 |
| 200-500 | PlanetScale → 付费版 + Fly.io 升到 512MB | ¥150 |
| 500+ | 上 RabbitMQ + ES + 专用 VPS | ¥500+ |
