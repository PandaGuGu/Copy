# =====================================================================
# CakeCake 一键运行 CLI（替代脆弱的批处理式启动脚本）
#  用法：
#    .\scripts\cakecake-run.ps1           交互菜单（实时状态 + 单服务干预）
#    .\scripts\cakecake-run.ps1 -Status   仅打印当前各服务状态后退出
#    .\scripts\cakecake-run.ps1 -Here <svc>  只启动/重试某个服务：mysql|redis|rabbit|docker|es|backend|rtmp|frontend
#  自动以管理员运行（启动 Windows 服务需要）。可带 -NoElevate 跳过自提升。
# =====================================================================
[CmdletBinding()]
param(
    [switch]$Status,          # 仅查看状态
    [string]$Here,            # 只启动单个服务
    [string]$Stop,            # 只关闭单个服务
    [switch]$StopAll,         # 一键全部停
    [switch]$NoElevate        # 不自动提升管理员
)

$ErrorActionPreference = "Continue"
$PROJECT  = "C:\Users\Administrator\Desktop\cakecake-project"
$NODE     = "C:\Users\Administrator\.workbuddy\binaries\node\versions\22.22.2\node.exe"
$NODE_MODULES = "C:\Users\Administrator\.workbuddy\binaries\node\workspace\node_modules"
$RABBIT   = "C:\rabbitmq\rabbitmq_server-4.3.0\sbin\rabbitmq-server.bat"
$ERL_HOME = "C:\Program Files\Erlang OTP"
$DOCKER_DESKTOP = "C:\Program Files\Docker\Docker\Docker Desktop.exe"
$LOG_DIR  = Join-Path $PROJECT "logs"
$BACKEND_PORT = 18080   # 本地端口（8080 被 WSL2/Hyper-V 排除段占用；8900 被 iNode 占）

