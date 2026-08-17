# CakeCake 移动端 App

> Cakecake（mini-bili）配套移动端，基于 **uni-app + Vue3 + TypeScript + Vite** 构建

## 📐 技术栈

| 层 | 选型 | 说明 |
|---|---|---|
| 框架 | uni-app 3.x | 编译到 H5 / iOS / Android / 微信小程序 |
| UI 库 | uni-ui + 自写 SCSS | 官方维护 + 灵活定制 |
| 状态 | Pinia 2.x | Vue 官方推荐 |
| 网络 | axios + JWT 自动刷新 | 信封 `{code, msg, data}` 自动解析 |
| 语言 | TypeScript 4.9 | 全套 Interface 约束 |
| 构建 | Vite 5 | HMR / 1.7s ready |
| 样式 | SCSS | BEM 命名 + rpx 单位 |

## 🚀 启动

```bash
cd cakecake-vue/cakecake-app
npm install            # 1min (npmmirror)
npm run dev:h5         # → http://localhost:5173
npm run build:h5       # → dist/build/h5/
npm run type-check     # vue-tsc --noEmit
```

> 端口被占用时自动切到 5174（已实测）。

## 🔌 环境变量

| 文件 | 变量 | 默认值 |
|---|---|---|
| `.env.development` | `VITE_API_BASE_URL` | `http://127.0.0.1:8080` |
| `.env.production`  | `VITE_API_BASE_URL` | `https://cakecake-api.onrender.com` |

dev 期 H5 通过 vite.config.ts 的 `/api` 代理转发，避免浏览器 CORS 阻断。

## 📂 目录结构

```
cakecake-app/
├── src/
│   ├── api/               # API 模块（auth/video/user/dynamic/category/banner）
│   │   ├── auth.ts
│   │   ├── video.ts
│   │   ├── user.ts
│   │   ├── dynamic.ts
│   │   ├── category.ts
│   │   ├── banner.ts
│   │   └── index.ts       # barrel
│   ├── store/             # Pinia
│   │   ├── user.ts
│   │   └── app.ts
│   ├── utils/
│   │   ├── request.ts     # axios + JWT 刷新 + 信封解析
│   │   └── types.ts       # TS Interface 全集
│   ├── pages/             # 10 个页面
│   │   ├── index/         # 首页
│   │   ├── categories/    # 全部分区
│   │   ├── follow/        # 关注流
│   │   ├── mine/          # 我的
│   │   ├── mine-services/ # 我的服务
│   │   ├── mall/          # 会员购
│   │   ├── publish/       # 视频发布器
│   │   ├── video-detail/  # 视频详情
│   │   ├── search/        # 搜索
│   │   └── login/         # 登录/注册
│   ├── styles/
│   │   └── common.scss    # 全局 SCSS 变量 + 工具类
│   ├── static/
│   │   ├── tabbar/        # TabBar 图标（8 个）
│   │   ├── avatar/        # 头像占位
│   │   ├── demo/          # 视频演示封面
│   │   ├── mall/          # 商城 promo / flash 图
│   │   ├── logo.png
│   │   └── placeholder.png
│   ├── App.vue
│   ├── main.ts            # Pinia 注入
│   ├── manifest.json      # uni-app 应用配置
│   ├── pages.json         # 路由 + tabBar + midButton
│   ├── uni.scss           # 全局 SCSS 变量
│   └── env.d.ts           # 环境变量类型声明
├── scripts/
│   └── gen-assets.py      # PIL 占位图生成器
├── index.html             # H5 入口
├── vite.config.ts         # Vite + 代理
├── tsconfig.json
├── .env.development
├── .env.production
└── package.json
```

## 🎨 视觉规范

- **主色**：`#FB7299`（B 站粉），配套 `#FFE4EC`（浅粉）背景
- **辅色**：`#FFD700`（大会员金）、`#F56C6C`（徽章红）、`#67C23A`（成功绿）
- **文字**：`#181818`（主要）/ `#666`（次要）/ `#999`（提示）
- **背景**：`#FFFFFF`（卡片）/ `#F8F8F8`（页面）
- **单位**：所有尺寸 rpx（750rpx = 屏幕宽度），字号建议 22-32rpx

## 📱 10 个页面

| # | 路由 | 截图对应 | 状态 |
|---|------|---------|------|
| 1 | `/pages/index/index` | 推荐 Tab | ✅ 骨架 |
| 2 | `/pages/categories/index` | 全部分区 | ✅ 骨架 |
| 3 | `/pages/follow/index` | 关注 | ✅ 骨架 |
| 4 | `/pages/mine/index` | 我的（上半屏） | ✅ 完整 |
| 5 | `/pages/mine-services/index` | 我的服务 | ✅ 完整 |
| 6 | `/pages/mall/index` | 会员购 | ✅ 骨架 |
| 7 | `/pages/publish/index` | 发布器 | ✅ 骨架 |
| 8 | `/pages/video-detail/index` | 视频详情 | ✅ 骨架 |
| 9 | `/pages/search/index` | 搜索 | ✅ 骨架 |
| 10 | `/pages/login/index` | 登录/注册 | ✅ 骨架 |

> ✅ = 页面渲染 + API 调用闭环可跑通；点击"待开发"按钮的部分是占位。

## 🔄 与后端的约定（继承 SPEC v2.0）

1. **响应格式**：`{ code: 0, msg: "ok", data: T }`，`code !== 0` 时 `request.ts` 自动 toast
2. **401 处理**：自动用 refresh_token 换 access_token，失败则跳登录
3. **BaseURL**：从 `VITE_API_BASE_URL` 环境变量注入
4. **鉴权**：除登录/注册/公开 API 外，`Authorization: Bearer <access_token>` 自动注入

## 🛠 待补项

- [ ] 后端 CORS 白名单：dev 加 `http://localhost:5173`
- [ ] 弹幕播放器（用 nvue 或 renderjs）
- [ ] 视频分片上传（>500MB 文件）
- [ ] 移动端专属 API：推送注册 / 商品下单 / 短信登录
- [ ] 真实图标替换 PIL 占位
- [ ] Netlify 部署：`dist/_redirects` 写 `/* /index.html 200`
- [ ] 移动端真机调试（Android Studio / Xcode）

## 📝 脚手架搭建过程的关键决策

1. **不用 uView UI**：uni-ui + 自写 SCSS 已够用，避免依赖第三方兼容性
3. **midButton 而不是单独发布 Tab**：uni-app 原生支持，截图里中央 + 号位置正确
4. **统一私信、商城等"待开发"页面**：用 toast 占位，避免分散精力到非核心页面
5. **API 接口命名统一 camelCase**：与后端 JSON 一致，无需前端转换
6. **静态资源 PIL 生成**：避免手工 PS，快速出占位，专业图标后续替换