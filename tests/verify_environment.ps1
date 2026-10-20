# tests/verify_environment.ps1
# 验证 Pulse Player 开发环境完整性与 Rust FFI 对 libmpv 的调用能力

$ErrorActionPreference = "Stop"

Write-Host ">>> 开始运行开发环境自动化测试..." -ForegroundColor Cyan

# 测试 1: 验证 Node.js 与 pnpm
$nodeVer = node --version
$pnpmVer = pnpm --version
if (-not $nodeVer -or -not $pnpmVer) {
    throw "测试失败: Node.js 或 pnpm 不可用"
}
Write-Host "[PASS] Node.js ($nodeVer) & pnpm ($pnpmVer) 可用" -ForegroundColor Green

# 测试 2: 验证 Rust 编译器
$rustVer = rustc --version
if (-not $rustVer) {
    throw "测试失败: rustc 不可用"
}
Write-Host "[PASS] Rust 编译器 ($rustVer) 可用" -ForegroundColor Green

# 测试 3: 验证 libs/mpv 开发文件
$projectRoot = Split-Path -Parent $PSScriptRoot
$mpvDll = Join-Path $projectRoot "libs\mpv\libmpv-2.dll"
$mpvLib = Join-Path $projectRoot "libs\mpv\mpv.lib"
$mpvHeader = Join-Path $projectRoot "libs\mpv\include\mpv\client.h"

if (-not (Test-Path $mpvDll) -or -not (Test-Path $mpvLib) -or -not (Test-Path $mpvHeader)) {
    throw "测试失败: libs/mpv 依赖文件不完整"
}
Write-Host "[PASS] libs/mpv 头文件及库文件完整存在" -ForegroundColor Green

# 测试 4: 编译并运行实际的 Rust - libmpv FFI 测试程序
$testTempDir = Join-Path $env:TEMP "pulse_player_env_test"
if (Test-Path $testTempDir) {
    Remove-Item -Recurse -Force $testTempDir
}
New-Item -ItemType Directory -Force -Path $testTempDir | Out-Null

$rustTestCode = @'
// 最小化 FFI 声明，验证与 libmpv-2.dll 的链接与函数调用
#[link(name = "mpv")]
extern "C" {
    fn mpv_client_api_version() -> std::os::raw::c_ulong;
}

fn main() {
    unsafe {
        let version = mpv_client_api_version();
        let major = version >> 16;
        let minor = version & 0xFFFF;
        println!("LIBMPV_API_VERSION={}.{}", major, minor);
        assert!(major >= 2, "mpv API version major 应该 >= 2");
    }
}
'@

$testRsPath = Join-Path $testTempDir "test_mpv_ffi.rs"
$testExePath = Join-Path $testTempDir "test_mpv_ffi.exe"
Set-Content -Path $testRsPath -Value $rustTestCode

# 复制 DLL 至测试目录以供动态加载
Copy-Item -Path $mpvDll -Destination (Join-Path $testTempDir "libmpv-2.dll") -Force

# 调用 rustc 进行链接测试
$libsMpvDir = Join-Path $projectRoot "libs\mpv"
rustc $testRsPath -L $libsMpvDir -o $testExePath

# 执行生成的测试程序
$output = & $testExePath
if ($output -notmatch "LIBMPV_API_VERSION=2\.") {
    throw "测试失败: FFI 调用输出异常: $output"
}
Write-Host "[PASS] Rust - libmpv FFI 动态链接与调用测试通过: $output" -ForegroundColor Green

# 清理测试临时文件
Remove-Item -Recurse -Force $testTempDir

Write-Host ">>> 所有环境测试全部通过！[OK]" -ForegroundColor Green
