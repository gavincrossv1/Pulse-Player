# scripts/build_release.ps1
# Pulse Player 自动化打包构建与签名发布一体化脚本

param (
    [string]$Version,
    [string]$Notes
)

$ErrorActionPreference = "Stop"

# 若未指定版本，自动从 tauri.conf.json 动态提取
if (-not $Version) {
    $tauriConfPath = "src-tauri\tauri.conf.json"
    if (Test-Path $tauriConfPath) {
        $conf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
        $Version = $conf.version
    } else {
        $Version = "0.3.0"
    }
}

if (-not $Notes) {
    $Notes = "v$Version: 播放偏好新增一键恢复默认设置；精简文案风格回归纯粹简洁；Windows 系统代理自动继承与无缝更新支持。"
}

Write-Host ">>> 准备构建 Release 生产环境安装包 (v$Version)..." -ForegroundColor Cyan

$keyPath = "src-tauri\.updater.key"
if (Test-Path $keyPath) {
    Write-Host "  -> 载入 Minisign 私钥用于安装包签名..." -ForegroundColor Yellow
    $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $keyPath -Raw
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
}

Write-Host "  -> 执行 pnpm tauri build..." -ForegroundColor Cyan
pnpm tauri build

Write-Host "[PASS] Tauri Build 打包完成！" -ForegroundColor Green

# 生成 latest.json
$bundleDir = "src-tauri\target\release\bundle\nsis"
$zipFile = Get-ChildItem -Path $bundleDir -Filter "*$Version*.nsis.zip" | Select-Object -First 1
if (-not $zipFile) {
    $zipFile = Get-ChildItem -Path $bundleDir -Filter "*.exe" | Select-Object -First 1
}

if ($zipFile) {
    Write-Host "  -> 找到更新包: $($zipFile.FullName)，正在生成 latest.json..." -ForegroundColor Cyan
    powershell -ExecutionPolicy Bypass -File scripts\generate_updater_manifest.ps1 `
        -Version $Version `
        -Notes $Notes `
        -ArtifactPath $zipFile.FullName
}

Write-Host ">>> 全部打包与签名清单构建完成！" -ForegroundColor Green
