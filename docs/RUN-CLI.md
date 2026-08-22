# Cakecake 一键运行 CLI 使用示例

> 统一启动/停止/观测 Cakecake 全栈（MySQL / Redis / RabbitMQ / Elasticsearch(Docker) / Backend / RTMP / Frontend）的交互式命令工具。
> 入口文件：`scripts/cakecake-run.ps1`，桌面的 `启动Cakecake.ps1` 已收敛为它的薄包装。

## 快速开始

双击桌面的 `启动Cakecake.ps1`（或直接运行 CLI 本体），会先请求管理员权限（UAC 点"是"，用于启动 Windows 服务），随后进入交互面板：

```
===== CakeCake 运行器 =====

   服务状态:
     mysql      3306   OK        服务 MySQL80
     redis      6379   OK        redis-server
     rabbit     5672   OK        RabbitMQ
     es         9200   OK        Elasticsearch(Docker)
     backend    18080  OK        mini-bili(.exe)
     rtmp       1935   OK        node-media-server
     frontend   8888   OK        Vite dev

   命令:
     1) 启动全部     2) 启动单个     3) 重查状态
     4) 关闭单个     5) 关闭全部     6) 查看日志     7) 退出
   选择 [1-7]
```

## 命令行用法

以下均可在 PowerShell 中直接执行：

```powershell
# 仅查看当前各服务状态（无需管理员）
powershell -File scripts\cakecake-run.ps1 -Status

# 只启动/重试单个服务（自动提升管理员，可加 -NoElevate 跳过）
powershell -File scripts\cakecake-run.ps1 -Here backend
powershell -File scripts\cakecake-run.ps1 -Here docker -NoElevate

# 只关闭单个服务
powershell -File scripts\cakecake-run.ps1 -Stop backend

# 一键启动全部服务
powershell -File scripts\cakecake-run.ps1            # 进入菜单后选 1）

# 一键全部停止（按依赖反向：frontend→rtmp→backend→docker→rabbit→redis→mysql）
powershell -File scripts\cakecake-run.ps1 -StopAll
```

## 服务名取值

`mysql` `redis` `rabbit` `docker` `es` `backend` `rtmp` `frontend`

## 关键行为与约定

| 项 | 说明 |
|----|------|
| 后端端口 | **18080**：本地 8080 落在 WSL2/Hyper-V 的端口排除段（`8032–8832`）无法绑定；8900 被校园网 iNode 客户端占用，故统一使用 18080。 |
| Docker 拉起 | 用计划任务（`LogonType=Interactive`）在用户交互会话启动 Docker Desktop，避免提权上下文拉不起 GUI；随后轮询 `docker info` 至引擎 Ready（最长 180s）。 |
| ES | `docker run/start elasticsearch-cakecake`（镜像 `elasticsearch:8.15.0`），首次启动需联网拉约 800MB。 |
| 依赖顺序 | `Start-All` 先中间件与存储再应用层；`StopAll` 反之，最后才停 MySQL80。后端强依赖 MySQL/Redis，缺库启动会 `Fatal`。 |
| 日志 | 各服务写入 `logs/*.log`（含 `backend.err.log`），菜单选 6) 或直接查看该目录。 |
| 幂等 | 服务已在运行会打印"已经在运行"并跳过，不会重复拉起。 |

## 一键全停示例输出

```powershell
> powershell -File scripts\cakecake-run.ps1 -StopAll -NoElevate
=== 关闭全部服务 ===
  关闭 frontend ...
  已停止端口 8888 的进程
  关闭 rtmp ...
  已停止端口 1935 的进程
  关闭 backend ...
  已停止端口 18080 的进程
  关闭 docker ...
已停止 ES 容器（Docker 引擎保留）
  关闭 rabbit ...
  已停止端口 5672 的进程
  关闭 redis ...
  已停止端口 6379 的进程
  关闭 mysql ...
MySQL80 服务已停止
全部服务已尝试关闭
```

## 说明

- 桌面 `启动Cakecake.ps1` 只有一份薄逻辑（调用 CLI），避免多副本导致的"旧脚本/空值"类问题。
- `-NoElevate` 仅用于无需管理员的操作（状态查看、后台类服务 launch 等）；启动 MySQL80 等服务需要管理员权限时请不要带它。