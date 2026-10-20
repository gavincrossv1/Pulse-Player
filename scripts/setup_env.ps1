# scripts/setup_env.ps1
# Pulse Player 开发环境一键初始化与依赖检测脚本
# 负责检测开发环境并按需拉取 libmpv 预编译库

$ErrorActionPreference = "Stop"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "  Pulse Player 开发环境配置与依赖检查" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# 1. 检测 Rust 编译器
try {
    $rustVersion = rustc --version
    Write-Host "[✓] Rust 编译器: $rustVersion" -ForegroundColor Green
} catch {
    Write-Error "[✕] 未检测到 Rust 编译器，请先安装 Rust: https://rustup.rs/"
}

# 2. 检测 Node.js 与 pnpm
try {
    $nodeVersion = node --version
    $pnpmVersion = pnpm --version
    Write-Host "[✓] Node.js: $nodeVersion | pnpm: $pnpmVersion" -ForegroundColor Green
} catch {
    Write-Error "[✕] 未检测到 Node.js 或 pnpm，请确保已安装。"
}

# 3. 检查并准备 libs/mpv 依赖
$projectRoot = Split-Path -Parent $PSScriptRoot
$libsMpvDir = Join-Path $projectRoot "libs\mpv"
$targetDll = Join-Path $libsMpvDir "libmpv-2.dll"
$targetLib = Join-Path $libsMpvDir "mpv.lib"

if (-not (Test-Path $targetDll) -or -not (Test-Path $targetLib)) {
    Write-Host "[*] 检测到 libs/mpv 缺失核心二进制库，正在从官方构建源获取..." -ForegroundColor Yellow
    
    if (-not (Test-Path $libsMpvDir)) {
        New-Item -ItemType Directory -Force -Path $libsMpvDir | Out-Null
    }

    # 获取 zhongfly/mpv-winbuild 最新 releases 的 mpv-dev 包
    $apiUrl = "https://api.github.com/repos/zhongfly/mpv-winbuild/releases/latest"
    $releaseInfo = curl.exe -s -H "User-Agent: Mozilla/5.0" $apiUrl | ConvertFrom-Json
    $asset = $releaseInfo.assets | Where-Object { $_.name -like "mpv-dev-x86_64-*.7z" -and $_.name -notlike "*-v3*" } | Select-Object -First 1

    if (-not $asset) {
        throw "未能获取到 mpv-dev 预编译发布包，请检查网络连接。"
    }

    $downloadUrl = $asset.browser_download_url
    $tempArchive = Join-Path $env:TEMP $asset.name
    Write-Host "[*] 正在下载: $($asset.name)..." -ForegroundColor Yellow
    curl.exe -L -o $tempArchive $downloadUrl

    Write-Host "[*] 正在解压至 libs/mpv..." -ForegroundColor Yellow
    7z x $tempArchive "-o$libsMpvDir" -y | Out-Null

    # 适配 MSVC 链接器的导入库
    if (Test-Path (Join-Path $libsMpvDir "libmpv.dll.a")) {
        Copy-Item (Join-Path $libsMpvDir "libmpv.dll.a") -Destination $targetLib -Force
    }

    # 清理临时下载文件
    if (Test-Path $tempArchive) {
        Remove-Item -Force $tempArchive
    }
    Write-Host "[✓] libmpv 下载与配置完成！" -ForegroundColor Green
} else {
    Write-Host "[✓] libs/mpv 预编译库已就绪 (包含 libmpv-2.dll 与 mpv.lib)" -ForegroundColor Green
}

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "  环境验证全部通过，开发环境就绪！" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Cyan
