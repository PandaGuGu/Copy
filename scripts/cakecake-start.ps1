<#
  cakecake / MiniBili 一键启动脚本
  ------------------------------------------------------------------
  桌面上的 cakecake-start.cmd 会调用本文件，双击即可。
  也可以用参数：
     .\cakecake-start.ps1                启动全部（默认）
     .\cakecake-start.ps1 -Action down   停止（保留数据卷）
     .\cakecake-start.ps1 -Action restart
     .\cakecake-start.ps1 -Action status 只看状态
     .\cakecake-start.ps1 -Action logs   跟踪后端日志
     .\cakecake-start.ps1 -Rebuild       强制重新构建后端镜像
     .\cakecake-start.ps1 -SkipBuild     不自动构建前端 dist

  干的事：起 Docker 引擎 → 修 ES 需要的 vm.max_map_count
        → 补 .env → 构建前端 → compose up → 等全部 healthy → 打印地址
#>
[CmdletBinding()]
param(
    [ValidateSet('up', 'down', 'restart', 'status', 'logs')]
    [string]$Action = 'up',
    [switch]$Rebuild,
    [switch]$SkipBuild
)

# 注意：必须是 Continue。PS 5.1 会把原生命令（docker/npm/wsl）的 stderr
# 包装成 ErrorRecord，Stop 模式下会让一次正常的“镜像不存在”直接中断整个脚本。
# 所有外部命令的失败都靠 $LASTEXITCODE 判断，需要中断时显式 throw。
$ErrorActionPreference = 'Continue'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

# ── 可改配置 ────────────────────────────────────────────────
$ProjectPath = 'C:\Project\Copy'
$DockerExe   = 'C:\Program Files\Docker\Docker\Docker Desktop.exe'
# ES 需要的 vm.max_map_count，WSL2 默认只有 65530，不改 ES 会一直重启
$EsMaxMapCount = 262144
$WaitEngineSec  = 240
$WaitHealthySec = 480

$ComposeFile = Join-Path $ProjectPath 'docker-compose.yml'
$EnvFile     = Join-Path $ProjectPath '.env'
$EnvExample  = Join-Path $ProjectPath '.env.example'
$VueDir      = Join-Path $ProjectPath 'cakecake-vue\bilibili-vue'
$DistDir     = Join-Path $VueDir 'dist'

# 容器名（顺序即启动依赖顺序），用于健康检查
$Services = @(
    'cake-mysql', 'cake-redis', 'cake-rabbitmq', 'cake-es',
    'cake-backend', 'cake-srs', 'cake-frontend'
)

# 需要的镜像。国内直连 Docker Hub 基本拉不动，先走镜像站再重打标签。
$ImagePlan = @(
    'mysql:8.0',
    'redis:7-alpine',
    'rabbitmq:3.12-management-alpine',
    'docker.elastic.co/elasticsearch/elasticsearch:8.15.3',
    'nginx:alpine',
    'ossrs/srs:5',
    # 下面两个是 Dockerfile 的构建基础镜像，缺了 build 阶段照样会卡在 Docker Hub
    'golang:1.25-alpine',
    'alpine:3.21'
)
# 镜像站前缀（换源改这里，比如 docker.1ms.run / dockerproxy.cn）
$MirrorPrefix = 'docker.m.daocloud.io'

# ── 输出小工具 ──────────────────────────────────────────────
function Write-Step([string]$msg) { Write-Host ""; Write-Host "[*] $msg" -ForegroundColor Cyan }
function Write-Ok([string]$msg) { Write-Host "    OK  $msg" -ForegroundColor Green }
function Write-Note([string]$msg) { Write-Host "    ..  $msg" -ForegroundColor DarkGray }
function Write-Warn2([string]$msg) { Write-Host "    !!  $msg" -ForegroundColor Yellow }
function Write-Err([string]$msg) { Write-Host "    XX  $msg" -ForegroundColor Red }

# ── 读 .env 里的某个键，取不到就返回默认值 ─────────────────
function Get-EnvValue([string]$key, [string]$default) {
    if (-not (Test-Path $EnvFile)) { return $default }
    $m = Select-String -Path $EnvFile -Pattern ("^\s*" + $key + "\s*=\s*(.*)$") | Select-Object -First 1
    if ($null -eq $m) { return $default }
    $v = $m.Matches[0].Groups[1].Value.Trim()
    if ([string]::IsNullOrWhiteSpace($v)) { return $default }
    return $v
}

