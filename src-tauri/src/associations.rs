// src-tauri/src/associations.rs
// Windows 平台文件默认打开关联检测、应用能力模型注册与原生交互引导模块

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, KEY_READ, REG_SZ,
};
use windows_sys::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAssociationItem {
    pub ext: String,          // 扩展名（无前导点），如 "mp4"
    pub is_video: bool,       // true: 视频, false: 音频
    pub is_default: bool,     // 是否当前默认是 Pulse Player
    pub current_app: String,  // 当前默认打开的应用程序名
    pub is_registered: bool,  // 是否已在 Pulse Player 能力表 (Capabilities) 中登记关联
}

/// 支持检测与关联的视频扩展名列表 (共 10 种主流格式)
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "flv", "mov", "wmv", "webm", "ts", "m4v", "rmvb",
];

/// 支持检测与关联的音频扩展名列表 (共 9 种主流格式)
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "m4a", "aac", "ogg", "ape", "opus", "wma",
];

/// 将 Rust 字符串编码为 UTF-16 宽字符并添加 NULL 结尾
fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// 在指定根键下创建或打开子键（具备写入权限）
fn reg_create_key(root: HKEY, subkey: &str) -> Option<HKEY> {
    let subkey_w = to_wide_null(subkey);
    unsafe {
        let mut hkey: HKEY = std::mem::zeroed();
        let status = RegCreateKeyW(root, subkey_w.as_ptr(), &mut hkey);
        if status == ERROR_SUCCESS {
            Some(hkey)
        } else {
            None
        }
    }
}

/// 写入 REG_SZ 字符串值（若 name 为空字符串则写入键的默认值）
fn reg_set_string(hkey: HKEY, name: &str, value: &str) -> bool {
    let name_w = to_wide_null(name);
    let val_w = to_wide_null(value);
    unsafe {
        let name_ptr = if name.is_empty() {
            std::ptr::null()
        } else {
            name_w.as_ptr()
        };
        let status = RegSetValueExW(
            hkey,
            name_ptr,
            0,
            REG_SZ,
            val_w.as_ptr() as *const u8,
            (val_w.len() * std::mem::size_of::<u16>()) as u32,
        );
        status == ERROR_SUCCESS
    }
}

/// 删除指定键下的键值
fn reg_delete_value(hkey: HKEY, name: &str) {
    let name_w = to_wide_null(name);
    unsafe {
        RegDeleteValueW(hkey, name_w.as_ptr());
    }
}

