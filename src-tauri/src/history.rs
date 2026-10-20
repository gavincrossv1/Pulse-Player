// src-tauri/src/history.rs
// 管理最近播放历史与断点续播时间戳（最多保留最近 5 项）

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryRecord {
    pub name: String,
    pub path: String,
    pub timestamp: f64, // 上次退出的播放进度（秒）
    pub duration: f64,  // 总时长（秒）
    pub last_played_at: u64, // 更新时间戳（毫秒）
}

/// 获取持久化存储路径
fn get_history_file_path() -> PathBuf {
    // 优先支持测试/沙箱隔离环境变量，彻底防止自动化测试读写用户真实生产数据
    if let Ok(sandbox_dir) = std::env::var("PULSE_DATA_DIR") {
        let dir = PathBuf::from(sandbox_dir);
        let _ = fs::create_dir_all(&dir);
        return dir.join("history.json");
    }

    if let Ok(app_data) = std::env::var("APPDATA") {
        let dir = PathBuf::from(&app_data).join("Pulse-Player");
        let _ = fs::create_dir_all(&dir);
        let new_file = dir.join("history.json");
        // 平滑迁移旧版历史记录
        if !new_file.exists() {
            let old_file = PathBuf::from(&app_data).join("Bi-Player").join("history.json");
            if old_file.exists() {
                let _ = fs::copy(&old_file, &new_file);
            }
        }
        new_file
    } else {
        PathBuf::from("history.json")
    }
}

/// 读取历史记录列表（最近 5 个）
pub fn load_history() -> Vec<HistoryRecord> {
    let path = get_history_file_path();
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(records) = serde_json::from_str::<Vec<HistoryRecord>>(&content) {
            return records;
        }
    }
    Vec::new()
}

/// 保存或更新单条历史记录 (统一规范化路径以杜绝 Windows 大小写与斜杠差异)
pub fn save_history_record(mut record: HistoryRecord) {
    record.path = crate::commands::normalize_path(&record.path);
    let mut records = load_history();

    // 若传入的 duration 未就绪 (<=0)，优先继承已有记录中的有效时长，杜绝切流竞态覆盖
    if record.duration <= 0.0 {
        if let Some(existing) = records.iter().find(|r| crate::commands::is_same_path(&r.path, &record.path)) {
            if existing.duration > 0.0 {
                record.duration = existing.duration;
            }
        }
    }

    // 移除已存在的相同路径记录 (统一规范化与斜杠/大小写无关比对)
    records.retain(|r| !crate::commands::is_same_path(&r.path, &record.path));

    // 将最新记录置于最前
    records.insert(0, record);

    // 严格限制只保留最近 5 条
    if records.len() > 5 {
        records.truncate(5);
    }

    let path = get_history_file_path();
    if let Ok(json) = serde_json::to_string_pretty(&records) {
        let _ = fs::write(path, json);
    }
}

/// 删除指定路径对应的单条历史记录 (支持路径规范化比对)
pub fn delete_history_record(target_path: &str) {
    let mut records = load_history();
    records.retain(|r| !crate::commands::is_same_path(&r.path, target_path));
    let path = get_history_file_path();
    if let Ok(json) = serde_json::to_string_pretty(&records) {
        let _ = fs::write(path, json);
    }
}

/// 清空所有历史播放记录
pub fn clear_history() {
    let path = get_history_file_path();
    if let Ok(json) = serde_json::to_string_pretty(&Vec::<HistoryRecord>::new()) {
        let _ = fs::write(path, json);
    }
}

