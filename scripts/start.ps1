# ============================================================
#  Cakecake — Windows 一键启动脚本
#
#  用法:
#    .\scripts\start.ps1               # 自动启动所有服务
#    .\scripts\start.ps1 -Docker       # 使用 Docker Compose
#    .\scripts\start.ps1 -SkipFrontend # 不启动前端
# ============================================================
param([switch]$Docker, [switch]$SkipFrontend)

$ErrorActionPreference = "Continue"

# ── 自动定位项目根目录 ──
$ROOT = Split-Path -Parent (Split-Path -Parent $PSCommandPath)

# ── 环境文件检查 ──
if (-not (Test-Path "$ROOT\.env")) {
    Write-Host "[!] 未找到 .env 文件" -ForegroundColor Red
    Write-Host "    请执行: cp .env.example .env  然后编辑 JWT_SECRET" -ForegroundColor Yellow
    exit 1
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    Cakecake 一键启动" -ForegroundColor Cyan
Write-Host "    项目: $ROOT" -ForegroundColor DarkGray
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# ── Docker 模式 ──
if ($Docker) {
    Write-Host "[Docker] 启动所有容器..." -ForegroundColor Cyan
    docker compose -f "$ROOT\docker-compose.yml" up -d
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[Docker] 启动成功! http://localhost" -ForegroundColor Green
        Start-Process http://localhost
    } else {
        Write-Host "[Docker] 启动失败，请检查 Docker 是否运行" -ForegroundColor Red
    }
    exit $LASTEXITCODE
}

# ── 前置检查函数 ──
function Test-PortListening($port) {
    $r = netstat -an 2>$null | Select-String ":$port "
    return ($r -match "LISTENING")
}

function Start-And-Forget($name, $exe, $args) {
    try {
        Start-Process -FilePath $exe -ArgumentList $args -WindowStyle Minimized -WorkingDirectory $ROOT
        return $true
    } catch {
        Write-Host " FAILED ($exe 未找到)" -ForegroundColor Red
        return $false
    }
}

# ════════════════════════════════════════
#  [1/6] MySQL
# ════════════════════════════════════════
Write-Host "[1/6] MySQL (3306)..." -NoNewline
if (Test-PortListening 3306) {
    Write-Host " OK" -ForegroundColor Green
} else {
    Write-Host " 未运行 — 请手动启动 MySQL 服务" -ForegroundColor Yellow
    Write-Host "       (net start MySQL80 或 sc start MySQL80)" -ForegroundColor DarkGray
}
Start-Sleep 1

# ════════════════════════════════════════
#  [2/6] Redis
# ════════════════════════════════════════
Write-Host "[2/6] Redis (6379)..." -NoNewline
if (Test-PortListening 6379) {
    Write-Host " OK" -ForegroundColor Green
} else {
    $redisFound = $false
    # Try PATH
    $r = Get-Command redis-server -ErrorAction SilentlyContinue
    if ($r) {
        Start-And-Forget "Redis" $r.Source "--port 6379"
        $redisFound = $true
    }
    # Try common install paths
    if (-not $redisFound) {
        $paths = @(
            "C:\Program Files\Redis\redis-server.exe",
            "C:\Redis\redis-server.exe"
        )
        foreach ($p in $paths) {
            if (Test-Path $p) {
                Start-And-Forget "Redis" $p "--port 6379"
                $redisFound = $true; break
            }
        }
    }
    if ($redisFound) { Write-Host " OK" -ForegroundColor Green } else {
        Write-Host " 未安装 — 请安装 Redis for Windows" -ForegroundColor Yellow
    }
}
Start-Sleep 1

# ════════════════════════════════════════
#  [3/6] RabbitMQ
# ════════════════════════════════════════
Write-Host "[3/6] RabbitMQ (5672)..." -NoNewline
if (Test-PortListening 5672) {
    Write-Host " OK" -ForegroundColor Green
} else {
    $mqFound = $false
    # Check Windows service
    $svc = Get-Service RabbitMQ -ErrorAction SilentlyContinue
    if ($svc -and $svc.Status -ne "Running") {
        Start-Service RabbitMQ; Start-Sleep 5
        $mqFound = Test-PortListening 5672
    }
    # Try common install paths
    if (-not $mqFound) {
        $paths = @(
            "C:\Program Files\RabbitMQ Server\rabbitmq_server-*\sbin\rabbitmq-server.bat",
            "C:\rabbitmq\rabbitmq_server-*\sbin\rabbitmq-server.bat"
        )
        foreach ($p in $paths) {
            $found = Get-Item $p -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($found) {
                if ($env:ERLANG_HOME) {
                    Start-And-Forget "RabbitMQ" $found.FullName
                    Start-Sleep 10; $mqFound = $true; break
                } else {
                    Write-Host " 需要设置 ERLANG_HOME 环境变量" -ForegroundColor Yellow
                    break
                }
            }
        }
    }
    if ($mqFound) { Write-Host " OK" -ForegroundColor Green } else {
        Write-Host " 未运行 — 请手动启动 RabbitMQ" -ForegroundColor Yellow
    }
}

