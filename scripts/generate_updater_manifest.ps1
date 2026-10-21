# scripts/generate_updater_manifest.ps1
# Pulse Player 原生更新清单 (latest.json) 自动生成与签名维护脚本

param (
    [string]$Version,
    [string]$Notes,
    [string]$ArtifactPath,
    [string]$Signature,
    [string]$PrivateKeyPath = "src-tauri\.updater.key",
    [string]$OutputFile = "latest.json",
    [string]$BaseUrl = "https://github.com/gavincrossv1/Pulse-Player/releases/download"
)

$ErrorActionPreference = "Stop"

Write-Host ">>> 正在构建 Tauri 2 原生更新清单 (latest.json)..." -ForegroundColor Cyan

# 1. 自动提取或使用指定的版本号
if (-not $Version) {
    $tauriConfPath = "src-tauri\tauri.conf.json"
    if (Test-Path $tauriConfPath) {
        $conf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
        $Version = $conf.version
    } else {
        $Version = "0.2.0"
    }
}

if (-not $Notes) {
    $Notes = "Pulse Player v$Version 发布更新：提升媒体原生硬件直解体验，提供丝滑流媒体级交互手感。"
}

# 2. 提取或生成 Minisign 签名
$finalSignature = ""
if ($Signature) {
    if (Test-Path $Signature) {
        $finalSignature = (Get-Content $Signature -Raw).Trim()
    } else {
        $finalSignature = $Signature.Trim()
    }
} elseif ($ArtifactPath -and (Test-Path $ArtifactPath)) {
    $sigFile = "$ArtifactPath.sig"
    if (Test-Path $sigFile) {
        $finalSignature = (Get-Content $sigFile -Raw).Trim()
    } elseif ($PrivateKeyPath -and (Test-Path $PrivateKeyPath)) {
        Write-Host "  -> 检测到更新包及私钥，正在调用 Tauri CLI 执行 Minisign 数字签名..." -ForegroundColor Yellow
        $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
        pnpm tauri signer sign -f $PrivateKeyPath --app-version $Version $ArtifactPath
        if (Test-Path $sigFile) {
            $finalSignature = (Get-Content $sigFile -Raw).Trim()
        }
    }
}

# 3. 构造下载 URL (对文件名中的空格实施 RFC 3986 转义)
$artifactName = if ($ArtifactPath) { [System.IO.Path]::GetFileName($ArtifactPath) } else { "Pulse-Player_${Version}_x64-setup.nsis.zip" }
$encodedArtifactName = [System.Uri]::EscapeDataString($artifactName)
$downloadUrl = "$BaseUrl/v$Version/$encodedArtifactName"

# 4. 生成符合 Tauri 2 规范的 RFC3339 日期
$pubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

# 5. 组合 JSON 数据结构（同时提供 windows-x86_64 与 windows-x86_64-nsis 以保最佳匹配兼容性）
$platformInfo = [ordered]@{
    signature = $finalSignature
    url       = $downloadUrl
}

$manifest = [ordered]@{
    version   = $Version
    notes     = $Notes
    pub_date  = $pubDate
    platforms = [ordered]@{
        "windows-x86_64"      = $platformInfo
        "windows-x86_64-nsis" = $platformInfo
    }
}

$jsonContent = $manifest | ConvertTo-Json -Depth 5
$outPath = if ([System.IO.Path]::IsPathRooted($OutputFile)) { $OutputFile } else { Join-Path (Get-Location) $OutputFile }

# 确保以无 BOM 的纯 UTF-8 编码写入 (杜绝 0xEF 0xBB 0xBF 导致 serde_json 解析报错)
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)
[System.IO.File]::WriteAllText($outPath, $jsonContent, $utf8NoBom)

Write-Host "[PASS] 成功生成规范更新清单: $outPath (版本: v$Version)" -ForegroundColor Green
Write-Host "清单内容预览:" -ForegroundColor DarkGray
Write-Host $jsonContent -ForegroundColor DarkGray
