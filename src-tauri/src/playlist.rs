// src-tauri/src/playlist.rs
// 智能扫描同目录下主流视频文件，并执行人类自然的自然排序 (Natural Sort)

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// 播放列表条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistItem {
    pub name: String,
    pub path: String,
}

/// 支持的主流视频与音频媒体格式扩展名
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    // 常见视频格式
    "mp4", "mkv", "avi", "flv", "mov", "wmv", "webm", "ts",
    "m4v", "rmvb", "rm", "m2ts", "vob", "mpg", "mpeg", "3gp",
    "f4v", "ogv",
    // 常见音频格式
    "mp3", "flac", "wav", "m4a", "aac", "ogg", "ape", "opus", "wma",
];

/// 自然排序拆分键（按连续数字与非数字分割，数字块按数值比较，数字优先于文字符合 Windows 标准）
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum KeyChunk {
    Number(u64),
    Text(String),
}

/// 将文件名解析为自然排序比较键序列
fn natural_sort_key(s: &str) -> Vec<KeyChunk> {
    let mut chunks = Vec::new();
    let mut current_text = String::new();
    let mut current_digits = String::new();

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            if !current_text.is_empty() {
                chunks.push(KeyChunk::Text(current_text.to_lowercase()));
                current_text.clear();
            }
            current_digits.push(ch);
        } else {
            if !current_digits.is_empty() {
                if let Ok(num) = current_digits.parse::<u64>() {
                    chunks.push(KeyChunk::Number(num));
                }
                current_digits.clear();
            }
            current_text.push(ch);
        }
    }

    if !current_digits.is_empty() {
        if let Ok(num) = current_digits.parse::<u64>() {
            chunks.push(KeyChunk::Number(num));
        }
    } else if !current_text.is_empty() {
        chunks.push(KeyChunk::Text(current_text.to_lowercase()));
    }

    chunks
}

/// 扫描指定视频文件所在文件夹中的所有支持格式视频，并按文件名自然排序
pub fn scan_folder_videos(current_file_path: &str) -> Vec<PlaylistItem> {
    let path = Path::new(current_file_path);
    let parent_dir = match path.parent() {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut items = Vec::new();

    if let Ok(entries) = fs::read_dir(parent_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                        let name = entry_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        let full_path = crate::commands::normalize_path(&entry_path.to_string_lossy());
                        items.push(PlaylistItem { name, path: full_path });
                    }
                }
            }
        }
    }

    // 执行自然排序（例如：1, 2, 10 而非 1, 10, 2）
    items.sort_by(|a, b| natural_sort_key(&a.name).cmp(&natural_sort_key(&b.name)));

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_sort() {
        let mut files = vec![
            "第10集 终章.mp4".to_string(),
            "第2集 启程.mp4".to_string(),
            "第1集 序幕.mp4".to_string(),
            "第20集 外传.mp4".to_string(),
        ];
        files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

        assert_eq!(
            files,
            vec![
                "第1集 序幕.mp4".to_string(),
                "第2集 启程.mp4".to_string(),
                "第10集 终章.mp4".to_string(),
                "第20集 外传.mp4".to_string(),
            ]
        );
    }

    #[test]
    fn test_natural_sort_number_before_text() {
        let mut files = vec![
            "徐嘉文 智算13班 3023244339.mp4".to_string(),
            "2e24e206423fd6b35cc3c535b37ee7ce.mp4".to_string(),
            "1.mp4".to_string(),
            "test_sample.mp4".to_string(),
        ];
        files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

        assert_eq!(
            files,
            vec![
                "1.mp4".to_string(),
                "2e24e206423fd6b35cc3c535b37ee7ce.mp4".to_string(),
                "test_sample.mp4".to_string(),
                "徐嘉文 智算13班 3023244339.mp4".to_string(),
            ]
        );
    }
}
