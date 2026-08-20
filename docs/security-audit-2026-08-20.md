# CakeCake 项目安全审计报告

> 扫描时间：2026-08-20 19:58 | 扫描工具：gosec v2.28.0 / govulncheck v1.7.0 / npm audit / 敏感信息扫描
> 技术栈：Go 1.24.5 后端 + Vue 前端（bilibili-vue + cakecake-app uni-app）

---

## 一、结论速览

| 扫描项 | 结果 | 严重度 |
|--------|------|--------|
| Go 依赖 + 标准库漏洞（govulncheck） | **36 个实际影响代码** | 🔴 高 |
| 前端依赖漏洞（npm audit） | bilibili-vue 11 个（9 high）；cakecake-app 56 个（13 high） | 🟠 中高 |
| Go 静态安全（gosec） | 47 项（1 HIGH 误报 + 27 MEDIUM + 19 LOW） | 🟡 中低 |
| 敏感信息硬编码 | **干净**（.env 已 gitignore，源码无密钥） | ✅ |

---

## 二、govulncheck：Go 依赖漏洞（最高优先）

**36 个漏洞实际影响你的代码路径**，另 17 个在 import 包中、29 个在 require 模块中但未被调用（不紧急）。

### 2.1 Go 标准库（go1.24.5 过旧）— 20+ 个 CVE

`crypto/tls`、`net/http`、`net/url`、`crypto/x509`、`html/template`、`encoding/xml`、`os/exec`、`database/sql` 等多个标准库包存在已知漏洞，典型：

- GO-2026-6089 net/http | GO-2026-6090/5856 crypto/tls | GO-2025-4175 crypto/x509
- GO-2025-3956 os/exec（ffmpeg 调用链）| GO-2025-3849 database/sql（BI 统计查询链）

**修复：升级 Go 工具链到 ≥ 1.25.13，一次性全部覆盖。**

### 2.2 第三方模块（需 go get 升级）

| 模块 | 当前 | 修复版本 | 风险 |
|------|------|----------|------|
| golang.org/x/net | v0.26.0 | ≥ v0.55.0 | XSS（markdown 渲染链）GO-2025-3595 |
| golang.org/x/text | v0.19.0 | ≥ v0.39.0 | 字符处理 |
| github.com/yuin/goldmark | v1.7.8 | ≥ v1.7.17 | markdown 解析 |
| **github.com/golang-jwt/jwt/v5** | v5.2.1 | **≥ v5.2.2** | **JWT 解析内存耗尽 DoS（鉴权核心）** |
| github.com/redis/go-redis/v9 | v9.6.1 | ≥ v9.6.3 | 连接建立乱序响应 |

---

## 三、npm audit：前端依赖漏洞

### 3.1 bilibili-vue（PC 端）— 11 个（2 moderate / 9 high）

| 包 | 严重度 | 问题 |
|----|--------|------|
| axios | high | SSRF 代理绕过、原型污染、formData 递归 DoS（≥1.0.0 <1.18.0） |
| nanoid | high | 非安全生成器负 size 死循环 |
| postcss | high | sourceMappingURL 路径穿越任意 .map 文件泄露 |
| vite | high | Windows 下 server.fs.deny 绕过、NTLMv2 hash 泄露 |

→ `npm audit fix` 可修复（无破坏性变更）

### 3.2 cakecake-app（移动端）— 56 个（24 low / 19 moderate / 13 high）

主要为 @dcloudio（uni-app）构建工具链 + ws + jest + jimp 传递依赖。
→ `npm audit fix --force` 会升级 @dcloudio 大版本（破坏性），**建议评估后分步处理**；不影响运行时安全的部分可暂缓。

---

## 四、gosec：Go 静态安全（47 项）

### 4.1 HIGH（1 项）— 已确认为误报

- `internal/errcode/errcode.go:43-81` G101 硬编码凭据：实际只是错误码→中文消息映射表，"密码"关键词触发误报，**无真实凭据**。

