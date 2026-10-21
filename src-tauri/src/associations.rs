// src-tauri/src/associations.rs
// Windows 平台文件默认打开关联检测与原生交互引导模块

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_SZ,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAssociationItem {
    pub ext: String,          // 扩展名（无前导点），如 "mp4"
    pub is_video: bool,       // true: 视频, false: 音频
    pub is_default: bool,     // 是否当前默认是 Pulse Player
    pub current_app: String,  // 当前默认打开的应用程序名
}

/// 支持检测的视频扩展名列表
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "flv", "mov", "wmv", "webm", "ts", "m4v", "rmvb",
];

/// 支持检测的音频扩展名列表
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "m4a", "aac", "ogg", "ape", "opus", "wma",
];

/// 将 Rust 字符串编码为 UTF-16 宽字符并添加 NULL 结尾
fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// 从注册表查询当前扩展名在 UserChoice 下设置的 ProgId
fn query_user_choice_prog_id(ext: &str) -> Option<String> {
    let subkey = format!(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{}\UserChoice",
        ext
    );
    let subkey_w = to_wide_null(&subkey);
    let val_name_w = to_wide_null("ProgId");

    unsafe {
        let mut hkey: HKEY = std::mem::zeroed();
        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey_w.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        );
        if status != ERROR_SUCCESS {
            return None;
        }

        let mut buf = [0u16; 512];
        let mut buf_size: u32 = (buf.len() * std::mem::size_of::<u16>()) as u32;
        let mut reg_type: u32 = 0;

        let query_status = RegQueryValueExW(
            hkey,
            val_name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut reg_type,
            buf.as_mut_ptr() as *mut _,
            &mut buf_size,
        );
        RegCloseKey(hkey);

        if query_status == ERROR_SUCCESS && reg_type == REG_SZ {
            let len = (buf_size as usize) / std::mem::size_of::<u16>();
            // 过滤结尾的 \0
            let slice = if len > 0 && buf[len - 1] == 0 {
                &buf[..len - 1]
            } else {
                &buf[..len]
            };
            return Some(String::from_utf16_lossy(slice));
        }
    }
    None
}

/// 将 ProgId 转换为人类可读的友好应用名称
fn format_friendly_app_name(prog_id: &str) -> (bool, String) {
    let lower = prog_id.to_lowercase();
    if lower.contains("pulse-player") || lower.contains("pulseplayer") || lower.contains("pulse player") {
        return (true, "Pulse Player".to_string());
    }

    if lower.contains("potplayer") {
        return (false, "PotPlayer".to_string());
    }
    if lower.contains("vlc") {
        return (false, "VLC Media Player".to_string());
    }
    if lower.contains("microsoft.zunevideo") || lower.contains("appx") && lower.contains("video") {
        return (false, "电影和电视 (Windows)".to_string());
    }
    if lower.contains("microsoft.zunemusic") || lower.contains("appx") && lower.contains("music") {
        return (false, "Windows Media Player".to_string());
    }
    if lower.contains("wmp11") || lower.contains("wmplayer") {
        return (false, "Windows Media Player (旧版)".to_string());
    }
    if lower.contains("quark") {
        return (false, "夸克播放器".to_string());
    }
    if lower.contains("chrome") {
        return (false, "Google Chrome".to_string());
    }
    if lower.contains("msedge") || lower.contains("edge") {
        return (false, "Microsoft Edge".to_string());
    }

    // 默认清理 Applications\ 前缀
    let clean = if let Some(stripped) = prog_id.strip_prefix(r"Applications\") {
        stripped
    } else {
        prog_id
    };
    (false, clean.to_string())
}

/// 获取全量支持媒体格式在当前系统的默认打开方式状态
pub fn get_all_associations() -> Vec<FileAssociationItem> {
    let mut results = Vec::new();

    // 1. 扫描视频类
    for &ext in VIDEO_EXTENSIONS {
        let (is_default, current_app) = match query_user_choice_prog_id(ext) {
            Some(prog_id) => format_friendly_app_name(&prog_id),
            None => (false, "未指定默认应用".to_string()),
        };
        results.push(FileAssociationItem {
            ext: ext.to_string(),
            is_video: true,
            is_default,
            current_app,
        });
    }

    // 2. 扫描音频类
    for &ext in AUDIO_EXTENSIONS {
        let (is_default, current_app) = match query_user_choice_prog_id(ext) {
            Some(prog_id) => format_friendly_app_name(&prog_id),
            None => (false, "未指定默认应用".to_string()),
        };
        results.push(FileAssociationItem {
            ext: ext.to_string(),
            is_video: false,
            is_default,
            current_app,
        });
    }

    results
}

/// 打开 Windows 原生默认应用设置面板 (ms-settings:defaultapps)
pub fn open_default_apps_settings() -> Result<(), String> {
    Command::new("cmd")
        .args(["/c", "start", "ms-settings:defaultapps"])
        .spawn()
        .map_err(|e| format!("打开系统默认应用设置失败: {}", e))?;
    Ok(())
}

/// 针对单项特定文件后缀唤起 Windows 原生“打开方式”选择弹窗
pub fn prompt_associate_extension(ext: &str) -> Result<(), String> {
    let clean_ext = ext.trim_start_matches('.').to_lowercase();
    let temp_dir = std::env::temp_dir().join("Pulse-Player");
    let _ = std::fs::create_dir_all(&temp_dir);
    let sample_file = temp_dir.join(format!("associate_sample.{}", clean_ext));

    // 创建或更新该扩展名的临时占位空文件
    if !sample_file.exists() {
        let _ = std::fs::write(&sample_file, b"");
    }

    // 调用 Windows 原生 shell32 OpenAs 对话框，由用户合规勾选“始终”
    Command::new("rundll32.exe")
        .args([
            "shell32.dll,OpenAs_RunDLL",
            &sample_file.to_string_lossy(),
        ])
        .spawn()
        .map_err(|e| format!("唤起 Windows 打开方式对话框失败: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_associations() {
        let list = get_all_associations();
        assert!(!list.is_empty());
        assert!(list.iter().any(|item| item.ext == "mp4" && item.is_video));
        assert!(list.iter().any(|item| item.ext == "mp3" && !item.is_video));
        for item in &list {
            assert!(!item.current_app.is_empty(), "每个扩展名都应返回当前关联应用描述");
        }
    }
}