function Is-Admin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    return (New-Object Security.Principal.WindowsPrincipal($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# ---- 自动提升 ----
if (-not $Status -and -not $NoElevate -and -not (Is-Admin)) {
    Write-Host "请求管理员权限以启动 Windows 服务，请在弹出的 UAC 中点击 [是]..." -ForegroundColor Cyan
    Start-Process -FilePath "powershell" -ArgumentList (
        "-NoProfile","-ExecutionPolicy","Bypass",
        "-File", $PSCommandPath,
        ($(if ($Here)   { "-Here $Here" } elseif ($Stop) { "-Stop $Stop" } elseif ($StopAll) { "-StopAll" } elseif ($Status) { "-Status" } else { "" }))
    ) -Verb RunAs
    exit
}

# ---- 工具函数 ----
function Listen($port)   { [bool](netstat -ano 2>$null | Select-String ":$port\s" | Select-String "LISTENING") }
function Write-Str($s,$c="White"){ Write-Host -NoNewline $s -ForegroundColor $c }
function Log($name,$line){ Add-Content -Path (Join-Path $LOG_DIR "$name.log") -Value "$(Get-Date -Format 'HH:mm:ss') $line" -ErrorAction SilentlyContinue }

function Get-SvcStatus {
    $rows = [ordered]@{
        "mysql"     = @{ Port=3306;   Note="服务 MySQL80" }
        "redis"     = @{ Port=6379;   Note="redis-server" }
        "rabbit"    = @{ Port=5672;   Note="RabbitMQ" }
        "es"        = @{ Port=9200;   Note="Elasticsearch(Docker)" }
        "backend"   = @{ Port=$BACKEND_PORT; Note="mini-bili(.exe)" }
        "rtmp"      = @{ Port=1935;   Note="node-media-server" }
        "frontend"  = @{ Port=8888;   Note="Vite dev" }
    }
    foreach($k in $rows.Keys){
        $p = $rows[$k].Port
        if (Listen $p) { "{0,-10} {1,-6} OK        {2}" -f $k, $p, $rows[$k].Note }
        else           { "{0,-10} {1,-6} DOWN      {2}" -f $k, $p, $rows[$k].Note }
    }
}

# ---- 各服务启动 ----
function Start-MySQL {
    if (Listen 3306){ Write-Str "MySQL(3306)  已经在运行"; return }
    $svc = Get-Service -Name MySQL80 -ErrorAction SilentlyContinue
    if ($svc -and $svc.Status -ne "Running"){ Start-Service -Name MySQL80 -ErrorAction SilentlyContinue; Start-Sleep 6 }
    if (Listen 3306){ Write-Str "MySQL(3306)  OK" Green } else { Write-Str "MySQL(3306)  FAIL(MySQL80 未启动)" Red }
}
function Start-Redis {
    if (Listen 6379){ Write-Str "Redis(6379)  已经在运行"; return }
    Start-Process redis-server -ArgumentList "--port","6379" -WindowStyle Minimized
    Start-Sleep 2
    Write-Str "Redis(6379)  已启动" Green
}
function Start-Rabbit {
    if (Listen 5672){ Write-Str "RabbitMQ(5672)  已经在运行"; return }
    $env:ERLANG_HOME = $ERL_HOME
    $erts = Get-ChildItem $ERL_HOME -Directory -Filter "erts-*" -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1
    $env:Path = ($erts.FullName + "\bin" + ";" + $ERL_HOME + "\bin;" + $env:Path)
    Start-Process -FilePath $RABBIT -WindowStyle Minimized
    Write-Str "RabbitMQ(5672)  等待 12s..."; Start-Sleep 12
    if (Listen 5672){ Write-Str "RabbitMQ(5672)  OK" Green } else { Write-Str "RabbitMQ(5672)  WARN(未就绪)" Yellow }
}
function Start-DockerDesktopInteractive {
    # 在“用户交互会话”里拉起 Docker Desktop（从提权环境用计划任务 LogonType=Interactive）
    if (Test-Path $DOCKER_DESKTOP) { return $true }
    return $false
}
function Start-DockerAndEs {
    # 1) 引擎
    $dockerUp = [bool](docker info --format '{{.ServerVersion}}' 2>$null)
    if (-not $dockerUp) {
        $svc = Get-Service -Name com.docker.service -ErrorAction SilentlyContinue
        if ($svc -and $svc.Status -ne "Running"){ Start-Service -Name com.docker.service -ErrorAction SilentlyContinue }
        if ((Test-Path $DOCKER_DESKTOP) -and (-not (Get-Process "Docker Desktop" -ErrorAction SilentlyContinue))) {
            # 交互会话拉起（计划任务，RunLevel=Limited、LogonType=Interactive）
            $t = "CakeCake_DockerLaunch"
            $a = New-ScheduledTaskAction -Execute $DOCKER_DESKTOP
            $pp = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited
            Register-ScheduledTask -TaskName $t -Action $a -Principal $pp -Force | Out-Null
            Start-ScheduledTask -TaskName $t
        }
        Write-Str "等待 Docker 引擎(≤180s)..."
        $t=0; while($t -lt 180 -and -not $dockerUp){ Start-Sleep 3; $t+=3; if(docker info --format '{{.ServerVersion}}' 2>$null){$dockerUp=$true} }
        Write-Str "Docker 引擎: $(if($dockerUp){'Ready'}else{'TIMEOUT 请手动启动 Docker Desktop'})" $(if($dockerUp){'Green'}else{'Red'})
    } else { Write-Str "Docker 引擎: Ready(已在)" Green }
    # 2) ES 容器
    if ($dockerUp -and -not (Listen 9200)) {
        $n = docker ps -a --format '{{.Names}}' 2>$null | Select-String "elasticsearch-cakecake" | Select-Object -First 1
        if ($n) { docker start elasticsearch-cakecake 2>$null | Out-Null }
        else {
            Write-Str "首次拉取 ES 镜像(约800MB，需联网)..." Cyan
            docker run -d --name elasticsearch-cakecake -p 9200:9200 -p 9300:9300 `
                -e "discovery.type=single-node" -e "xpack.security.enabled=false" `
                -e "ES_JAVA_OPTS=-Xms512m -Xmx512m" docker.elastic.co/elasticsearch/elasticsearch:8.15.0 2>$null | Out-Null
        }
        Write-Str "等待 ES(9200)..." 
        $t=0; while($t -lt 180 -and -not (Listen 9200)){ Start-Sleep 8; $t+=8 }
        Write-Str "ES(9200): $(if((Listen 9200)){'OK'}else{'FAIL'})" $(if((Listen 9200)){'Green'}else{'Red'})
    }
}
function Start-Backend {
    if (Listen $BACKEND_PORT){ Write-Str "Backend($BACKEND_PORT)  已经在运行"; return }
    # 内部连接兜底（防外部残留覆盖 .env）
    $env:MYSQL_DSN="root:123456@tcp(127.0.0.1:3306)/minibili?charset=utf8mb4&parseTime=True&loc=Local"
    $env:REDIS_ADDR="127.0.0.1:6379"; $env:REDIS_PASSWORD=""
    $env:RABBITMQ_URL="amqp://guest:guest@127.0.0.1:5672/"
    $env:ELASTICSEARCH_URL="http://127.0.0.1:9200"
    $env:HTTP_ADDR=":$BACKEND_PORT"
    $bin = "$PROJECT\bin\mini-bili"
    $tmp = "$PROJECT\bin\mini-bili.exe"
    if ((Test-Path $bin)) { if(-not (Test-Path $tmp)){ Copy-Item $bin $tmp } ; $bin = $tmp }
    if (-not (Test-Path $bin)){ Write-Str "Backend($BACKEND_PORT)  FAIL(未找到 $bin，请先 go build)" Red; return }
    Start-Process -FilePath $bin -WorkingDirectory $PROJECT -WindowStyle Minimized `
        -RedirectStandardError (Join-Path $LOG_DIR "backend.err.log") -RedirectStandardOutput (Join-Path $LOG_DIR "backend.out.log")
    Write-Str "等待 Backend($BACKEND_PORT)..."
    $t=0; while($t -lt 30 -and -not (Listen $BACKEND_PORT)){ Start-Sleep 3; $t+=3 }
    Write-Str "Backend($BACKEND_PORT): $(if((Listen $BACKEND_PORT)){'OK'}else{'WARN 见 logs/backend.err.log'})" $(if((Listen $BACKEND_PORT)){'Green'}else{'Yellow'})
}
function Start-Rtmp {
    if (Listen 1935){ Write-Str "RTMP(1935)  已经在运行"; return }
    $env:NODE_PATH = $NODE_MODULES
    Start-Process -FilePath $NODE -ArgumentList "$PROJECT\scripts\rtmp-server.js" -WorkingDirectory $PROJECT -WindowStyle Minimized
    Start-Sleep 3
    Write-Str "RTMP(1935)  已启动" Green
}
function Start-Frontend {
    if (Listen 8888){ Write-Str "Frontend(8888)  已经在运行"; return }
    $viteDir = "$PROJECT\cakecake-vue\bilibili-vue"
    Start-Process -FilePath $NODE -ArgumentList "node_modules\vite\bin\vite.js","--host","0.0.0.0","--port","8888" -WorkingDirectory $viteDir -WindowStyle Minimized
    Write-Str "等待 Frontend(8888)..."
    $t=0; while($t -lt 40 -and -not (Listen 8888)){ Start-Sleep 3; $t+=3 }
    Write-Str "Frontend(8888): $(if((Listen 8888)){'OK'}else{'WARN 见 vite 终端'})" $(if((Listen 8888)){'Green'}else{'Yellow'})
}

# ---- 关闭单个服务 ----
function Stop-Port($port){
    $ids = netstat -ano 2>$null | Select-String ":$port\s" | Select-String "LISTENING" |
        ForEach-Object { ($_ -split "\s+")[-1] } | Where-Object { $_ -match '^\d+$' } | Sort-Object -Unique
    if (-not $ids){ Write-Str "  $port 未在监听，无需停止"; return }
    foreach($id in $ids){ Stop-Process -Id $id -Force -ErrorAction SilentlyContinue | Out-Null }
    Start-Sleep 1
    Write-Str "  已停止端口 $port 的进程" Green
}
function Stop-One($name){
    switch ($name) {
        "mysql"   { Stop-Service -Name MySQL80 -Force -ErrorAction SilentlyContinue; Write-Str "MySQL80 服务已停止" Green; break }
        "es"      { docker stop elasticsearch-cakecake 2>$null | Out-Null; Write-Str "ES 容器已停止" Green; break }
        "docker"  { docker stop elasticsearch-cakecake 2>$null | Out-Null; Write-Str "已停止 ES 容器（Docker 引擎保留）" Green; break }
        "redis"   { Stop-Port 6379; break }
        "rabbit"  { Stop-Port 5672; break }
        "backend" { Stop-Port $BACKEND_PORT; break }
        "rtmp"    { Stop-Port 1935; break }
        "frontend"{ Stop-Port 8888; break }
        default   { Write-Str "未知服务: $name。可选：mysql redis rabbit docker es backend rtmp frontend" Red }
    }
}
function Stop-All {
    # 按依赖反向关闭：先应用层，后中间件/存储，最后 MySQL 服务
    foreach($s in @("frontend","rtmp","backend","docker","rabbit","redis","mysql")){
        Write-Host ("  关闭 {0} ..." -f $s) -ForegroundColor DarkGray
        Stop-One $s
    }
    Write-Host "全部服务已尝试关闭" Green
}

$S = @{
    mysql    = @{ label="MySQL(3306)";   run={ Start-MySQL } }
    redis    = @{ label="Redis(6379)";   run={ Start-Redis } }
    rabbit   = @{ label="Rabbit(5672)";  run={ Start-Rabbit } }
    docker   = @{ label="Docker+ES";     run={ Start-DockerAndEs } }
    es       = @{ label="(见 docker)";   run={ Start-DockerAndEs } }
    backend  = @{ label="Backend($BACKEND_PORT)"; run={ Start-Backend } }
    rtmp     = @{ label="RTMP(1935)";    run={ Start-Rtmp } }
    frontend = @{ label="Frontend(8888)";run={ Start-Frontend } }
}

New-Item -ItemType Directory -Force -Path $LOG_DIR | Out-Null

# ---- 单服务启动 ----
if ($Here) {
    if (-not $S.ContainsKey($Here)){ Write-Host "未知服务: $Here。可选: mysql redis rabbit docker es backend rtmp frontend" Red; exit 1 }
    Write-Host "=== 启动 $Here ===" -ForegroundColor Cyan
    & $S[$Here].run
    exit
}

# ---- 单服务关闭 ----
if ($Stop) {
    if (-not $S.ContainsKey($Stop)){ Write-Host "未知服务: $Stop。可选: mysql redis rabbit docker es backend rtmp frontend" Red; exit 1 }
    Write-Host "=== 关闭 $Stop ===" -ForegroundColor Cyan
    Stop-One $Stop
    exit
}

# ---- 一键全部停 ----
if ($StopAll) {
    Write-Host "=== 关闭全部服务 ===" -ForegroundColor Cyan
    Stop-All
    exit
}

# ---- 状态模式 ----
if ($Status) {
    Write-Host "===== CakeCake 服务状态 =====" -ForegroundColor Cyan
    Get-SvcStatus
    exit
}

# ---- 交互菜单 ----
while ($true) {
    Clear-Host
    Write-Host "===== CakeCake 运行器 =====" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "   服务状态:"
    Get-SvcStatus | ForEach-Object { Write-Host "     $_" }
    Write-Host ""
    Write-Host "   命令:"
    Write-Host "     1) 启动全部     2) 启动单个     3) 重查状态"
    Write-Host "     4) 关闭单个     5) 关闭全部     6) 查看日志     7) 退出"
    $n = Read-Host "   选择 [1-7]"
    switch ($n) {
        "1" { Start-MySQL; Start-Redis; Start-Rabbit; Start-DockerAndEs; Start-Rtmp; Start-Frontend; Start-Backend
              Write-Host "" ; Write-Host "全部执行完，按任意键返回..." -ForegroundColor Cyan; $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") }
        "2" { Write-Host "服务: mysql redis rabbit docker es backend rtmp frontend"; $s = Read-Host " 输入服务名"; if($S.ContainsKey($s)){ & $S[$s].run } else { Write-Host "未知服务" Red }; $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") }
        "3" { Write-Host "" ; "  已刷新（见上方）"; Start-Sleep 1 }
        "4" { Write-Host "服务: mysql redis rabbit docker es backend rtmp frontend"; $s = Read-Host " 输入服务名"; if($S.ContainsKey($s)){ Stop-One $s } else { Write-Host "未知服务" Red }; $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") }
        "5" { Write-Host ""; Stop-All; $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") }
        "6" { Write-Host "  logs 目录: $LOG_DIR"; Get-ChildItem $LOG_DIR -File | Select-Object Name,Length,LastWriteTime; $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") }
        "7" { break }
        default { }
    }
}