### 4.2 MEDIUM（27 项）

| 规则 | 数量 | 位置 | 实际风险 |
|------|------|------|----------|
| G204 子进程参数 | 5 | ffmpeg.go×4、subtitle_asr.go×1 | **低**。均为 `exec.Command(可执行名, 参数切片)`，不经 shell，无命令注入面；路径参数可能有穿越面（结合 G304） |
| G304 文件包含变量 | 6 | storage/local.go×3、oss.go、video.go:281、migrate_tool.go | **需人工复核**。上传文件路径是否净化文件名（重点看 video.go 上传流程） |
| G301 目录权限 0755 | 20 | 各 handler/worker MkdirAll | 低。Windows 环境基本无影响，规范建议统一 0750 |
| G306 文件权限 0644 | 1 | transcode.go:294 | 低，建议 0600 |

### 4.3 LOW（19 项）

以 G104 未检查错误为主，非安全风险，日常 code review 处理。

---

## 五、敏感信息扫描 — 干净 ✅

- `internal/`、`cmd/`、`configs/`、`cakecake-app/src`：无硬编码 API key / 密码 / token
- `.env` 已被 `.gitignore` 排除，git 仅跟踪 `.env.example` 模板
- 无 .pem / .key / id_rsa 被提交

---

## 六、修复优先级建议

### P0 — 立即修复（低风险高收益）
```bash
# 后端依赖升级
go get golang.org/x/net@v0.55.0 golang.org/x/text@v0.39.0 \
       github.com/yuin/goldmark@v1.7.17 \
       github.com/golang-jwt/jwt/v5@v5.2.2 \
       github.com/redis/go-redis/v9@v9.6.3
go mod tidy && go build -o ./bin/mini-bili ./cmd/   # S-001 验证

# PC 前端
cd cakecake-vue/bilibili-vue && npm audit fix
```

### P1 — 计划内
- **升级 Go 工具链 1.24.5 → ≥1.25.13**（一次性覆盖 20+ 标准库 CVE；注意验证编译兼容性）
- cakecake-app 依赖：评估 `npm audit fix --force`（@dcloudio 破坏性升级）或接受风险暂缓

### P2 — 人工复核
- G304 路径穿越 6 处：确认上传文件名净化逻辑（video.go / storage 层）
- G301/G306：文件权限统一收紧为 0750/0600
- 建议升级后重跑 govulncheck 确认归零

---

## 七、原始报告文件

- `data/gosec-report.json` — gosec 完整 JSON
- `data/govulncheck-report.txt` — govulncheck 完整输出（首轮）
- `data/govulncheck-report-v3.txt` — govulncheck 完整输出（修复后，0 漏洞）

---

## 八、修复结果（2026-08-20 晚）

| 项 | 修复动作 | 结果 |
|----|----------|------|
| Go 依赖 | `go get` 升级 x/net v0.55.0、x/text v0.39.0、goldmark v1.7.17、jwt/v5 v5.2.2、go-redis v9.6.3（连带 x/crypto v0.51.0、x/sys v0.45.0），go directive 1.22.0 → 1.25.0 | ✅ 第三方漏洞清零 |
| Go 标准库 | `go mod edit -toolchain=go1.25.13`，S-001 编译通过 | ✅ 标准库 29 个 CVE 清零 |
| bilibili-vue | `npm audit fix`（axios/nanoid/postcss/vite） | ✅ 11 → 0 |
| govulncheck 终扫 | `Your code is affected by 0 vulnerabilities` | ✅ 归零（剩余 15 个未调用，无实际影响） |
| Skill.md | 修正 S-001：构建路径 `./cmd/` → `./cmd/mini-bili`；新增 vendor 同步步骤 | ✅ |

**注意**：修复后项目要求 Go ≥ 1.25.0（go.mod directive），本地 go 命令会自动下载 toolchain go1.25.13，无需手动安装。
