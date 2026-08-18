@echo off
setlocal enabledelayedexpansion

set "ProjectRoot=%~dp0.."
pushd "%ProjectRoot%"

:: 设置工具链环境变量（本机 FlyEnv 路径）
set "PATH=D:\sofeware\FlyEnv-Data\app\nodejs\v18.20.8;D:\sofeware\FlyEnv-Data\env\rust\bin;D:\sofeware\FlyEnv-Data\app\rust\1.95.0\cargo\bin;%PATH%"
set "LIB=C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64"

echo ╔══════════════════════════════════════════════════╗
echo ║        TokenHusk ^· 开发环境启动                  ║
echo ╚══════════════════════════════════════════════════╝
echo.

:: ── 1. 启动后端代理（新窗口后台运行） ──
echo [1/2] 启动后端代理 (cargo run)...
start "TokenHuskProxy" cmd /c "set PATH=%PATH% && set LIB=%LIB% && cd /d "%ProjectRoot%" && cargo run"

:: 等待几秒让 cargo 开始编译
timeout /t 3 /nobreak >nul

:: ── 2. 启动前端开发服务器（前台，阻塞直到 Ctrl+C） ──
echo [2/2] 启动前端开发服务器...
echo.
echo   前端界面 : http://localhost:5173
echo   代理端点 : http://127.0.0.1:10520/health
echo   按 Ctrl+C 停止所有服务
echo.
echo   ⏳ 首次启动会编译 Rust 依赖（约 3-10 分钟）
echo      代理就绪后终端会显示 health check 地址
echo.

call npm run dev

echo.
echo 正在停止服务...
taskkill /fi "WINDOWTITLE eq TokenHuskProxy" /f >nul 2>&1
echo ✅ 已停止

popd
endlocal