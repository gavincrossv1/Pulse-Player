// src-tauri/src/commands.rs
// Tauri IPC 命令路由中心

use std::sync::Mutex;
use tauri::{AppHandle, State, Window};
use crate::associations::{get_all_associations, open_default_apps_settings as do_open_default_apps, prompt_associate_extension as do_prompt_associate, FileAssociationItem};
use crate::history::{clear_history, delete_history_record, load_history, save_history_record, HistoryRecord};
use crate::mpv_player::MpvPlayer;
use crate::playlist::{scan_folder_videos, PlaylistItem, SUPPORTED_EXTENSIONS};

pub struct AppState {
    pub player: Mutex<Option<MpvPlayer>>,
    pub launch_file: Mutex<Option<String>>,
}

/// 弹出系统原生文件选择对话框
#[tauri::command]
pub async fn open_file_dialog() -> Result<Option<String>, String> {
    let dialog = rfd::AsyncFileDialog::new()
        .set_title("选择要播放的本地媒体文件")
        .add_filter("媒体文件 (视频/音频)", SUPPORTED_EXTENSIONS);

    if let Some(file) = dialog.pick_file().await {
        Ok(Some(file.path().to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

/// 规范化 Windows 文件路径 (统一反斜杠、去除两端空格，并在文件存在时解析真实绝对路径)
pub fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let unified = trimmed.replace('/', "\\");
    if let Ok(abs) = std::fs::canonicalize(&unified) {
        let s = abs.to_string_lossy().to_string();
        if s.starts_with(r"\\?\UNC\") {
            format!(r"\\{}", &s[8..])
        } else if s.starts_with(r"\\?\") {
            s[4..].to_string()
        } else {
            s
        }
    } else {
        unified
    }
}

/// 统一比对两个路径是否指向同一文件 (无视正反斜杠差异与 Windows 大小写差异)
pub fn is_same_path(p1: &str, p2: &str) -> bool {
    let n1 = normalize_path(p1).replace('/', "\\").to_lowercase();
    let n2 = normalize_path(p2).replace('/', "\\").to_lowercase();
    !n1.is_empty() && n1 == n2
}

/// 启动播放指定视频
#[tauri::command]
pub fn play_video(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    window: Window,
    path: String,
    start_time: Option<f64>,
) -> Result<(), String> {
    let clean_path = normalize_path(&path);
    let mut guard = state.player.lock().unwrap();
    if guard.is_none() {
        // 获取窗口句柄 HWND，以备将 mpv 视口绑定
        let hwnd = window.hwnd().ok().map(|h| h.0 as isize);
        let player = MpvPlayer::new(app_handle, hwnd)?;
        *guard = Some(player);
    }

    if let Some(ref player) = *guard {
        player.load_file(&clean_path, start_time)?;
    }
    Ok(())
}

/// 切换 播放 / 暂停
#[tauri::command]
pub fn toggle_pause(state: State<'_, AppState>) -> Result<bool, String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.toggle_pause()
    } else {
        Ok(false)
    }
}

/// 明确设置暂停状态
#[tauri::command]
pub fn set_pause(state: State<'_, AppState>, paused: bool) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.set_pause(paused)
    } else {
        Ok(())
    }
}

/// 执行跳转 Seek
#[tauri::command]
pub fn seek(state: State<'_, AppState>, seconds: f64, relative: bool) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.seek(seconds, relative)
    } else {
        Ok(())
    }
}

/// 设置播放倍速
#[tauri::command]
pub fn set_speed(state: State<'_, AppState>, speed: f64) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.set_speed(speed)
    } else {
        Ok(())
    }
}

/// 设置独立软件音量
#[tauri::command]
pub fn set_volume(state: State<'_, AppState>, volume: f64) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.set_volume(volume)
    } else {
        Ok(())
    }
}

/// 设置视频比例
#[tauri::command]
pub fn set_aspect_ratio(state: State<'_, AppState>, ratio: String) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.set_aspect_ratio(&ratio)
    } else {
        Ok(())
    }
}