# ── Docker 引擎存活检查 ────────────────────────────────────
function Test-DockerEngine {
    $null = & docker info --format '{{.ServerVersion}}' 2>&1
    return ($LASTEXITCODE -eq 0)
}

function Ensure-DockerEngine {
    if (Test-DockerEngine) {
        Write-Ok 'Docker 引擎已在运行'
        return
    }
    if (-not (Test-Path $DockerExe)) {
        throw "找不到 Docker Desktop：$DockerExe"
    }
    Write-Step "Docker 引擎没起来，正在拉起 Docker Desktop（最多等 $WaitEngineSec 秒）"
    Start-Process -FilePath $DockerExe | Out-Null
    $deadline = (Get-Date).AddSeconds($WaitEngineSec)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Seconds 5
        Write-Host '.' -NoNewline
        if (Test-DockerEngine) {
            Write-Host ''
            Write-Ok 'Docker 引擎已就绪'
            return
        }
    }
    Write-Host ''
    throw '等待 Docker 引擎超时，手动打开一次 Docker Desktop 再试'
}

# ── 修 ES 的 vm.max_map_count ──────────────────────────────
function Repair-EsSysctl {
    Write-Step '检查 Elasticsearch 前置条件（vm.max_map_count）'
    try {
        $cur = (& wsl -d docker-desktop -u root sysctl -n vm.max_map_count 2>&1) -join ' '
        if ($cur -match '(\d+)') {
            $now = [int]$Matches[1]
            if ($now -ge $EsMaxMapCount) {
                Write-Ok "已满足：vm.max_map_count = $now"
                return
            }
            Write-Note "当前 $now，需要 $EsMaxMapCount，正在调整"
        }
        else {
            Write-Note '读不到当前值，直接尝试写入'
        }
        $null = & wsl -d docker-desktop -u root sysctl -w "vm.max_map_count=$EsMaxMapCount" 2>&1
        $after = (& wsl -d docker-desktop -u root sysctl -n vm.max_map_count 2>&1) -join ' '
        if ($after -match '(\d+)' -and [int]$Matches[1] -ge $EsMaxMapCount) {
            Write-Ok "已设置为 $EsMaxMapCount"
        }
        else {
            Write-Warn2 '设置失败，ES 可能反复重启。可改用 .wslconfig 永久生效：'
            Write-Warn2 '  %USERPROFILE%\.wslconfig 里加 [wsl2] kernelCommandLine = sysctl.vm.max_map_count=262144'
        }
    }
    catch {
        Write-Warn2 "WSL 检查跳过（$($_.Exception.Message)）"
    }
}

# ── 镜像预检（国内直连 Docker Hub 会超时，先走镜像站） ──────
function Get-MirrorName([string]$img) {
    $first = ($img -split '/')[0]
    if ($img -match '/') {
        if ($first -match '\.' -or $first -eq 'localhost') { return $null }
        return "$MirrorPrefix/$img"
    }
    return "$MirrorPrefix/library/$img"
}

function Ensure-Images {
    Write-Step '检查镜像（缺的自动拉取）'
    foreach ($img in $ImagePlan) {
        $have = (& docker image ls --quiet $img) -join ''
        if (-not [string]::IsNullOrWhiteSpace($have)) {
            Write-Ok "$img 已在本地"
            continue
        }
        $mirror = Get-MirrorName $img
        $got = $false
        if ($mirror) {
            Write-Note "拉取 $mirror"
            & docker pull $mirror
            if ($LASTEXITCODE -eq 0) {
                $null = & docker tag $mirror $img 2>&1
                $got = ($LASTEXITCODE -eq 0)
            }
        }
        if (-not $got) {
            Write-Note "镜像站失败，回退直连 $img"
            & docker pull $img
            $got = ($LASTEXITCODE -eq 0)
        }
        if (-not $got) { throw "镜像 $img 拉取失败：检查网络，或改脚本顶部的 `$MirrorPrefix" }
        Write-Ok "$img 就绪"
    }
}

