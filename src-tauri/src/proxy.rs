// src-tauri/src/proxy.rs
// Windows 平台系统网络代理自动透视与环境变量无缝继承模块

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_DWORD,
    REG_SZ,
};

/// 将 Rust 字符串编码为 UTF-16 宽字符并添加 NULL 结尾
fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// 读取当前 Windows 系统代理配置（如果已开启）
pub fn get_system_proxy() -> Option<String> {
    unsafe {
        let subkey = to_wide_null(r"Software\Microsoft\Windows\CurrentVersion\Internet Settings");
        let mut hkey: HKEY = std::mem::zeroed();

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey)
            != ERROR_SUCCESS
        {
            return None;
        }

        // 1. 检查 ProxyEnable 是否为 1
        let val_enable = to_wide_null("ProxyEnable");
        let mut enable_val: u32 = 0;
        let mut data_size: u32 = std::mem::size_of::<u32>() as u32;
        let mut val_type: u32 = 0;

        let status = RegQueryValueExW(
            hkey,
            val_enable.as_ptr(),
            std::ptr::null(),
            &mut val_type,
            &mut enable_val as *mut u32 as *mut u8,
            &mut data_size,
        );

        if status != ERROR_SUCCESS || val_type != REG_DWORD || enable_val != 1 {
            RegCloseKey(hkey);
            return None;
        }

        // 2. 读取 ProxyServer 字符串 (如 "127.0.0.1:7897")
        let val_server = to_wide_null("ProxyServer");
        let mut buf = [0u8; 1024];
        let mut buf_size: u32 = buf.len() as u32;

        let status = RegQueryValueExW(
            hkey,
            val_server.as_ptr(),
            std::ptr::null(),
            &mut val_type,
            buf.as_mut_ptr(),
            &mut buf_size,
        );

        RegCloseKey(hkey);

        if status == ERROR_SUCCESS && val_type == REG_SZ && buf_size > 0 {
            let u16_slice =
                std::slice::from_raw_parts(buf.as_ptr() as *const u16, (buf_size as usize) / 2);
            let s = String::from_utf16_lossy(u16_slice)
                .trim_matches('\0')
                .trim()
                .to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }

        None
    }
}

/// 自动检测并继承 Windows 系统代理至进程内存环境变量
/// 使 Rust reqwest 及 updater 插件天然支持开启系统代理的客户端（Clash, v2rayN 等）
pub fn auto_inherit_system_proxy() {
    // 若当前环境已显式设置代理环境变量，则优先尊重用户既有设置
    if std::env::var("HTTP_PROXY").is_ok()
        || std::env::var("http_proxy").is_ok()
        || std::env::var("HTTPS_PROXY").is_ok()
        || std::env::var("https_proxy").is_ok()
        || std::env::var("ALL_PROXY").is_ok()
        || std::env::var("all_proxy").is_ok()
    {
        return;
    }

    if let Some(proxy_str) = get_system_proxy() {
        // 格式处理：可能是 "127.0.0.1:7890" 或 "http=127.0.0.1:7890;https=127.0.0.1:7890"
        if proxy_str.contains('=') {
            for entry in proxy_str.split(';') {
                let mut parts = entry.splitn(2, '=');
                if let (Some(proto), Some(addr)) = (parts.next(), parts.next()) {
                    let proto = proto.trim().to_lowercase();
                    let addr = addr.trim();
                    let full_url = if addr.starts_with("http://") || addr.starts_with("socks5://") {
                        addr.to_string()
                    } else {
                        format!("http://{}", addr)
                    };

                    match proto.as_str() {
                        "http" => {
                            std::env::set_var("HTTP_PROXY", &full_url);
                            std::env::set_var("http_proxy", &full_url);
                        }
                        "https" => {
                            std::env::set_var("HTTPS_PROXY", &full_url);
                            std::env::set_var("https_proxy", &full_url);
                        }
                        "socks" => {
                            std::env::set_var("ALL_PROXY", &full_url);
                            std::env::set_var("all_proxy", &full_url);
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // 单一地址默认适用所有协议
            let full_url = if proxy_str.starts_with("http://") || proxy_str.starts_with("socks5://") {
                proxy_str
            } else {
                format!("http://{}", proxy_str)
            };

            std::env::set_var("HTTP_PROXY", &full_url);
            std::env::set_var("http_proxy", &full_url);
            std::env::set_var("HTTPS_PROXY", &full_url);
            std::env::set_var("https_proxy", &full_url);
            std::env::set_var("ALL_PROXY", &full_url);
            std::env::set_var("all_proxy", &full_url);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_proxy_structure() {
        // 读取本地系统代理结构，确保不崩溃
        let proxy = get_system_proxy();
        // 若开启了系统代理，应成功获取非空字符串
        if let Some(ref p) = proxy {
            assert!(!p.is_empty());
        }
    }

    #[test]
    fn test_auto_inherit_system_proxy() {
        auto_inherit_system_proxy();
        // 验证执行后不会产生未定义状态
        if let Some(proxy) = get_system_proxy() {
            let http_proxy = std::env::var("HTTP_PROXY").or_else(|_| std::env::var("http_proxy"));
            assert!(http_proxy.is_ok(), "当检测到系统代理开启时，环境变量应成功注入");
            assert!(http_proxy.unwrap().contains(&proxy) || proxy.contains('='));
        }
    }
}
