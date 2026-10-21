# scripts/build_release.ps1
# Pulse Player 自动化打包构建与签名发布一体化脚本

$ErrorActionPreference = "Stop"

Write-Host ">>> 准备构建 Release 生产环境安装包..." -ForegroundColor Cyan

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
$zipFile = Get-ChildItem -Path $bundleDir -Filter "*.nsis.zip" | Select-Object -First 1

if ($zipFile) {
    Write-Host "  -> 找到更新包: $($zipFile.FullName)，正在生成 latest.json..." -ForegroundColor Cyan
    powershell -ExecutionPolicy Bypass -File scripts\generate_updater_manifest.ps1 `
        -Version "0.2.0" `
        -Notes "v0.2.0: 优化文件关联管理交互与锁定状态引导，支持 Windows 规范应用程序能力模型及 Tauri 原生静默更新。" `
        -ArtifactPath $zipFile.FullName
}

Write-Host ">>> 全部打包与签名清单构建完成！" -ForegroundColor Green
