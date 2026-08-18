<#
.TokenHusk 开发环境启动脚本
同时启动后端代理 (cargo run) 和前端开发服务器 (npm run dev)
用法：在 PowerShell 中运行  .\scripts\start.ps1
#>

$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $ProjectRoot

# 设置工具链环境变量（本机 FlyEnv 路径）
$env:PATH = "D:\sofeware\FlyEnv-Data\app\nodejs\v18.20.8;D:\sofeware\FlyEnv-Data\env\rust\bin;D:\sofeware\FlyEnv-Data\app\rust\1.95.0\cargo\bin;$env:PATH"
$env:LIB = "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64"

Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║        TokenHusk · 开发环境启动                  ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# ── 1. 启动后端代理（后台进程） ──
Write-Host "[1/2] 启动后端代理 (cargo run)..." -ForegroundColor Yellow
$proxyJob = Start-Job -Name "TokenHuskProxy" -ScriptBlock {
    param($root)
    $env:PATH = "D:\sofeware\FlyEnv-Data\app\nodejs\v18.20.8;D:\sofeware\FlyEnv-Data\env\rust\bin;D:\sofeware\FlyEnv-Data\app\rust\1.95.0\cargo\bin;$env:PATH"
    $env:LIB = "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64"
    Set-Location $root
    cargo run
} -ArgumentList $ProjectRoot
Start-Sleep -Seconds 2

# ── 2. 启动前端开发服务器（前台，阻塞直到 Ctrl+C） ──
Write-Host "[2/2] 启动前端开发服务器..." -ForegroundColor Yellow
Write-Host ""
Write-Host "  前端界面 : http://localhost:5173" -ForegroundColor Cyan
Write-Host "  代理端点 : http://127.0.0.1:10520/health" -ForegroundColor Cyan
Write-Host "  按 Ctrl+C 停止所有服务" -ForegroundColor Gray
Write-Host ""
Write-Host "  ⏳ 首次启动会编译 Rust 依赖（约 3-10 分钟）" -ForegroundColor DarkYellow
Write-Host "     代理就绪后终端会显示 health check 地址" -ForegroundColor DarkYellow
Write-Host ""

try {
    npm run dev
} finally {
    Write-Host ""
    Write-Host "正在停止服务..." -ForegroundColor Yellow
    Stop-Job -Name "TokenHuskProxy" -ErrorAction SilentlyContinue
    Remove-Job -Name "TokenHuskProxy" -ErrorAction SilentlyContinue
    Write-Host "✅ 已停止" -ForegroundColor Green
}