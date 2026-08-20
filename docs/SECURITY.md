# SECURITY.md — 项目安全基线

> 本文件是项目安全审计的**常驻文档**，定义安全审计流程、工具链、当前基线与修复记录。
> 关联文档：`security-audit-2026-08-20.md`（首次全量审计报告）、`Rule.md`（研发制度）、`Skill.md`（S-001 编译验证等）

---

## 1. 安全审计流程

### 1.1 工具链

| 工具 | 用途 | 安装 | 状态 |
|------|------|------|------|
| **govulncheck** v1.7.0 | Go 标准库 + 依赖 CVE | `go install golang.org/x/vuln/cmd/govulncheck@latest` | ✅ 已装 `C:/Users/Administrator/go/bin` |
| **gosec** v2.28.0 | Go 静态安全扫描（SAST） | `go install github.com/securego/gosec/v2/cmd/gosec@latest` | ✅ 已装（需 go ≥1.25 toolchain 自动切换） |
| **npm audit** | 前端依赖漏洞 | npm 自带 | ✅ |
| **TruffleHog / git hooks** | 敏感信息 / Secret 扫描 | 项目 `.githooks/pre-commit` 已内置 | ✅ |

### 1.2 标准命令（全量审计）

```bash
# 1. Go 依赖 + 标准库漏洞（核心）
cd C:/Users/Administrator/Desktop/cakecake-project
"C:/Users/Administrator/go/bin/govulncheck.exe" ./... > data/govulncheck-report.txt 2>&1

# 2. Go 静态安全
"C:/Users/Administrator/go/bin/gosec.exe" -fmt=json -out=data/gosec-report.json ./...

# 3. 前端依赖
cd cakecake-vue/bilibili-vue && npm audit
cd cakecake-vue/cakecake-app && npm audit

# 4. 敏感信息（git 侧已由 pre-commit + TruffleHog 覆盖）
```

### 1.3 触发时机

- **每次依赖变更后**：`go get` / `npm install` 后必跑 govulncheck + npm audit
- **发版前**：全量四件套（§1.2）
- **每季度**：主动全量审计 + 复盘
- **收到安全公告**（如 Go 工具链新 CVE）：评估升级

---

## 2. 当前基线（2026-08-20 首轮）

| 扫描项 | 首轮结果 | 修复后（2026-08-20） | 状态 |
|--------|----------|---------------------|------|
| govulncheck | 36 个实际影响漏洞 | **0**（依赖升级 + 工具链 1.25.13） | ✅ 已清零 |
| npm audit bilibili-vue | 11 个（9 high） | **0**（npm audit fix） | ✅ 已清零 |
| npm audit cakecake-app | 56 个（13 high） | 暂缓（@dcloudio 破坏性升级） | ⏳ 待评估 |
| gosec | 47 项（1 HIGH 误报 / 27 MEDIUM / 19 LOW） | 无代码变更（G204 低危确认） | 🟡 部分待复核 |
| 敏感信息 | 干净 | 干净 | ✅ |

### 已识别的待办（P2，非阻塞）

- [ ] G304 路径穿越 6 处人工复核：`internal/storage/local.go`×3、`oss.go`、`handler/video.go:281`、`data/migrate_tool.go:82`（确认上传文件名净化逻辑）
- [ ] G301/G306 文件权限统一收紧为 0750/0600
- [ ] cakecake-app `npm audit fix --force` 评估（@dcloudio 大版本破坏性升级）
- [ ] govulncheck 剩余 15 个"未调用"漏洞持续观察（require 模块存在但代码未使用，无实际影响）

---

## 3. 修复记录

| 日期 | 内容 | 状态 |
|------|------|------|
| 2026-08-20 | 首轮全量审计，产出 `security-audit-2026-08-20.md` | ✅ |
| 2026-08-20 | P0 修复：升级 5 个 Go 模块（x/net v0.55.0、x/text v0.39.0、goldmark v1.7.17、jwt/v5 v5.2.2、go-redis v9.6.3）+ bilibili-vue `npm audit fix`（11 → 0） | ✅ |
| 2026-08-20 | Go 工具链升级：go.mod `toolchain go1.25.13`（覆盖 20+ 标准库 CVE，S-001 编译通过，govulncheck 36 → 0） | ✅ |
| 2026-08-20 | 修正 Skill.md S-001 构建路径（`./cmd/` → `./cmd/mini-bili`）+ 补充 vendor 同步步骤 | ✅ |

---

## 4. 安全红线（与 Rule.md 联动）

- R-DB-1/2/3：敏感信息走环境变量，严禁拼接 SQL（GORM 参数化）
- R-API-1/4：统一 `{code,msg,data}` 信封；写操作 + WS 必须 JWT 鉴权
- R-OBS-1/2：严禁 `fmt.Println` 进版本控制；zap 结构化日志
- git hooks 已强制：敏感信息扫描 + go vet + 大文件检测
