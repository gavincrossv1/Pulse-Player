// src-tauri/src/lib.rs
// Pulse Player 桌面应用 Rust 主入口

pub mod associations;
pub mod commands;
pub mod history;
pub mod mpv_player;
pub mod playlist;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use commands::*;
use history::save_history_record;
use playlist::SUPPORTED_EXTENSIONS;

const SINGLE_INSTANCE_PORT: u16 = 45389;
const SINGLE_INSTANCE_MAGIC: &str = "PULSE_PLAYER_IPC:v1:";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. 检测启动入参（双击视频文件关联启动或命令行入参）
    let mut initial_file: Option<String> = None;
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let candidate = &args[1];
        let p = std::path::Path::new(candidate);
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                    initial_file = Some(normalize_path(&p.to_string_lossy()));
                }
            }
        }
    }

    // 2. 单实例 IPC 互斥机制：双击新视频唤起已有实例切换播放，不重复拉起进程
    let single_instance_listener = match TcpListener::bind(("127.0.0.1", SINGLE_INSTANCE_PORT)) {
        Ok(listener) => Some(listener),
        Err(_) => {
            // 已有实例在运行，连接至首个实例并发送目标文件路径
            if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", SINGLE_INSTANCE_PORT)) {
                let msg = format!("{}{}", SINGLE_INSTANCE_MAGIC, initial_file.as_deref().unwrap_or(""));
                if stream.write_all(msg.as_bytes()).is_ok() {
                    let _ = stream.flush();
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                    std::process::exit(0);
                }
            }
            // 若端口冲突或连接未响应，优雅降级作为独立新实例启动，杜绝进程假死
            None
        }
    };

    let app_state = AppState {
        player: Mutex::new(None),
        launch_file: Mutex::new(initial_file),
    };

    tauri::Builder::default()
        .manage(app_state)
        .setup(move |app| {
            // 确保 Windows 应用程序能力模型及 SupportedTypes 基础注册表键就绪
            let _ = associations::register_base_capabilities();

            // 若为首个实例，启动后台单实例 IPC 监听线程
            if let Some(listener) = single_instance_listener {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    for stream in listener.incoming().flatten() {
                        let mut s = stream;
                        let mut buf = Vec::new();
                        let _ = s.read_to_end(&mut buf);
                        let text = String::from_utf8_lossy(&buf);

                        if let Some(path_raw) = text.strip_prefix(SINGLE_INSTANCE_MAGIC) {
                            let path_str = path_raw.trim().to_string();

                            // 唤起主窗口至最前并还原
                            if let Some(w) = app_handle.get_webview_window("main") {
                                let _ = w.unminimize();
                                let _ = w.show();
                                let _ = w.set_focus();
                            }

                            // 通知前端即刻切集播放
                            if !path_str.is_empty() {
                                let _ = app_handle.emit("open-file", path_str);
                            }
                        }
                    }
                });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // 接入 Tauri 窗口尺寸与全屏事件，向前端严格双向同步全屏状态
            if let tauri::WindowEvent::Resized(_) = event {
                if let Ok(fs) = window.is_fullscreen() {
                    let _ = window.emit("window:fullscreen", fs);
                }
            }
            // 接入 Tauri 窗口 CloseRequested 事件，确保 Alt+F4、任务栏关闭等均能持久化保存历史进度
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.try_state::<AppState>() {
                    if let Ok(guard) = state.player.lock() {
                        if let Some(ref player) = *guard {
                            if let Some(record) = player.get_current_record() {
                                save_history_record(record);
                            }
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            open_file_dialog,
            play_video,
            toggle_pause,
            set_pause,
            seek,
            set_speed,
            set_volume,
            set_aspect_ratio,
            set_rotation,
            set_mirror,
            set_hwdec,
            set_video_margin,
            scan_playlist,
            get_history,
            save_history,
            delete_history_item,
            clear_all_history,
            get_launch_file,
            set_pinned,
            start_dragging,
            minimize_window,
            toggle_maximize,
            set_fullscreen,
            is_fullscreen,
            close_window,
            get_file_associations,
            apply_file_associations,
            open_default_apps_settings,
            prompt_associate_extension,
            open_external_url,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Pulse Player 应用程序异常");
}