# ════════════════════════════════════════
#  [4/6] Go 后端
# ════════════════════════════════════════
Write-Host "[4/6] Backend (8080)..." -NoNewline
if (Test-PortListening 8080) {
    Write-Host " OK (已运行)" -ForegroundColor Green
} else {
    $bin = "$ROOT\bin\mini-bili.exe"
    if (-not (Test-Path $bin)) {
        Write-Host " 构建中..." -NoNewline -ForegroundColor Cyan
        go build -o $bin "$ROOT\cmd\mini-bili\" 2>&1 | Out-Null
        if (-not (Test-Path $bin)) {
            Write-Host " 构建失败 — 请先执行 go build -o .\bin\mini-bili.exe .\cmd\mini-bili\" -ForegroundColor Red
        }
    }
    if (Test-Path $bin) {
        Start-And-Forget "Backend" $bin
        Start-Sleep 4
        if (Test-PortListening 8080) {
            Write-Host " OK" -ForegroundColor Green
        } else {
            Write-Host " 启动中（请检查 logs/ 目录）" -ForegroundColor Yellow
        }
    }
}

# ════════════════════════════════════════
#  [5/6] RTMP 直播服务器
# ════════════════════════════════════════
Write-Host "[5/6] RTMP Server (1935)..." -NoNewline
if (Test-PortListening 1935) {
    Write-Host " OK (已运行)" -ForegroundColor Green
} else {
    $rtmpScript = "$ROOT\scripts\rtmp-server.js"
    if (Test-Path "$ROOT\scripts\node_modules\node-media-server") {
        $nodeBin = (Get-Command node -ErrorAction SilentlyContinue).Source
        if ($nodeBin) {
            Start-And-Forget "RTMP" $nodeBin $rtmpScript
            Start-Sleep 3
            if (Test-PortListening 1935) { Write-Host " OK" -ForegroundColor Green }
            else { Write-Host " 启动中..." -ForegroundColor Yellow }
        } else {
            Write-Host " Node.js 未安装 — 跳过直播功能" -ForegroundColor Yellow
        }
    } else {
        Write-Host " 依赖未安装 — 执行: cd scripts && npm install" -ForegroundColor Yellow
    }
}

# ════════════════════════════════════════
#  [6/6] 前端 (Vite dev server)
# ════════════════════════════════════════
if (-not $SkipFrontend) {
    Write-Host "[6/6] Frontend (8888)..." -NoNewline
    if (Test-PortListening 8888) {
        Write-Host " OK (已运行)" -ForegroundColor Green
    } else {
        $vueDir = "$ROOT\cakecake-vue\bilibili-vue"
        if (-not (Test-Path "$vueDir\node_modules")) {
            Write-Host " 安装依赖中..." -ForegroundColor Cyan
            npm install --prefix $vueDir 2>&1 | Out-Null
        }
        $nodeBin = (Get-Command node -ErrorAction SilentlyContinue).Source
        if ($nodeBin) {
            Start-Process -FilePath $nodeBin -ArgumentList "node_modules\vite\bin\vite.js","--host","0.0.0.0","--port","8888" `
                -WorkingDirectory $vueDir -WindowStyle Minimized
            Start-Sleep 4
            Write-Host " OK" -ForegroundColor Green
        } else {
            Write-Host " Node.js 未安装" -ForegroundColor Red
        }
    }
} else {
    Write-Host "[6/6] Frontend — 已跳过 (-SkipFrontend)" -ForegroundColor DarkGray
}

# ════════════════════════════════════════
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  启动完成!" -ForegroundColor Green
Write-Host "  前端:  http://localhost:8888" -ForegroundColor Cyan
Write-Host "  后台:  http://localhost:8888/#/admin" -ForegroundColor Cyan
Write-Host "  API:   http://localhost:8080/api/v1/health" -ForegroundColor DarkGray
Write-Host "========================================" -ForegroundColor Cyan

# 自动打开浏览器
Start-Process http://localhost:8888
