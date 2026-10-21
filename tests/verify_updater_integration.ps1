# tests/verify_updater_integration.ps1
# Pulse Player Tauri 2 原生静默更新体系自动化验证脚本

$ErrorActionPreference = "Stop"

Write-Host ">>> 开始验证 Tauri 2 原生静默更新 (Tauri Updater) 完整配置与契约..." -ForegroundColor Cyan

# 1. 验证 tauri.conf.json 核心配置
$confPath = "src-tauri\tauri.conf.json"
if (-not (Test-Path $confPath)) {
    throw "未找到配置文件: $confPath"
}
$conf = Get-Content $confPath -Raw | ConvertFrom-Json

if ($conf.bundle.createUpdaterArtifacts -ne $true) {
    throw "校验失败: bundle.createUpdaterArtifacts 必须为 true"
}

if (-not $conf.plugins.updater) {
    throw "校验失败: plugins.updater 插件配置缺失"
}

$pubkey = $conf.plugins.updater.pubkey
if (-not $pubkey -or -not $pubkey.StartsWith("dW50cnVzdGVkIGNvbW1lbnQ")) {
    throw "校验失败: updater.pubkey 格式不符合 Minisign 公钥要求"
}

$endpoints = $conf.plugins.updater.endpoints
if (-not $endpoints -or $endpoints.Count -eq 0 -or -not $endpoints[0].EndsWith("/latest.json")) {
    throw "校验失败: updater.endpoints 必须指向规范的 latest.json 清单"
}

$installMode = $conf.plugins.updater.windows.installMode
if ($installMode -ne "passive") {
    throw "校验失败: plugins.updater.windows.installMode 必须为 'passive' 以确保原地覆盖静默安装"
}
Write-Host "[PASS] tauri.conf.json 更新配置 (产物生成、公钥、被动静默模式) 校验通过！" -ForegroundColor Green

# 2. 验证 capabilities 权限模型
$capPath = "src-tauri\capabilities\default.json"
$cap = Get-Content $capPath -Raw | ConvertFrom-Json
if (-not ($cap.permissions -contains "updater:default")) {
    throw "校验失败: capabilities 缺少 updater:default 权限"
}
if (-not ($cap.permissions -contains "process:default")) {
    throw "校验失败: capabilities 缺少 process:default 权限"
}
Write-Host "[PASS] capabilities 权限模型 (updater:default / process:default) 校验通过！" -ForegroundColor Green

# 3. 验证 generate_updater_manifest.ps1 脚本生成合规 latest.json
$tempManifest = Join-Path $env:TEMP "test_latest_manifest.json"
if (Test-Path $tempManifest) { Remove-Item $tempManifest -Force }

powershell -ExecutionPolicy Bypass -File scripts\generate_updater_manifest.ps1 -OutputFile $tempManifest -Version "0.3.0" -Notes "自动化集成测试更新版本"
if (-not (Test-Path $tempManifest)) {
    throw "校验失败: 未能成功生成更新清单: $tempManifest"
}

$genManifest = Get-Content $tempManifest -Raw | ConvertFrom-Json
if ($genManifest.version -ne "0.3.0") {
    throw "校验失败: 生成清单中的版本号不正确"
}
if (-not $genManifest.platforms."windows-x86_64") {
    throw "校验失败: 生成清单缺少 windows-x86_64 平台定义"
}
if (-not $genManifest.platforms."windows-x86_64".url.EndsWith("Pulse-Player_0.3.0_x64-setup.nsis.zip")) {
    throw "校验失败: 下载 URL 构造异常: $($genManifest.platforms.'windows-x86_64'.url)"
}
Remove-Item $tempManifest -Force -ErrorAction SilentlyContinue
Write-Host "[PASS] scripts/generate_updater_manifest.ps1 规范清单生成校验通过！" -ForegroundColor Green

# 4. 验证数字签名工具与密钥有效性
if (Test-Path "src-tauri\.updater.key") {
    $dummyPayload = Join-Path $env:TEMP "test_updater_dummy.bin"
    Set-Content -Path $dummyPayload -Value "pulse-player-updater-test-binary-blob"
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
    pnpm tauri signer sign -f src-tauri\.updater.key --app-version "0.2.0" $dummyPayload | Out-Null
    $dummySig = "$dummyPayload.sig"
    if (-not (Test-Path $dummySig)) {
        throw "校验失败: 调用 tauri signer sign 未生成签名文件 $dummySig"
    }
    $sigContent = (Get-Content $dummySig -Raw).Trim()
    $decoded = [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String($sigContent))
    if (-not $decoded.Contains("untrusted comment: signature from tauri secret key")) {
        throw "校验失败: 解密后的签名不包含 Minisign untrusted comment 标识"
    }
    if (-not $decoded.Contains("version:0.2.0")) {
        throw "校验失败: 签名 trusted comment 中未正确绑定版本号"
    }
    Remove-Item $dummyPayload, $dummySig -Force -ErrorAction SilentlyContinue
    Write-Host "[PASS] Minisign 数字签名自动化生成与版本绑定测试通过！" -ForegroundColor Green
}

Write-Host ">>> 全部 Tauri Updater 体系测试均已通过！[OK]`n" -ForegroundColor Green
