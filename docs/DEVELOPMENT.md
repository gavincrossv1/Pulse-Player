# Pulse Player 开发指南与技术栈说明 (DEVELOPMENT.md)

## 一、 技术栈选型

Pulse Player 采用 **Tauri 2.0 (Rust) + libmpv 原生解码内核 + Vue 3 / TypeScript** 的混合架构。

```
+-------------------------------------------------------------+
|              表现层 (Presentation Layer / Web)               |
|   Vue 3 + TypeScript + Tailwind CSS (B站视觉手感、动画与交互)   |
+-------------------------------------------------------------+
                              ↕ Tauri IPC / 事件通道
+-------------------------------------------------------------+
|               核心层 (Core Layer / Rust Backend)             |
|   窗口控制、文件系统扫描、单实例管理、Aero Snap 消息处理      |
+-------------------------------------------------------------+
                              ↕ FFI (C-API)
+-------------------------------------------------------------+
|               多媒体引擎 (Media Engine / libmpv)             |
|   16 类视频硬解、无级变速(0.1-10.0x)、精确Seek、音视频渲染输出 |
+-------------------------------------------------------------+
```

### 选型依据（第一性原理）
1. **B 站交互手感像素级还原**：
   - B 站网页播放器的悬浮展开进度条、移出即逝的卡片、胶囊 OSD 等交互，使用 **Vue 3 + CSS 动画** 能以极低成本做到 1:1 的丝滑还原，规避传统桌面 UI 框架繁琐笨重的样式定义。
2. **工业级解码与全格式支持**：
   - 依赖 **libmpv**（基于 FFmpeg）作为多媒体引擎，彻底解决 Chromium/WebView2 无法原生解码 `.rmvb`, `.rm`, `.avi`, `.flv`, `.wmv` 等 16 类格式的痛点，支持显卡硬件加速（d3d11va / nvdec / dxva2）及纯 CPU 软解兜底。
3. **极小体积与极致性能**：
   - 相比 Electron 的高内存与重体积，Tauri 借力系统原生 WebView2，结合 Rust 高性能后端，内存占用低（~60-80MB），启动极速。

---

## 二、 开发环境要求

| 环境组件 | 要求版本 | 用途 |
| :--- | :--- | :--- |
| **Rust** | `>= 1.80` (推荐 1.97+) | 编写 Tauri 桌面宿主与 libmpv 绑定 |
| **MSVC C++ Build Tools** | VS 2022 C++ 生成工具 | Windows 平台原生二进制编译与符号链接 |
| **Node.js & pnpm** | Node `>= 20` (当前 v24.19.0) / pnpm `>= 9` | 前端组件与交互界面构建 |
| **WebView2 Runtime** | Windows 10/11 内置或独立运行时 | Tauri 前端视口运行载体 |
| **libmpv** | `libmpv-2.dll` & `mpv.lib` (放置于 `libs/mpv/`) | 视频解码渲染内核 |

---

## 三、 常用命令

### 1. 环境准备与校验
```powershell
# 自动检测环境并拉取缺失的 libmpv 库
powershell -ExecutionPolicy Bypass -File scripts\setup_env.ps1

# 运行自动化环境完整性测试
powershell -ExecutionPolicy Bypass -File tests\verify_environment.ps1
```

### 2. 开发与调试
```powershell
# 安装前端依赖
pnpm install

# 启动开发服务器与桌面调试窗口 (热重载)
pnpm tauri dev
```

### 3. 构建与发布
```powershell
# 编译并生成 Windows 可执行安装包 / 便携包
pnpm tauri build
```

---

## 四、 目录结构设计

```text
播放器2/
├── docs/                   # 项目设计与开发文档
│   ├── DESIGN.md           # 产品与交互设计规范
│   └── DEVELOPMENT.md      # 开发技术栈与构建指南
├── libs/                   # 第三方原生开发库
│   └── mpv/                # libmpv 头文件、动态库与导入库
├── scripts/                # 自动化运维与环境脚本
│   └── setup_env.ps1       # 环境一键配置脚本
├── tests/                  # 自动化测试用例
│   └── verify_environment.ps1 # 环境与 FFI 验证测试
├── src/                    # 前端表现层源码 (Vue 3 / TS)
├── src-tauri/              # Rust 宿主层源码 (Tauri 2.0 / mpv 绑定)
└── AGENTS.md               # 协同规范与项目总览
```

---

## 五、 核心架构特性与优化设计

1. **双客户端解耦架构 (Dual-Client Decoupling)**：
   - 基于 libmpv 官方双客户端机制，后台事件线程运行在独立的 `events` client 上，主命令通道无锁等待，彻底根治 50ms 锁饥饿卡顿。
2. **乐观更新 (Optimistic UI)**：
   - 点击视口、空格与播放按钮瞬时 0ms 翻转播放/暂停状态，通过 `set_pause` 明确目标状态同步后端，杜绝交错反转，失败自动回滚。
3. **高频事件节流与松手 Seek**：
   - 进度条拖拽采用本地即时反馈 + 120ms 节流 Seek + 松手精确 Seek 机制，并在 `mouseup` 时显式调用 `cancel()` 取消挂起回调，杜绝进度倒退覆盖；音量滑块与滚轮统一节流，滚轮结合本地累加驱动杜绝快速滚动丢步。
4. **GPU 零开销画面变换**：
   - 画面水平镜像采用 GPU GLSL 着色器钩子 (`glsl-shaders`)，比例采用 `keepaspect` 与 `video-aspect-override` 硬件缩放，绝不插入打断硬解的 CPU vf 滤镜。
5. **单实例 IPC 与生命周期兜底**：
   - 基于本地回环端口与 `PULSE_PLAYER_IPC:v1:` 魔数握手协议的单实例通信，双击新文件唤起已有窗口切播；在端口异常时优雅降级独立运行；接入 Tauri `CloseRequested` 事件在 Alt+F4 及系统关闭时兜底保存断点历史。
6. **原生全屏双向严格同步**：
   - 接入 Tauri 窗口 `WindowEvent::Resized` 原生窗口全屏检测与 DOM `fullscreenchange`，双向同步全屏状态机，Esc 退出无延迟无脱节。
7. **路径归一化与播放结束重置**：
   - 统一采用反斜杠规范化与不区分大小写的 `is_same_path` 判定，保证选集高亮 100% 准确与历史记录可靠去重；单集播放完后历史记录自动归零重置。

