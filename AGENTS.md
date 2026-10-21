# 项目说明

## 项目简介
Pulse Player（脉冲播放器）是一款专为 Windows 平台打造的现代本地离线媒体播放器。项目立足于**第一性原理**与 **KISS 原则**，杜绝弹幕、在线登录与云端拉流，旨在**提供丝滑流媒体级别的跟手交互手感与原生硬件直解性能**。
架构采用 **Tauri 2.0 (Rust) + libmpv 原生解码引擎 + Vue 3 / TypeScript**。

## 目录结构
```text
播放器2/
├── docs/                   # 项目设计与开发文档
│   ├── DESIGN.md           # 产品与交互设计规范
│   └── DEVELOPMENT.md      # 开发技术栈与构建指南
├── libs/                   # 第三方原生开发库
│   └── mpv/                # libmpv 头文件、导入库 (mpv.lib) 及动态库
├── scripts/                # 自动化运维与环境脚本
│   └── setup_env.ps1       # 开发环境一键检测与配置脚本
├── tests/                  # 自动化测试用例
│   └── verify_environment.ps1 # 环境与 FFI 接口自动化验证
├── AGENTS.md               # 协同规范与项目总览
└── .gitignore              # Git 忽略配置规则
```

## 常用命令
```powershell
# 1. 开发环境初始化与依赖拉取
powershell -ExecutionPolicy Bypass -File scripts\setup_env.ps1

# 2. 运行自动化全量测试 (FFI 环境 + Rust 单元测试 + 前端编译 + 安装包部署与 DLL 验证)
powershell -ExecutionPolicy Bypass -File tests\run_all_tests.ps1

# 3. 安装前端依赖
pnpm install

# 4. 启动开发模式 (热重载)
pnpm tauri dev

# 5. 编译打包生成 Windows 可执行程序
pnpm tauri build
```

## 开发规范（必须遵守）

1. 每次功能改动完成后，必须创建一个对应的 git commit，
   commit 信息用中文简明描述本次改动，便于后续追踪和回滚。
2. 每次功能改动，必须同时编写或更新对应的测试，
   并在回复"已完成"之前实际运行测试，确认全部通过。
   未通过测试的改动不得交付。(但如果改动和功能实现无关,就不必测试,比如改动的是文档)
3. 请务必保证最终代码的简洁优雅,如果一段代码在起初写下之后被发现是有误的且和现有代码矛盾,请删除,不要出现冗余代码,更不要出现那种重构完后新代码和旧代码都在情况,此时应该删除旧代码
4. 在随后的修改中不断完善该文档

## 关键机制说明
1. **播放生命周期与自然播放结束 (EOF) 机制**：
   在 mpv 启用 `keep-open=yes` 配置时，媒体自然播放至尾帧不会被卸载，底层不会派发 `MPV_EVENT_END_FILE`，而是通过属性 `eof-reached` 切换为 `true`。后端通过双客户端监听 `eof-reached` 变动并在首次自然播完时向前端派发 `player:ended` 事件，以正确驱动“自动切集”、“单集循环”、“列表循环”等逻辑。
2. **光标隐藏作用域隔离**：
   静止自动隐藏光标只作用于视频视口（`.player-viewport`），绝对不作用于物理顶栏（`TitleBar`）。顶栏作为标准桌面窗口交互层，光标永远可见且不受视频控制层隐藏影响。
3. **四位一体统一品牌视觉 (Pulse Mark)**：
   在应用程序图标（.ico / 任务栏 / 快捷方式）、Windows 资源管理器关联文件图标、主页 Hero 品牌标、纯音频播放黑胶中心统一使用纯原创的 Pulse Mark 矢量与位图资产，彻底消除任何第三方版权与商标争议风险。
4. **视频视口物理安全边距机制**：
   mpv 底层绑定整个 Windows 顶层窗口 HWND。在非全屏窗口模式下，mpv 默认从 `y=0` 绘制会导致顶部画面被前端独立常驻的 40px `TitleBar` 遮挡。通过动态设置 mpv 原生属性 `video-margin-ratio-top = 40.0 / window.innerHeight`，将视频物理渲染有效范围精准限制在顶栏下方（`y >= 40px`），保持宽高比且零裁切；在全屏模式下将边距重置为 `0.0` 实现 100% 沉浸全屏铺满。
5. **用户数据 Schema Versioning 与平滑自愈演进**：
   持久化存储在 `%APPDATA%\Pulse-Player\history.json` 中统一包裹 `HistoryData { version: 1, records }`。读取阶段向下兼容旧版裸数组，识别旧格式时自动自愈升级并持久化，保障后续数据结构演进的向下兼容。
6. **Windows 原生文件关联透视与合规引导机制**：
   严格遵循 Windows 10/11 的 UserChoice 安全防护机制，后端通过 Win32 注册表 API 毫秒级读取各后缀的 ProgId 并解析出友好软件名；彻底废弃容易导致“仅一次”弹窗与误播放空文件的 legacy `OpenAs_RunDLL` 占位调用，改为通过 `ms-settings:defaultapps?registeredAppUser=PulsePlayer` 原生深度跳转定位到 Pulse Player 专属管理页引导合规设置，并在单实例 IPC 及启动阶段对 0 字节无效文件与探测占位文件实施严格过滤。
7. **Tauri 2 原生内置静默更新 (Tauri Updater) 闭环与原地覆盖机制**：
   杜绝跳转外部网页手动下载的割裂体验。前端通过 `@tauri-apps/plugin-updater` 异步检测 `latest.json` 官方清单并内置全量下载进度条（百分比、已下载大小 / 总大小）；下载完成自动校验 Minisign (Ed25519) 防篡改数字签名；Windows 下配置 `installMode = "passive"`，安装触发时主进程调用 `install({ restartAfterInstall: true })` 原地优雅退出释放文件锁，由 NSIS 在当前程序运行目录精准执行原地覆盖静默安装并自拉起，彻底杜绝多份路径残留与文件锁占用冲突。