/// 设置画面旋转角度
#[tauri::command]
pub fn set_rotation(state: State<'_, AppState>, degree: i32) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.set_rotation(degree)
    } else {
        Ok(())
    }
}

/// 设置画面水平镜像
#[tauri::command]
pub fn set_mirror(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.set_mirror(enabled)
    } else {
        Ok(())
    }
}

/// 切换解码模式
#[tauri::command]
pub fn set_hwdec(state: State<'_, AppState>, mode: String) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.set_hwdec(&mode)
    } else {
        Ok(())
    }
}

/// 动态设置视频顶部安全边距比例 (窗口模式留出 TitleBar 边距，全屏模式设为 0)
#[tauri::command]
pub fn set_video_margin(state: State<'_, AppState>, ratio: f64) -> Result<(), String> {
    let guard = state.player.lock().unwrap();
    if let Some(ref player) = *guard {
        player.set_top_margin_ratio(ratio)
    } else {
        Ok(())
    }
}

/// 扫描指定文件所在文件夹并生成自然排序的播放列表
#[tauri::command]
pub fn scan_playlist(path: String) -> Result<Vec<PlaylistItem>, String> {
    let clean_path = normalize_path(&path);
    Ok(scan_folder_videos(&clean_path))
}

/// 获取最近播放历史
#[tauri::command]
pub fn get_history() -> Result<Vec<HistoryRecord>, String> {
    Ok(load_history())
}

/// 保存断点与播放历史
#[tauri::command]
pub fn save_history(record: HistoryRecord) -> Result<(), String> {
    save_history_record(record);
    Ok(())
}

/// 删除指定单条播放历史记录
#[tauri::command]
pub fn delete_history_item(path: String) -> Result<(), String> {
    delete_history_record(&path);
    Ok(())
}

/// 清空全部播放历史记录
#[tauri::command]
pub fn clear_all_history() -> Result<(), String> {
    clear_history();
    Ok(())
}

/// 获取随命令行传入的视频路径（双击文件关联打开直达播放态）
#[tauri::command]
pub fn get_launch_file(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let guard = state.launch_file.lock().unwrap();
    Ok(guard.clone())
}

/// 窗口控制：拖拽移动窗口
#[tauri::command]
pub fn start_dragging(window: Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

/// 窗口控制：置顶切换
#[tauri::command]
pub fn set_pinned(window: Window, pinned: bool) -> Result<(), String> {
    window.set_always_on_top(pinned).map_err(|e| e.to_string())
}

/// 窗口控制：最小化
#[tauri::command]
pub fn minimize_window(window: Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

/// 窗口控制：最大化 / 还原
#[tauri::command]
pub fn toggle_maximize(window: Window) -> Result<bool, String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

/// 窗口控制：原生窗口级全屏切换
#[tauri::command]
pub fn set_fullscreen(window: Window, fullscreen: bool) -> Result<(), String> {
    window.set_fullscreen(fullscreen).map_err(|e| e.to_string())
}

/// 窗口控制：查询是否全屏
#[tauri::command]
pub fn is_fullscreen(window: Window) -> Result<bool, String> {
    window.is_fullscreen().map_err(|e| e.to_string())
}

/// 窗口控制：关闭
#[tauri::command]
pub fn close_window(window: Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

/// 获取全量媒体文件在当前系统的默认打开程序状态
#[tauri::command]
pub fn get_file_associations() -> Result<Vec<FileAssociationItem>, String> {
    Ok(get_all_associations())
}

/// 唤起 Windows 系统默认应用设置面板
#[tauri::command]
pub fn open_default_apps_settings() -> Result<(), String> {
    do_open_default_apps()
}

/// 针对单项特定文件后缀唤起 Windows 原生打开方式选择对话框
#[tauri::command]
pub fn prompt_associate_extension(ext: String) -> Result<(), String> {
    do_prompt_associate(&ext)
}

/// 打开指定的外部链接 (如 GitHub Release 页面)
#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", "", &url])
        .spawn()
        .map_err(|e| format!("打开链接失败: {}", e))?;
    Ok(())
}
