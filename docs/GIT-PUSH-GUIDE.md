# Git Push 规范与代码保护指南

> **目标**：防止未经审查的代码直接进入 `main` 分支，统一提交信息格式，自动化质量检查。

---

## 一、整体架构

```
开发者本地                          GitHub 远程仓库
┌─────────────────────┐          ┌─────────────────────────┐
│  git commit          │          │  分支保护规则 (Branch     │
│  ├─ pre-commit ✓     │          │  Protection Rules)       │
│  │  敏感信息扫描      │          │  ├─ 禁止直推 main        │
│  │  go vet           │          │  ├─ 必须 PR              │
│  │  调试代码检查      │          │  ├─ 必须 Code Review     │
│  │  大文件检查        │          │  ├─ CODEOWNERS 自动分配  │
│  ├─ commit-msg ✓     │          │  └─ CI 必须通过          │
│  │  Conventional     │          │                         │
│  │  Commits 校验     │          │  CI 检查 (ci.yml)        │
│  └────────────────   │   push   │  ├─ Go build/vet/test    │
│  git push            │ ──────►  │  ├─ Vue build            │
│  ├─ pre-push ✓       │          │  ├─ Commit lint          │
│  │  禁止直推 main    │          │  └─ Secret scan          │
│  │  go build         │          │                         │
│  │  go test          │          │  部署 (deploy.yml)       │
│  └────────────────   │          │  └─ 手动触发             │
└─────────────────────┘          └─────────────────────────┘
```

**三层拦截**：
| 层级 | 时机 | Hook | 作用 |
|------|------|------|------|
| 第一层 | `git commit` | `pre-commit` | 敏感信息、go vet、调试代码、大文件 |
| 第二层 | `git commit` | `commit-msg` | 提交信息格式校验 |
| 第三层 | `git push` | `pre-push` | 禁止直推 main、编译、测试 |

---

## 二、快速开始

### 2.1 安装 Git Hooks（每个协作者必须执行）

```bash
# 方式一：一键安装脚本（推荐）
bash scripts/install-hooks.sh

# 方式二：手动配置
git config core.hooksPath .githooks
chmod +x .githooks/*
```

### 2.2 配置 GitHub 分支保护（仓库管理员操作）

在 GitHub 仓库页面操作：

```
Settings → Branches → Add branch protection rule
```

**必选项**：

| 设置项 | 值 | 说明 |
|--------|-----|------|
| Branch name pattern | `main` | 保护 main 分支 |
| ✅ Require a pull request before merging | 开启 | 禁止直推，必须走 PR |
| └ Required approving reviews | `1` | 至少 1 人 review 通过 |
| └ ✅ Require review from Code Owners | 开启 | CODEOWNERS 自动分配 reviewer |
| └ Dismiss stale pull request approvals | 开启 | 新提交自动取消已批准的 review |
| ✅ Require status checks to pass | 开启 | CI 必须通过才能合并 |
| └ Required checks | 见下表 | 选择必须通过的 CI job |
| ✅ Require branches to be up to date | 开启 | 合并前必须 rebase 最新 |
| ✅ Do not allow bypassing the above | 开启 | 管理员也不能绕过（最严格） |

**Required status checks 选择**：
- `Backend (Go)`
- `Frontend (Vue)`
- `Commit Message Lint`
- `Secret Scan`

---

## 三、提交信息规范（Conventional Commits）

### 3.1 格式

```
<type>(<scope>): <description>

[可选 body]

[可选 footer]
```

### 3.2 Type 列表

| Type | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat(video): 添加弹幕发送功能` |
| `fix` | 修复 Bug | `fix(auth): 修复 JWT 刷新 Token 过期逻辑` |
| `docs` | 文档变更 | `docs(api): 更新 API 文档` |
| `style` | 代码格式 | `style(backend): 统一缩进为 tab` |
| `refactor` | 重构 | `refactor(user): 重构用户服务层` |
| `perf` | 性能优化 | `perf(transcode): 优化转码队列并发性能` |
| `test` | 测试 | `test(auth): 添加 JWT 单元测试` |
| `build` | 构建/依赖 | `build: 升级 Go 到 1.22` |
| `ci` | CI 配置 | `ci: 添加前端构建检查` |
| `chore` | 杂项 | `chore: 更新 .gitignore` |
| `revert` | 回滚 | `revert: feat(video): 添加弹幕发送功能` |
| `hotfix` | 紧急修复 | `hotfix(auth): 修复登录崩溃` |
| `wip` | 开发中 | `wip(video): 弹幕功能开发中` |

### 3.3 Scope 列表

```
video auth user comment danmaku upload transcode
admin api config deploy frontend backend db cache
mq oss llm feed subtitle ticket risk copyright bi
cs ops rbac player creator report dynamic
dashboard settings llm_config app
```

> scope 可省略：`chore: 更新依赖版本`
> `app` 为移动端工程（`cakecake-vue/cakecake-app/`），2026-08-20 加入白名单（此前 `feat(app)` 会被 commit-msg hook 拒绝）

---

## 四、分支管理策略

### 4.1 分支命名规范

| 分支类型 | 命名格式 | 示例 |
|----------|----------|------|
| 功能分支 | `feature/<模块>-<描述>` | `feature/danmaku-send` |
| 修复分支 | `fix/<模块>-<描述>` | `fix/auth-jwt-refresh` |
| 热修复 | `hotfix/<模块>-<描述>` | `hotfix/auth-login-crash` |
| 发布分支 | `release/v<版本>` | `release/v1.2.0` |
| 文档分支 | `docs/<描述>` | `docs/api-update` |

### 4.2 工作流

```
main (受保护，只接受 PR 合并)
 │
 ├── feature/danmaku-send     → PR → Code Review → CI 通过 → 合并
 ├── fix/auth-jwt-refresh      → PR → Code Review → CI 通过 → 合并
 └── hotfix/auth-login-crash   → PR → 紧急 Review → 合并
