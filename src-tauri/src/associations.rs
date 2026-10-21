// src-tauri/src/associations.rs
// Windows 平台文件默认打开关联检测、应用能力模型注册与原生交互引导模块

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, KEY_READ, REG_SZ,
};
use windows_sys::Win32::UI::Shell::{
    ShellExecuteW, SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

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
/// 从注册表查询当前扩展名在 UserChoice 或 Classes 下设置的 ProgId
fn query_default_prog_id(ext: &str) -> Option<String> {
    // 1. 优先读取 Windows 10/11 Explorer UserChoice
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
        if status == ERROR_SUCCESS {
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
                let slice = if len > 0 && buf[len - 1] == 0 {
                    &buf[..len - 1]
                } else {
                    &buf[..len]
                };
                let id = String::from_utf16_lossy(slice).trim().to_string();
                if !id.is_empty() {
                    return Some(id);
                }
            }
        }
    }

    // 2. 若未显式设置 UserChoice，回退查询 HKCU\Software\Classes\.{ext} (Default)
    let classes_subkey = format!(r"Software\Classes\.{}", ext);
    let classes_subkey_w = to_wide_null(&classes_subkey);
    unsafe {
        let mut hkey: HKEY = std::mem::zeroed();
        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            classes_subkey_w.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        );
        if status == ERROR_SUCCESS {
            let mut buf = [0u16; 512];
            let mut buf_size: u32 = (buf.len() * std::mem::size_of::<u16>()) as u32;
            let mut reg_type: u32 = 0;

            let query_status = RegQueryValueExW(
                hkey,
                std::ptr::null(), // default value
                std::ptr::null_mut(),
                &mut reg_type,
                buf.as_mut_ptr() as *mut _,
                &mut buf_size,
            );
            RegCloseKey(hkey);

            if query_status == ERROR_SUCCESS && reg_type == REG_SZ {
                let len = (buf_size as usize) / std::mem::size_of::<u16>();
                let slice = if len > 0 && buf[len - 1] == 0 {
                    &buf[..len - 1]
                } else {
                    &buf[..len]
                };
                let id = String::from_utf16_lossy(slice).trim().to_string();
                if !id.is_empty() {
                    return Some(id);
                }
            }
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

    if lower.contains("musicplayer2") {
        return (false, "MusicPlayer2".to_string());
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

/// 注册 Windows 规范应用程序能力模型（Capabilities、RegisteredApplications、SupportedTypes 及 ProgIDs）
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

    // 2. 注册 HKCU\Software\Classes\Applications\pulse-player.exe (自动映射至 HKCR\Applications\pulse-player.exe)
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

    // 3. 尝试向 HKEY_CLASSES_ROOT\Applications\pulse-player.exe\SupportedTypes 写入 (若权限允许)
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

    // 6. 注册 Capabilities\FileAssociations 完整映射表（声明支持的所有 19 种主流音视频格式）
    if let Some(cap_assoc_key) = reg_create_key(HKEY_CURRENT_USER, r"Software\PulsePlayer\Capabilities\FileAssociations") {
        for &ext in &all_exts {
            let dot_ext = format!(".{}", ext);
            let prog_id = format!("PulsePlayer.AssocFile.{}", ext);
            reg_set_string(cap_assoc_key, &dot_ext, &prog_id);
        }
        unsafe { RegCloseKey(cap_assoc_key); }
    }

    // 7. 注册全部 19 种 ProgID (HKCU\Software\Classes\PulsePlayer.AssocFile.{ext})
    for &ext in &all_exts {
        let prog_id = format!("PulsePlayer.AssocFile.{}", ext);
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
    }

    Ok(())
}

/// 批量应用用户所勾选的文件格式关联，写入关联注册表并广播系统 Shell 刷新通知
pub fn apply_file_associations(selected_exts: &[String]) -> Result<(), String> {
    // 1. 确保基础能力模型、SupportedTypes 及 ProgIDs 均已完整登记
    register_base_capabilities()?;

    let all_exts: Vec<&str> = VIDEO_EXTENSIONS.iter().chain(AUDIO_EXTENSIONS.iter()).copied().collect();

    for &ext in &all_exts {
        let prog_id = format!("PulsePlayer.AssocFile.{}", ext);
        let is_selected = selected_exts.iter().any(|s| s.eq_ignore_ascii_case(ext));

        if is_selected {
            // 写入 HKCU\Software\Classes\.{ext}\OpenWithProgids
            let classes_openwith = format!(r"Software\Classes\.{}\OpenWithProgids", ext);
            if let Some(ow_key) = reg_create_key(HKEY_CURRENT_USER, &classes_openwith) {
                reg_set_string(ow_key, &prog_id, "");
                unsafe { RegCloseKey(ow_key); }
            }

            // 写入 Explorer\FileExts\.{ext}\OpenWithProgids
            let explorer_openwith = format!(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{}\OpenWithProgids",
                ext
            );
            if let Some(exp_ow_key) = reg_create_key(HKEY_CURRENT_USER, &explorer_openwith) {
                reg_set_string(exp_ow_key, &prog_id, "");
                unsafe { RegCloseKey(exp_ow_key); }
            }

            // 写入 Explorer\FileExts\.{ext}\OpenWithList
            let explorer_list = format!(
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{}\OpenWithList",
                ext
            );
            if let Some(exp_list_key) = reg_create_key(HKEY_CURRENT_USER, &explorer_list) {
                reg_set_string(exp_list_key, "pulse-player.exe", "");
                unsafe { RegCloseKey(exp_list_key); }
            }

            // 写入 HKCU\Software\Classes\.{ext} 默认关联键
            let classes_ext_key = format!(r"Software\Classes\.{}", ext);
            if let Some(ext_key) = reg_create_key(HKEY_CURRENT_USER, &classes_ext_key) {
                reg_set_string(ext_key, "", &prog_id);
                unsafe { RegCloseKey(ext_key); }
            }
        } else {
            // 用户未勾选该格式，清除 OpenWithProgids 登记
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

            // 如果 HKCU\Software\Classes\.{ext} 默认值为 PulsePlayer，则清理
            let classes_ext_key = format!(r"Software\Classes\.{}", ext);
            if let Some(ext_key) = reg_create_key(HKEY_CURRENT_USER, &classes_ext_key) {
                if let Some(curr_default) = query_default_prog_id(ext) {
                    if curr_default.contains("PulsePlayer") || curr_default.contains("pulse-player") {
                        reg_delete_value(ext_key, "");
                    }
                }
                unsafe { RegCloseKey(ext_key); }
            }
        }
    }

    // 记录用户已主动配置过关联的标记，防止后续清空选项时被误判为首次启动
    if let Some(app_meta_key) = reg_create_key(HKEY_CURRENT_USER, r"Software\PulsePlayer") {
        reg_set_string(app_meta_key, "AssociationsConfigured", "1");
        unsafe { RegCloseKey(app_meta_key); }
    }

    // 2. 广播 Windows Shell 关联变更通知 (SHCNE_ASSOCCHANGED)，令外壳立即刷新图标与右键菜单
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
        let (is_default, current_app) = match query_default_prog_id(ext) {
            Some(prog_id) => format_friendly_app_name(&prog_id),
            None => (false, "未指定默认应用".to_string()),
        };
        let is_ow_registered = reg_value_exists(
            HKEY_CURRENT_USER,
            &format!(r"Software\Classes\.{}\OpenWithProgids", ext),
            &format!("PulsePlayer.AssocFile.{}", ext),
        );
        let is_registered = is_default || is_ow_registered;

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
        let (is_default, current_app) = match query_default_prog_id(ext) {
            Some(prog_id) => format_friendly_app_name(&prog_id),
            None => (false, "未指定默认应用".to_string()),
        };
        let is_ow_registered = reg_value_exists(
            HKEY_CURRENT_USER,
            &format!(r"Software\Classes\.{}\OpenWithProgids", ext),
            &format!("PulsePlayer.AssocFile.{}", ext),
        );
        let is_registered = is_default || is_ow_registered;

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

/// 打开 Windows 原生默认应用设置面板 (直接定位至 Pulse Player 能力配置页)
pub fn open_default_apps_settings() -> Result<(), String> {
    let direct_url_w = to_wide_null("ms-settings:defaultapps?registeredAppUser=PulsePlayer");
    let op_w = to_wide_null("open");

    let res = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            op_w.as_ptr(),
            direct_url_w.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    // ShellExecute 返回值大于 32 表示成功启动外部进程或协议
    if (res as usize) <= 32 {
        let fallback_url_w = to_wide_null("ms-settings:defaultapps");
        let fallback_res = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                op_w.as_ptr(),
                fallback_url_w.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };
        if (fallback_res as usize) <= 32 {
            return Err("打开系统默认应用设置失败".to_string());
        }
    }
    Ok(())
}

/// 针对单项特定文件后缀唤起 Windows 原生设置引导用户设为默认播放器
pub fn prompt_associate_extension(ext: &str) -> Result<(), String> {
    let clean_ext = ext.trim_start_matches('.').to_lowercase();

    // 1. 清理以往遗留的历史临时探测占位文件 (若存在)
    let temp_dir = std::env::temp_dir().join("Pulse-Player");
    let sample_file = temp_dir.join(format!("associate_sample.{}", clean_ext));
    if sample_file.exists() {
        let _ = std::fs::remove_file(&sample_file);
    }

    // 2. 确保 Pulse Player 应用程序能力模型及该扩展名的 ProgID 与 OpenWith 注册表就绪
    register_base_capabilities()?;

    let prog_id = format!("PulsePlayer.AssocFile.{}", clean_ext);

    // 写入 HKCU\Software\Classes\.{clean_ext}\OpenWithProgids
    let classes_openwith = format!(r"Software\Classes\.{}\OpenWithProgids", clean_ext);
    if let Some(ow_key) = reg_create_key(HKEY_CURRENT_USER, &classes_openwith) {
        reg_set_string(ow_key, &prog_id, "");
        unsafe { RegCloseKey(ow_key); }
    }

    // 写入 Explorer\FileExts\.{clean_ext}\OpenWithProgids
    let explorer_openwith = format!(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{}\OpenWithProgids",
        clean_ext
    );
    if let Some(exp_ow_key) = reg_create_key(HKEY_CURRENT_USER, &explorer_openwith) {
        reg_set_string(exp_ow_key, &prog_id, "");
        unsafe { RegCloseKey(exp_ow_key); }
    }

    // 写入 Explorer\FileExts\.{clean_ext}\OpenWithList
    let explorer_list = format!(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.{}\OpenWithList",
        clean_ext
    );
    if let Some(exp_list_key) = reg_create_key(HKEY_CURRENT_USER, &explorer_list) {
        reg_set_string(exp_list_key, "pulse-player.exe", "");
        unsafe { RegCloseKey(exp_list_key); }
    }

    // 写入 HKCU\Software\Classes\.{clean_ext} 默认关联键
    let classes_ext_key = format!(r"Software\Classes\.{}", clean_ext);
    if let Some(ext_key) = reg_create_key(HKEY_CURRENT_USER, &classes_ext_key) {
        reg_set_string(ext_key, "", &prog_id);
        unsafe { RegCloseKey(ext_key); }
    }

    // 记录用户已主动配置过关联的标记
    if let Some(app_meta_key) = reg_create_key(HKEY_CURRENT_USER, r"Software\PulsePlayer") {
        reg_set_string(app_meta_key, "AssociationsConfigured", "1");
        unsafe { RegCloseKey(app_meta_key); }
    }

    // 广播 Windows Shell 关联变更通知 (SHCNE_ASSOCCHANGED)
    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        );
    }

    // 3. 原生唤起 Windows 默认应用深度设置面板，直达 Pulse Player 格式接管视图
    open_default_apps_settings()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static REG_TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_get_all_associations() {
        let _lock = REG_TEST_MUTEX.lock().unwrap();
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
        let _lock = REG_TEST_MUTEX.lock().unwrap();
        // 1. 测试将 mp4 与 mp3 登记关联
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

        // 验证基础注册表键与能力表完整存在
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
        // Capabilities\FileAssociations 必须包含所有支持格式（声明完整播放能力）
        assert!(reg_value_exists(
            HKEY_CURRENT_USER,
            r"Software\PulsePlayer\Capabilities\FileAssociations",
            ".mp4"
        ));
        assert!(reg_value_exists(
            HKEY_CURRENT_USER,
            r"Software\PulsePlayer\Capabilities\FileAssociations",
            ".rmvb"
        ));

        // 2. 测试取消选择 mp4（仅保留 mp3）
        let test_exts_2 = vec!["mp3".to_string()];
        let res_2 = apply_file_associations(&test_exts_2);
        assert!(res_2.is_ok());

        let list_2 = get_all_associations();
        let mp4_item_2 = list_2.iter().find(|i| i.ext == "mp4").expect("应包含 mp4");
        assert!(!mp4_item_2.is_registered || mp4_item_2.is_default, "取消选择后若非系统默认则不应再登记关联");

        let mp3_item_2 = list_2.iter().find(|i| i.ext == "mp3").expect("应包含 mp3");
        assert!(mp3_item_2.is_registered, "mp3 仍应处于登记关联状态");
    }

    #[test]
    fn test_prompt_associate_extension_registers_progid_and_cleans_sample() {
        let _lock = REG_TEST_MUTEX.lock().unwrap();
        // 创建一个虚假遗留的 sample 占位文件
        let temp_dir = std::env::temp_dir().join("Pulse-Player");
        let _ = std::fs::create_dir_all(&temp_dir);
        let sample_file = temp_dir.join("associate_sample.wav");
        let _ = std::fs::write(&sample_file, b"test dummy");
        assert!(sample_file.exists());

        // 执行单项扩展名关联引导 (wav)
        let res = prompt_associate_extension("wav");
        assert!(res.is_ok(), "prompt_associate_extension 应该成功: {:?}", res);

        // 验证遗留 sample 文件已被彻底清理
        assert!(!sample_file.exists(), "prompt_associate_extension 应自动清理 associate_sample 占位文件");

        // 验证 wav 已在 OpenWithProgids 中正确登记
        assert!(reg_value_exists(
            HKEY_CURRENT_USER,
            r"Software\Classes\.wav\OpenWithProgids",
            "PulsePlayer.AssocFile.wav"
        ));
    }
}


