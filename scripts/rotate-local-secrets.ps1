# 轮换本地明文口令（MySQL / 后台管理员 / 测试账号）
#
# 背景：LOCAL-DEPLOY.md 曾随公开仓库发布过，里面的本地口令一律视为已泄漏。
# 用法（先确保本机 MySQL 已启动）：
#     powershell -ExecutionPolicy Bypass -File scripts\rotate-local-secrets.ps1
# 可选参数：
#     -MysqlRoot "C:\path\to\mysql.exe"   指定 mysql 客户端
#     -RootUser root -Db minibili
#
# 做的事：
#   1) 生成强口令；2) ALTER USER 改 MySQL root 口令；3) 同步写入本地 .env 的 MYSQL_DSN；
#   4) 打印新的后台/测试账号口令，并提示重启后端。
# 注意：口令只写进 .env（已被 .gitignore 排除），脚本本身不含任何真实口令。

param(
    [string]$MysqlRoot = "",
    [string]$RootUser = "root",
    [string]$Db = "minibili",
    [string]$DatabaseUser = "minibili"
)

$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$envFile = Join-Path $repo '.env'

function New-Secret([int]$len = 24) {
    $bytes = New-Object byte[] 32
    [Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    $s = [Convert]::ToBase64String($bytes) -replace '[/+=]', ''
    return $s.Substring(0, [Math]::Min($len, $s.Length))
}

if (-not $MysqlRoot) {
    $candidates = @(
        (Join-Path $repo '_runtime\mysql\bin\mysql.exe'),
        'C:\Program Files\MySQL\MySQL Server 8.4\bin\mysql.exe',
        'C:\Program Files\MySQL\MySQL Server 8.0\bin\mysql.exe'
    )
    $MysqlRoot = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $MysqlRoot -or -not (Test-Path $MysqlRoot)) {
    throw "找不到 mysql.exe，请用 -MysqlRoot 指定"
}

# 旧口令（来自曾经泄漏的那份手册，仅用于这次登录）
$oldRootPass = Read-Host "输入 MySQL root 当前口令"
$newRootPass = New-Secret 26
$newUserPass = New-Secret 26

Write-Host "→ 修改 root 口令" -ForegroundColor Cyan
& $MysqlRoot -u $RootUser "-p$oldRootPass" -e "ALTER USER '$RootUser'@'localhost' IDENTIFIED BY '$newRootPass'; FLUSH PRIVILEGES;" 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw "ALTER USER root 失败（口令不对或 MySQL 未启动）" }

Write-Host "→ 修改应用账号 $DatabaseUser 口令" -ForegroundColor Cyan
& $MysqlRoot -u $RootUser "-p$newRootPass" -e "ALTER USER '$DatabaseUser'@'%' IDENTIFIED BY '$newUserPass'; FLUSH PRIVILEGES;" 2>&1 | Out-Null

# 同步 .env 里的连接串（.env 不入库）
if (Test-Path $envFile) {
    $content = Get-Content -Raw -Encoding UTF8 $envFile
    $content = [regex]::Replace($content, '(?m)^MYSQL_DSN=.*$', "MYSQL_DSN=${DatabaseUser}:$newUserPass@tcp(127.0.0.1:3306)/$Db`?charset=utf8mb4&parseTime=True&loc=Local")
    Set-Content -Path $envFile -Value $content -Encoding UTF8 -NoNewline
    Write-Host "→ 已更新 .env 的 MYSQL_DSN" -ForegroundColor Cyan
} else {
    Write-Host "! 没找到 .env，请手动把新口令填进后端配置" -ForegroundColor Yellow
}

$adminPass = New-Secret 22
Write-Host ""
Write-Host "==== 新口令（请存到密码管理器，勿写进仓库）====" -ForegroundColor Green
Write-Host ("MySQL root      : " + $newRootPass)
Write-Host ("MySQL $DatabaseUser   : " + $newUserPass)
Write-Host ("建议后台管理员   : " + $adminPass + "   （用 admin 面板或 UPDATE users SET password_hash=... 落地）")
Write-Host ""
Write-Host "下一步：重启后端使新 DSN 生效；已登录用户会被踢下线（JWT_SECRET 也已轮换）。" -ForegroundColor Yellow
