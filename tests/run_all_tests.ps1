# tests/run_all_tests.ps1
# Pulse Player 自动化全量回归测试套件

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "    Pulse Player 自动化全量测试执行中" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. 运行底层环境与 FFI 接口测试
Write-Host "`n[1/5] 执行底层环境与 FFI 调用验证..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File tests\verify_environment.ps1

# 2. 运行 Rust 后端单元测试
Write-Host "`n[2/5] 执行 Rust 后端单元与集成测试 (自然排序、核心逻辑与 Tauri Updater 契约)..." -ForegroundColor Yellow
cargo test --manifest-path src-tauri\Cargo.toml
if ($LASTEXITCODE -ne 0) {
    throw "Rust 单元测试未通过"
}
Write-Host "[PASS] Rust 后端测试全部通过！" -ForegroundColor Green

# 3. 运行 Tauri Updater 原生更新体系验证
Write-Host "`n[3/5] 执行 Tauri Updater 原生静默更新配置、签名与清单生成契约验证..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File tests\verify_updater_integration.ps1
if ($LASTEXITCODE -ne 0) {
    throw "Tauri Updater 体系验证未通过"
}
Write-Host "[PASS] Tauri Updater 体系全部验证通过！" -ForegroundColor Green

# 4. 运行前端构建与 TypeScript 类型检查测试
Write-Host "`n[4/5] 执行前端 TypeScript 类型检查与打包编译测试..." -ForegroundColor Yellow
pnpm build
if ($LASTEXITCODE -ne 0) {
    throw "前端 TypeScript / Vite 构建失败"
}
Write-Host "[PASS] 前端 Vue 3 + TypeScript 构建校验全部通过！" -ForegroundColor Green

# 5. 运行安装包部署与 DLL 依赖完整性测试
Write-Host "`n[5/5] 执行 NSIS 安装包部署与 DLL 依赖完整性验证..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File tests\verify_package_installation.ps1
if ($LASTEXITCODE -ne 0) {
    throw "安装包部署与 DLL 验证未通过"
}
Write-Host "[PASS] 安装包部署与同级 DLL 完整性测试通过！" -ForegroundColor Green

Write-Host "`n==========================================" -ForegroundColor Cyan
Write-Host "  >>> 全部测试通过！项目功能完整无损！ <<<" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