/// 检查指定根键与子键下的键值是否存在
fn reg_value_exists(root: HKEY, subkey: &str, val_name: &str) -> bool {
    let subkey_w = to_wide_null(subkey);
    let val_w = to_wide_null(val_name);
    unsafe {
        let mut hkey: HKEY = std::mem::zeroed();
        let status = RegOpenKeyExW(root, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey);
        if status != ERROR_SUCCESS {
            return false;
        }
        let mut reg_type: u32 = 0;
        let query_status = RegQueryValueExW(
            hkey,
            val_w.as_ptr(),
            std::ptr::null_mut(),
            &mut reg_type,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        RegCloseKey(hkey);
        query_status == ERROR_SUCCESS
    }
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

/// 注册 Windows 规范应用程序能力模型（Capabilities、RegisteredApplications 及 SupportedTypes）
pub fn register_base_capabilities() -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "pulse-player.exe".to_string());
    let open_cmd = format!("\"{}\" \"%1\"", exe_path);
    let icon_val = format!("\"{}\",0", exe_path);

    let all_exts: Vec<&str> = VIDEO_EXTENSIONS.iter().chain(AUDIO_EXTENSIONS.iter()).copied().collect();

    // 1. 注册 HKCU\Software\Applications\pulse-player.exe
    if let Some(app_key) = reg_create_key(HKEY_CURRENT_USER, r"Software\Applications\pulse-player.exe") {
        reg_set_string(app_key, "FriendlyAppName", "Pulse Player");
        if let Some(icon_key) = reg_create_key(app_key, "DefaultIcon") {
            reg_set_string(icon_key, "", &icon_val);
            unsafe { RegCloseKey(icon_key); }
        }
        if let Some(cmd_key) = reg_create_key(app_key, r"shell\open\command") {
            reg_set_string(cmd_key, "", &open_cmd);
            unsafe { RegCloseKey(cmd_key); }
        }
        if let Some(supp_key) = reg_create_key(app_key, "SupportedTypes") {
            for &ext in &all_exts {
                reg_set_string(supp_key, &format!(".{}", ext), "");
            }
            unsafe { RegCloseKey(supp_key); }
        }
        unsafe { RegCloseKey(app_key); }
    }

    // 2. 注册 HKCU\Software\Classes\Applications\pulse-player.exe (镜像映射至 HKCR\Applications\pulse-player.exe)
    if let Some(classes_app_key) = reg_create_key(HKEY_CURRENT_USER, r"Software\Classes\Applications\pulse-player.exe") {
        reg_set_string(classes_app_key, "FriendlyAppName", "Pulse Player");
        if let Some(icon_key) = reg_create_key(classes_app_key, "DefaultIcon") {
            reg_set_string(icon_key, "", &icon_val);
            unsafe { RegCloseKey(icon_key); }
        }
        if let Some(cmd_key) = reg_create_key(classes_app_key, r"shell\open\command") {
            reg_set_string(cmd_key, "", &open_cmd);
            unsafe { RegCloseKey(cmd_key); }
        }
        if let Some(supp_key) = reg_create_key(classes_app_key, "SupportedTypes") {
            for &ext in &all_exts {
                reg_set_string(supp_key, &format!(".{}", ext), "");
            }
            unsafe { RegCloseKey(supp_key); }
        }
        unsafe { RegCloseKey(classes_app_key); }
    }

    // 3. 尝试直接向 HKEY_CLASSES_ROOT\Applications\pulse-player.exe\SupportedTypes 写入 (若权限允许)
    if let Some(hkcr_supp_key) = reg_create_key(HKEY_CLASSES_ROOT, r"Applications\pulse-player.exe\SupportedTypes") {
        for &ext in &all_exts {
            reg_set_string(hkcr_supp_key, &format!(".{}", ext), "");
        }
        unsafe { RegCloseKey(hkcr_supp_key); }
    }

    // 4. 注册 Windows 规范 Capabilities（使 Windows 11 设置中可正确检索到 Pulse Player）
    if let Some(cap_key) = reg_create_key(HKEY_CURRENT_USER, r"Software\PulsePlayer\Capabilities") {
        reg_set_string(cap_key, "ApplicationName", "Pulse Player");
        reg_set_string(cap_key, "ApplicationDescription", "Pulse Player 本地极简离线媒体播放器");
        reg_set_string(cap_key, "ApplicationIcon", &icon_val);
        unsafe { RegCloseKey(cap_key); }
    }

    // 5. 登记至 HKCU\Software\RegisteredApplications
    if let Some(reg_apps_key) = reg_create_key(HKEY_CURRENT_USER, r"Software\RegisteredApplications") {
        reg_set_string(reg_apps_key, "PulsePlayer", r"Software\PulsePlayer\Capabilities");
        unsafe { RegCloseKey(reg_apps_key); }
    }

    Ok(())
}