# ── 前端产物 ───────────────────────────────────────────────
function Ensure-FrontendDist {
    Write-Step '检查前端静态产物'
    if (Test-Path (Join-Path $DistDir 'index.html')) {
        Write-Ok "dist 已存在：$DistDir"
        return
    }
    if ($SkipBuild) {
        Write-Warn2 'dist 不存在，但用了 -SkipBuild，前端会是白页'
        return
    }
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        Write-Warn2 '没装 Node/npm，无法构建前端。装好后重跑，或手动 npm run build'
        return
    }
    if (-not (Test-Path (Join-Path $VueDir 'node_modules'))) {
        Write-Note '首次构建，正在 npm install（会比较久）'
        Push-Location $VueDir
        try { & npm install --no-audit --no-fund } finally { Pop-Location }
        if ($LASTEXITCODE -ne 0) { throw 'npm install 失败' }
    }
    Write-Note '正在 npm run build'
    Push-Location $VueDir
    try { & npm run build } finally { Pop-Location }
    if ($LASTEXITCODE -ne 0) { throw 'npm run build 失败' }
    Write-Ok '前端构建完成'
}

# ── 等待全部容器 healthy ───────────────────────────────────
function Wait-AllHealthy {
    Write-Step "等待 7 个容器变成 healthy（最多 $WaitHealthySec 秒，首次起 ES 会慢）"
    $deadline = (Get-Date).AddSeconds($WaitHealthySec)
    $lastPending = ''
    while ((Get-Date) -lt $deadline) {
        $rows = & docker ps -a --format '{{.Names}}|{{.Status}}' 2>$null
        $map = @{}
        foreach ($r in $rows) {
            $p = $r -split '\|', 2
            if ($p.Count -eq 2) { $map[$p[0]] = $p[1] }
        }
        $pending = @()
        foreach ($n in $Services) {
            $st = $map[$n]
            if (-not $st) { $pending += "$n(未创建)"; continue }
            if ($st -match 'unhealthy' -or $st -match '^(Exited|Restarting)') {
                Write-Host ''
                Write-Err "$n 状态异常：$st"
                return $false
            }
            if ($st -notmatch '\(healthy\)') { $pending += $n }
        }
        if ($pending.Count -eq 0) {
            Write-Ok '全部 healthy'
            return $true
        }
        $line = $pending -join ', '
        if ($line -ne $lastPending) {
            Write-Host ''
            Write-Note "还在等：$line"
            $lastPending = $line
        }
        else { Write-Host '.' -NoNewline }
        Start-Sleep -Seconds 6
    }
    Write-Host ''
    Write-Err "等待超时（$WaitHealthySec 秒）"
    return $false
}

# ── 出问题时抓日志 ─────────────────────────────────────────
function Show-FailureHints {
    Write-Step '容器现状'
    & docker ps -a --filter 'name=cake-' --format 'table {{.Names}}\t{{.Status}}'
    Write-Step '可疑容器的最近日志'
    foreach ($n in $Services) {
        $st = (& docker ps -a --filter "name=^/$n$" --format '{{.Status}}' 2>$null) -join ''
        if ([string]::IsNullOrWhiteSpace($st)) { continue }
        if ($st -match 'healthy|Up' -and $st -notmatch 'unhealthy|Restarting') { continue }
        Write-Host ""
        Write-Warn2 "--- $n ($st) ---"
        & docker logs --tail 25 $n 2>&1 | ForEach-Object { Write-Host "      $_" -ForegroundColor DarkGray }
    }
}