```

### 4.3 标准操作流程

```bash
# 1. 从 main 创建功能分支
git checkout main
git pull origin main
git checkout -b feature/your-feature

# 2. 开发 + 提交（hooks 自动检查）
git add .
git commit -m "feat(video): 添加弹幕发送功能"

# 3. 推送到远程
git push origin feature/your-feature

# 4. 在 GitHub 创建 Pull Request
#    - CODEOWNERS 自动分配 reviewer
#    - CI 自动运行检查
#    - Review 通过后合并

# 5. 合并后删除分支
git checkout main
git pull origin main
git branch -d feature/your-feature
```

---

## 五、Git Hooks 详细说明

### 5.1 pre-commit（提交前）

| 检查项 | 说明 | 失败处理 |
|--------|------|----------|
| 敏感信息扫描 | 扫描 token/密钥/密码 | 移除敏感信息后重新提交 |
| go vet | Go 静态分析 | 修复 vet 报错 |
| 调试代码 | 检测 `fmt.Println` 残留 | 替换为 zap 日志 |
| 大文件检查 | > 5MB 的文件 | 使用 Git LFS 或移除 |

### 5.2 commit-msg（提交信息校验）

- 强制 Conventional Commits 格式
- 校验 type 和 scope 是否合法
- 描述长度 5-100 字符

### 5.3 pre-push（推送前）

| 检查项 | 说明 | 失败处理 |
|--------|------|----------|
| 保护分支拦截 | 禁止直推 main/master | 改用 PR 流程 |
| go build | 编译检查 | 修复编译错误 |
| go test | 单元测试 | 修复失败的测试 |

### 5.4 跳过检查（紧急情况）

```bash
# 跳过 pre-commit + commit-msg
git commit --no-verify -m "hotfix(auth): 紧急修复"

# 跳过 pre-push
BYPASS_PUSH=1 git push origin main
```

> ⚠️ 跳过检查不推荐，仅在紧急生产故障时使用

---

## 六、CODEOWNERS 配置

文件位置：`.github/CODEOWNERS`

```
# 全局默认
*                           @PandaGuGu

# 后端
/internal/handler/          @PandaGuGu
/internal/service/          @PandaGuGu

# 前端
/cakecake-vue/              @PandaGuGu

# 基础设施（高敏感）
/.github/workflows/         @PandaGuGu
/deploy/                    @PandaGuGu
/Dockerfile                 @PandaGuGu
```

- 修改对应目录文件时，owner 自动被请求 review
- 多人协作时替换为实际的 GitHub 用户名/团队名
- 配合 Branch Protection 的 "Require review from Code Owners" 生效

---

## 七、CI 自动化检查

文件位置：`.github/workflows/ci.yml`

| Job | 触发 | 检查内容 |
|-----|------|----------|
| Backend (Go) | push + PR | go build / go vet / go test |
| Frontend (Vue) | push + PR | npm ci / npm run build |
| Commit Message Lint | PR | 提交信息格式校验 |
| Secret Scan | push + PR | TruffleHog 敏感信息扫描 |

---

## 八、安全提醒

### ⚠️ 当前仓库 Token 暴露问题

当前 Git remote URL 中嵌入了 GitHub Token：

```
origin  https://ghp_xxxx@github.com/PandaGuGu/Copy.git
```

**风险**：任何能读取 `.git/config` 的人都能获取你的 Token。

**修复建议**：

```bash
# 1. 移除 URL 中的 token
git remote set-url origin https://github.com/PandaGuGu/Copy.git

# 2. 使用 Git Credential Manager 缓存凭证
git config --global credential.helper manager

# 3. 或使用 SSH
git remote set-url origin git@github.com:PandaGuGu/Copy.git

# 4. 在 GitHub 上撤销已暴露的 Token
#    Settings → Developer settings → Personal access tokens → Delete
```

---

## 九、文件清单

| 文件 | 用途 |
|------|------|
| `.githooks/pre-commit` | 提交前检查 hook |
| `.githooks/commit-msg` | 提交信息校验 hook |
| `.githooks/pre-push` | 推送前检查 hook |
| `.github/CODEOWNERS` | 代码所有者定义 |
| `.github/workflows/ci.yml` | CI 自动化检查 |
| `scripts/install-hooks.sh` | 一键安装 hooks |
| `docs/GIT-PUSH-GUIDE.md` | 本文档 |