/// 批量应用用户所勾选的文件格式关联，写入能力注册表并广播系统 Shell 刷新通知
pub fn apply_file_associations(selected_exts: &[String]) -> Result<(), String> {
    // 1. 确保基础能力模型与 SupportedTypes 注册表键已登记
    register_base_capabilities()?;

    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "pulse-player.exe".to_string());
    let open_cmd = format!("\"{}\" \"%1\"", exe_path);
    let icon_val = format!("\"{}\",0", exe_path);

    let all_exts: Vec<&str> = VIDEO_EXTENSIONS.iter().chain(AUDIO_EXTENSIONS.iter()).copied().collect();

    // 2. 打开 Capabilities\FileAssociations 键
    let cap_file_assoc_key = reg_create_key(
        HKEY_CURRENT_USER,
        r"Software\PulsePlayer\Capabilities\FileAssociations",
    ).ok_or_else(|| "无法创建或打开 Capabilities\\FileAssociations 注册表项".to_string())?;

    for &ext in &all_exts {
        let dot_ext = format!(".{}", ext);
        let prog_id = format!("PulsePlayer.AssocFile.{}", ext);
        let is_selected = selected_exts.iter().any(|s| s.eq_ignore_ascii_case(ext));

        if is_selected {
            // 登记至 Capabilities\FileAssociations: .ext -> PulsePlayer.AssocFile.{ext}
            reg_set_string(cap_file_assoc_key, &dot_ext, &prog_id);

            // 注册 ProgId: HKCU\Software\Classes\PulsePlayer.AssocFile.{ext}
            let prog_subkey = format!(r"Software\Classes\{}", prog_id);
            if let Some(prog_key) = reg_create_key(HKEY_CURRENT_USER, &prog_subkey) {
                let type_desc = format!("Pulse Player {} 媒体文件", ext.to_uppercase());
                reg_set_string(prog_key, "", &type_desc);
                reg_set_string(prog_key, "FriendlyTypeName", &type_desc);

                if let Some(icon_key) = reg_create_key(prog_key, "DefaultIcon") {
                    reg_set_string(icon_key, "", &icon_val);
                    unsafe { RegCloseKey(icon_key); }
                }
                if let Some(cmd_key) = reg_create_key(prog_key, r"shell\open\command") {
                    reg_set_string(cmd_key, "", &open_cmd);
                    unsafe { RegCloseKey(cmd_key); }
                }
                unsafe { RegCloseKey(prog_key); }
            }

            // 在 HKCU\Software\Classes\.{ext}\OpenWithProgids 中登记
            let classes_openwith = format!(r"Software\Classes\.{}\OpenWithProgids", ext);
            if let Some(ow_key) = reg_create_key(HKEY_CURRENT_USER, &classes_openwith) {
                reg_set_string(ow_key, &prog_id, "");
                unsafe { RegCloseKey(ow_key); }
            }

            // 在 Explorer\FileExts\.{ext}\OpenWithProgids 中登记
            let explorer_openwith = format!(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{}\OpenWithProgids",
                ext
            );
            if let Some(exp_ow_key) = reg_create_key(HKEY_CURRENT_USER, &explorer_openwith) {
                reg_set_string(exp_ow_key, &prog_id, "");
                unsafe { RegCloseKey(exp_ow_key); }
            }

            // 在 Explorer\FileExts\.{ext}\OpenWithList 中确保登记 pulse-player.exe
            let explorer_list = format!(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{}\OpenWithList",
                ext
            );
            if let Some(exp_list_key) = reg_create_key(HKEY_CURRENT_USER, &explorer_list) {
                reg_set_string(exp_list_key, "pulse-player.exe", "");
                unsafe { RegCloseKey(exp_list_key); }
            }
        } else {
            // 用户未勾选该格式，清除 Capabilities 与 OpenWithProgids 登记
            reg_delete_value(cap_file_assoc_key, &dot_ext);

            let classes_openwith = format!(r"Software\Classes\.{}\OpenWithProgids", ext);
            if let Some(ow_key) = reg_create_key(HKEY_CURRENT_USER, &classes_openwith) {
                reg_delete_value(ow_key, &prog_id);
                unsafe { RegCloseKey(ow_key); }
            }

            let explorer_openwith = format!(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{}\OpenWithProgids",
                ext
            );
            if let Some(exp_ow_key) = reg_create_key(HKEY_CURRENT_USER, &explorer_openwith) {
                reg_delete_value(exp_ow_key, &prog_id);
                unsafe { RegCloseKey(exp_ow_key); }
            }
        }
    }

    unsafe {
        RegCloseKey(cap_file_assoc_key);
    }

    // 3. 广播 Windows Shell 关联变更通知 (SHCNE_ASSOCCHANGED)，令外壳立即刷新图标与右键菜单
    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        );
    }

    Ok(())
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
        let is_registered = reg_value_exists(
            HKEY_CURRENT_USER,
            r"Software\PulsePlayer\Capabilities\FileAssociations",
            &format!(".{}", ext),
        );
        results.push(FileAssociationItem {
            ext: ext.to_string(),
            is_video: true,
            is_default,
            current_app,
            is_registered,
        });
    }

    // 2. 扫描音频类
    for &ext in AUDIO_EXTENSIONS {
        let (is_default, current_app) = match query_user_choice_prog_id(ext) {
            Some(prog_id) => format_friendly_app_name(&prog_id),
            None => (false, "未指定默认应用".to_string()),
        };
        let is_registered = reg_value_exists(
            HKEY_CURRENT_USER,
            r"Software\PulsePlayer\Capabilities\FileAssociations",
            &format!(".{}", ext),
        );
        results.push(FileAssociationItem {
            ext: ext.to_string(),
            is_video: false,
            is_default,
            current_app,
            is_registered,
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

    #[test]
    fn test_apply_and_query_associations() {
        // 测试将 mp4 与 mp3 登记关联
        let test_exts = vec!["mp4".to_string(), "mp3".to_string()];
        let res = apply_file_associations(&test_exts);
        assert!(res.is_ok(), "apply_file_associations 应该执行成功: {:?}", res);

        let list = get_all_associations();
        let mp4_item = list.iter().find(|i| i.ext == "mp4").expect("应包含 mp4");
        assert!(mp4_item.is_registered, "mp4 应该已被登记关联");

        let mp3_item = list.iter().find(|i| i.ext == "mp3").expect("应包含 mp3");
        assert!(mp3_item.is_registered, "mp3 应该已被登记关联");

        let rmvb_item = list.iter().find(|i| i.ext == "rmvb").expect("应包含 rmvb");
        assert!(!rmvb_item.is_registered, "未选中的 rmvb 应该未登记关联");

        // 验证基础注册表键存在
        assert!(reg_value_exists(
            HKEY_CURRENT_USER,
            r"Software\Applications\pulse-player.exe\SupportedTypes",
            ".mp4"
        ));
        assert!(reg_value_exists(
            HKEY_CURRENT_USER,
            r"Software\PulsePlayer\Capabilities",
            "ApplicationName"
        ));
        assert!(reg_value_exists(
            HKEY_CURRENT_USER,
            r"Software\RegisteredApplications",
            "PulsePlayer"
        ));
    }
}