# ── 结尾地址表 ─────────────────────────────────────────────
function Show-Endpoints {
    $fe = Get-EnvValue 'FRONTEND_PORT' '80'
    $be = Get-EnvValue 'BACKEND_PORT' '8080'
    $my = Get-EnvValue 'MYSQL_PORT' '3306'
    $rd = Get-EnvValue 'REDIS_PORT' '6379'
    $mq = Get-EnvValue 'RABBITMQ_MGMT_PORT' '15672'
    $fl = Get-EnvValue 'SRS_FLV_PORT' '8000'
    $es = Get-EnvValue 'ES_PORT' '9200'
    $adm = Get-EnvValue 'ADMIN_SEED_USERNAME' 'admin'

    $feUrl = if ($fe -eq '80') { 'http://localhost' } else { "http://localhost:$fe" }

    Write-Host ""
    Write-Host '  ────────────────────────────────────────────────────────' -ForegroundColor DarkCyan
    Write-Host '   开好了，这些地址可以直接用：' -ForegroundColor Cyan
    Write-Host ""
    Write-Host ("     前台页面    {0}" -f $feUrl) -ForegroundColor White
    Write-Host ("     后端健康    http://localhost:{0}/api/v1/health" -f $be) -ForegroundColor White
    Write-Host ("     搜索 API    http://localhost:{0}/api/v1/search?keyword=test" -f $be) -ForegroundColor White
    Write-Host ("     搜索索引    http://localhost:{0}/_cat/indices?v" -f $es) -ForegroundColor White
    Write-Host ("     RabbitMQ    http://localhost:{0}  (guest/guest)" -f $mq) -ForegroundColor White
    Write-Host ("     直播播放    http://localhost:{0}/live/livestream.flv" -f $fl) -ForegroundColor White
    Write-Host  '     直播推流    rtmp://localhost:1935/live/livestream' -ForegroundColor White
    Write-Host ("     MySQL       localhost:{0}  库 minibili" -f $my) -ForegroundColor White
    Write-Host ("     Redis       localhost:{0}" -f $rd) -ForegroundColor White
    Write-Host ""
    Write-Host ("     超级管理员账号：{0}（密码见 .env 的 ADMIN_SEED_PASSWORD）" -f $adm) -ForegroundColor DarkGray
    Write-Host '  ────────────────────────────────────────────────────────' -ForegroundColor DarkCyan
    Write-Host  '   停止：双击同目录 cakecake-stop.cmd，或 -Action down' -ForegroundColor DarkGray
}

# ── 主流程 ─────────────────────────────────────────────────
Write-Host ''
Write-Host '  ╔══════════════════════════════════════════════════════╗' -ForegroundColor DarkCyan
Write-Host '  ║        cakecake / MiniBili  一键启动                  ║' -ForegroundColor Cyan
Write-Host '  ╚══════════════════════════════════════════════════════╝' -ForegroundColor DarkCyan

# 任何未捕获的错误都只打一行红字，别甩一堆调用栈给用户
trap {
    Write-Host ''
    Write-Err $_.Exception.Message
    exit 1
}

if (-not (Test-Path $ComposeFile)) { throw "找不到 $ComposeFile" }
if ($Action -eq 'status') {
    & docker ps -a --filter 'name=cake-' --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
    exit 0
}
if ($Action -eq 'logs') {
    & docker logs -f --tail 50 cake-backend
    exit 0
}
if ($Action -eq 'down') {
    Write-Step '停止所有容器（数据卷保留）'
    Push-Location $ProjectPath
    try { & docker compose -f $ComposeFile down } finally { Pop-Location }
    Write-Ok '已停止'
    exit 0
}

Ensure-DockerEngine
Repair-EsSysctl
Ensure-Images

if (-not (Test-Path $EnvFile)) {
    Write-Step '.env 不存在，从 .env.example 复制一份'
    Copy-Item $EnvExample $EnvFile
    Write-Warn2 '用的是默认密码，正式对外前记得改 .env'
}

Ensure-FrontendDist

Write-Step '启动全部服务'
Push-Location $ProjectPath
try {
    if ($Action -eq 'restart') {
        & docker compose -f $ComposeFile down
        if ($LASTEXITCODE -ne 0) { throw 'compose down 失败' }
    }
    $upArgs = @('-f', $ComposeFile, 'up', '-d', '--remove-orphans')
    if ($Rebuild) { $upArgs += '--build' }
    & docker compose @upArgs
}
finally { Pop-Location }
if ($LASTEXITCODE -ne 0) {
    Show-FailureHints
    throw 'docker compose up 失败'
}
Write-Ok 'compose 已下发'

$ok = Wait-AllHealthy
if (-not $ok) {
    Show-FailureHints
    Write-Host ''
    Write-Err '有服务没起来，常见原因：'
    Write-Host '      - 端口被占用（本机原生 MySQL/Redis 还开着？先停掉）' -ForegroundColor DarkGray
    Write-Host '      - vm.max_map_count 没生效，ES 反复重启' -ForegroundColor DarkGray
    Write-Host '      - 磁盘/内存不足，ES 堆设太大' -ForegroundColor DarkGray
    exit 1
}

Show-Endpoints
exit 0
