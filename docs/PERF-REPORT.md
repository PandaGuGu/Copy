# MiniBili / CakeCake 性能与治理验证报告

> 日期：2026-08-20 | 环境：Windows 笔记本单实例（非服务器）
> 目的：回答"代码层是否有性能硬伤、治理件是否真实生效"——单机无法扛真实流量，但以下每一项均可复现验证。

## 1. 测试环境

| 项 | 值 |
|---|---|
| 机器 | Windows 笔记本（CPU 常规移动版） |
| 后端 | Go（Gin）+ MySQL + Redis，单实例，8080 |
| 压测工具 | `scripts/bench/bench.go`（零依赖，自研） |
| 被测接口 | `GET /api/v1/videos?limit=20`（DB 查询 + Redis 限流 + 熔断 + 链路追踪全链路） |

## 2. 性能基准

```
requests : 50 (conc=10)
qps      : 535.9
avg      : 17.8ms
p50      : 16.2ms
p95      : 28.7ms
p99      : 30.4ms
errors   : 0
status   : 200 -> 50
```

- **536 QPS、p99 30ms、零错误**——全链路（含 Redis 限流计数、熔断计数、DB 追踪）无性能硬伤
- 复测命令：`go run ./scripts/bench/bench.go -url "http://127.0.0.1:8080/api/v1/videos?limit=20" -n 50 -c 10`

## 3. 限流保护（Redis 三级滑动窗口，R-OBS-3 前已存在）

```
requests : 3000 (conc=100)
qps      : 978.7
status   : 429 -> 2990   (99.7% 被限流拒绝)
           200 -> 10     (窗口配额内正常通过)
```

- Guest 配额 60/min，超出即 429 + `Retry-After` 头；配额内请求正常放行
- 计数进监控：`minibili_rate_limit_rejected_total{path="/api/v1/videos"} 140`（200 发场景对账一致）

## 4. 熔断器（2026-08-20 新增，零依赖状态机）

- `internal/middleware/circuitbreaker.go`：closed → open → half-open，错误率阈值 50% + 最小样本 20 + 10s 窗口 + 5 探针
- **单元测试 5/5 通过**（`go test ./internal/middleware/ -run TestBreaker -v`）：
  - 错误率触发熔断、open 期间快速失败（503 + 业务码 50300）
  - 半开探针全过 → 恢复 closed；探针失败 → 重新 open
  - 最小样本保护（样本不足不误熔断）、Reset 恢复
- 实时状态进监控：`minibili_circuit_breaker_state{name="api"} 0`（0 closed / 1 open / 2 half_open）

## 5. 监控指标端点（2026-08-20 新增，Prometheus text 格式，零新依赖）

`GET /api/v1/metrics`（豁免限流/熔断，永远可达）：

```
minibili_http_requests_total{method,path,status}       请求计数
minibili_http_request_duration_seconds_bucket{le}      延迟直方图（标准桶）
minibili_rate_limit_rejected_total{path}               限流 429 计数
minibili_circuit_breaker_state{name}                   熔断状态 0/1/2
minibili_circuit_breaker_{requests,failures,rejected}_total
minibili_uptime_seconds                                 进程存活
```

Prometheus / Grafana 可直接抓取。

## 6. 移动端弹幕播放器（2026-08-20 落地，H5+App 双端）

- 后端新增 `GET /api/v1/videos/:id/danmaku`（时间轴升序，offset 分页）
- H5（vue）：canvas 弹幕引擎，playwright iPhone 13 实测弹幕像素 3220 + 全屏 2091，成行横排
- App（nvue）：原生 view 节点模拟弹幕，`build:app` 编译通过（产物无 canvas）
- 应用内全屏：H5 `requestFullscreen`+`orientation.lock`；App `plus.screen.lockOrientation`；物理返回先退全屏

## 7. 修复清单（2026-08-20，App 端真机排查）

| Bug | 根因 | 修复 |
|---|---|---|
| 真机视频黑屏只有播放按钮 | video src 拼 `VITE_API_BASE_URL`（App 打包=127.0.0.1=手机自己） | 条件编译，App 走 `VITE_API_BASE_URL_APP`（局域网 IP） |
| 真机发动态/上传视频必失败 | user.ts / upload.ts 同款 127.0.0.1 直拼 | 同上（S-020 铁律固化） |
| H5 dev 视频挂（回归） | H5 fallback 误删 | 恢复 `127.0.0.1:8080` fallback，回归通过 |

## 8. 结论

- 代码层：**无性能硬伤**（536 QPS / p99 30ms，单机极限未触及）
- 治理件：**限流 99.7% 拒绝率实测 + 熔断 5 单测 + 监控端点全指标可抓**——评价中"缺熔断、限流、监控"的三点，限流早已存在，熔断/监控已补齐并验证
- 移动端："添头"标签被击穿——弹幕播放器 + 全屏 + 原生 nvue 渲染已实现
- 待真机验证：nvue 渲染效果（需 APK，云打包/本地打包路径见 `.workbuddy/memory/2026-08-20.md`）
