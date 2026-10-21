# tests/verify_package_installation.ps1
# 验证 NSIS 安装包的安装正确性、同级 DLL 完整性及进程起播能力

$ErrorActionPreference = "Stop"

$installerItem = Get-ChildItem "src-tauri\target\release\bundle\nsis\*_x64-setup.exe" -ErrorAction SilentlyContinue | Select-Object -Last 1
if (-not $installerItem) {
    Write-Host "[SKIP] 未找到已打包的 NSIS 安装包，跳过安装包部署测试。" -ForegroundColor Yellow
    exit 0
}
$installer = $installerItem.FullName

Write-Host ">>> 开始验证 NSIS 安装包部署与 DLL 依赖完整性..." -ForegroundColor Cyan

# 关闭可能存在的残留测试进程以避免单实例端口冲突
Stop-Process -Name "pulse-player" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "bi-player" -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

$tempDir = Join-Path $env:TEMP "pulse_player_package_test"
if (Test-Path $tempDir) {
    Remove-Item -Recurse -Force $tempDir
}

# 1. 静默安装到测试目录
Write-Host "  -> 正在静默部署至测试路径: $tempDir"
$proc = Start-Process -FilePath $installer -ArgumentList "/S /D=$tempDir" -Wait -PassThru
if ($proc.ExitCode -ne 0) {
    throw "NSIS 安装程序返回非零退出代码: $($proc.ExitCode)"
}

# 2. 校验主程序及 libmpv-2.dll 是否位于同一根目录下
$mainExe = Join-Path $tempDir "pulse-player.exe"
if (-not (Test-Path $mainExe)) {
    $mainExe = Join-Path $tempDir "bi-player.exe"
}
$mpvDll = Join-Path $tempDir "libmpv-2.dll"

if (-not (Test-Path $mainExe)) {
    throw "测试失败: 未在安装目录下找到主程序 (pulse-player.exe 或 bi-player.exe)"
}
if (-not (Test-Path $mpvDll)) {
    throw "测试失败: 未在主程序同级目录下找到 libmpv-2.dll！Windows 动态链接器将导致启动崩溃"
}
Write-Host "[PASS] 主程序与 libmpv-2.dll 成功安装于同一根目录！" -ForegroundColor Green

# 3. 运行已安装的主程序验证 PE 加载与进程存活
Write-Host "  -> 测试启动已安装的播放器进程..."
$appProc = Start-Process -FilePath $mainExe -PassThru
Start-Sleep -Seconds 3

if ($appProc.HasExited) {
    $code = $appProc.ExitCode
    Remove-Item -Recurse -Force $tempDir
    throw "测试失败: 安装后的程序 ($mainExe) 启动异常退出，ExitCode=$code (若为 -1073741515 即 DLL 缺失崩溃)"
}

Stop-Process -Id $appProc.Id -Force
[void]$appProc.WaitForExit(3000)
Start-Sleep -Milliseconds 500

# 4. 执行规范静默卸载并清理测试残留注册表，彻底杜绝污染真实用户环境
$uninstaller = Join-Path $tempDir "uninstall.exe"
if (Test-Path $uninstaller) {
    $uninstProc = Start-Process -FilePath $uninstaller -ArgumentList "/S" -Wait -PassThru -ErrorAction SilentlyContinue
}
Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Pulse Player" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Bi-Player" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\pulseplayer" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\PulsePlayer" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Applications\pulse-player.exe" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Classes\Applications\pulse-player.exe" -Recurse -Force -ErrorAction SilentlyContinue
Get-ChildItem -Path "HKCU:\Software\Classes" -ErrorAction SilentlyContinue | Where-Object { $_.PSChildName -like "PulsePlayer.AssocFile.*" } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
Remove-ItemProperty -Path "HKCU:\Software\RegisteredApplications" -Name "PulsePlayer" -Force -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\biplayer" -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "[PASS] 安装包程序启动正常，无 DLL 缺失弹窗或崩溃！[OK]" -ForegroundColor Green
