// src-tauri/src/mpv_player.rs
// 基于 libmpv 原生 C-FFI 接口的高性能视频播放器引擎 (官方双客户端解耦架构)

use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::{c_char, c_double, c_int, c_void};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};
use crate::history::HistoryRecord;

#[repr(C)]
pub struct mpv_handle {
    _unused: [u8; 0],
}

#[repr(C)]
struct mpv_event {
    event_id: c_int,
    error: c_int,
    reply_userdata: u64,
    data: *mut c_void,
}

#[repr(C)]
struct mpv_event_property {
    name: *const c_char,
    format: c_int,
    data: *mut c_void,
}

const MPV_EVENT_SHUTDOWN: c_int = 1;
const MPV_EVENT_END_FILE: c_int = 7;
const MPV_EVENT_PROPERTY_CHANGE: c_int = 22;

const MPV_FORMAT_DOUBLE: c_int = 5;
const MPV_FORMAT_FLAG: c_int = 3;

/// mpv end-file 事件原因定义
const MPV_END_FILE_REASON_EOF: c_int = 0;
#[allow(dead_code)]
const MPV_END_FILE_REASON_STOP: c_int = 2;

#[repr(C)]
struct mpv_event_end_file {
    reason: c_int,
    error: c_int,
    playlist_entry_id: i64,
}

#[link(name = "mpv")]
extern "C" {
    fn mpv_create() -> *mut mpv_handle;
    fn mpv_create_client(ctx: *mut mpv_handle, name: *const c_char) -> *mut mpv_handle;
    fn mpv_initialize(ctx: *mut mpv_handle) -> c_int;
    fn mpv_command(ctx: *mut mpv_handle, args: *const *const c_char) -> c_int;
    fn mpv_set_option_string(ctx: *mut mpv_handle, name: *const c_char, data: *const c_char) -> c_int;
    fn mpv_set_property_string(ctx: *mut mpv_handle, name: *const c_char, data: *const c_char) -> c_int;
    fn mpv_get_property_string(ctx: *mut mpv_handle, name: *const c_char) -> *mut c_char;
    fn mpv_observe_property(ctx: *mut mpv_handle, reply_userdata: u64, name: *const c_char, format: c_int) -> c_int;
    fn mpv_wait_event(ctx: *mut mpv_handle, timeout: c_double) -> *mut mpv_event;
    fn mpv_free(data: *mut c_void);
    fn mpv_destroy(ctx: *mut mpv_handle);
    fn mpv_terminate_destroy(ctx: *mut mpv_handle);
}


pub struct MpvPlayer {
    raw: *mut mpv_handle,
    current_path: Arc<Mutex<Option<String>>>,
    current_time: Arc<Mutex<f64>>,
    duration: Arc<Mutex<f64>>,
    eof_reached: Arc<Mutex<bool>>,
}

unsafe impl Send for MpvPlayer {}
unsafe impl Sync for MpvPlayer {}

impl MpvPlayer {
    pub fn new(app_handle: AppHandle, wid: Option<isize>) -> Result<Self, String> {
        let raw = unsafe { mpv_create() };
        if raw.is_null() {
            return Err("创建 mpv 句柄失败".into());
        }

        unsafe {
            // 基础配置：关闭 mpv 自带粗糙的控制条与文字，由前端完全复刻 B 站 UI
            Self::set_opt(raw, "osc", "no");
            Self::set_opt(raw, "osd-level", "0");
            Self::set_opt(raw, "keep-open", "yes");
            Self::set_opt(raw, "hwdec", "auto-safe");
            Self::set_opt(raw, "hr-seek", "yes"); // 高精度跳转
            Self::set_opt(raw, "vo", "gpu");
            Self::set_opt(raw, "force-window", "yes");

            // 若传入了目标窗口 HWND，则将画面绑定至该窗口渲染
            if let Some(hwnd) = wid {
                let wid_str = format!("{}", hwnd);
                Self::set_opt(raw, "wid", &wid_str);
            }

            let init_res = mpv_initialize(raw);
            if init_res < 0 {
                mpv_terminate_destroy(raw);
                return Err(format!("初始化 mpv 失败: {}", init_res));
            }
        }

        // 官方双客户端解耦架构：创建专门用于事件监听的独立客户端，彻底消除锁竞争
        let c_client_name = CString::new("events").unwrap();
        let event_raw = unsafe { mpv_create_client(raw, c_client_name.as_ptr()) };
        if event_raw.is_null() {
            unsafe { mpv_terminate_destroy(raw) };
            return Err("创建 mpv 事件客户端失败".into());
        }

        unsafe {
            // 在事件客户端句柄上注册属性监听，完全不占用主客户端锁
            Self::observe(event_raw, 1, "time-pos", MPV_FORMAT_DOUBLE);
            Self::observe(event_raw, 2, "duration", MPV_FORMAT_DOUBLE);
            Self::observe(event_raw, 3, "pause", MPV_FORMAT_FLAG);
            Self::observe(event_raw, 4, "eof-reached", MPV_FORMAT_FLAG);
        }

        let current_path = Arc::new(Mutex::new(None));
        let current_time = Arc::new(Mutex::new(0.0));
        let duration = Arc::new(Mutex::new(0.0));
        let eof_reached = Arc::new(Mutex::new(false));

        let current_time_clone = Arc::clone(&current_time);
        let duration_clone = Arc::clone(&duration);
        let eof_reached_clone = Arc::clone(&eof_reached);
        let event_raw_usize = event_raw as usize;

        // 启动后台事件轮询线程：独立等待，绝不持有前端 IPC 锁
        thread::spawn(move || {
            let handle = event_raw_usize as *mut mpv_handle;
            loop {
                let event = unsafe { mpv_wait_event(handle, 0.25) };
                if event.is_null() {
                    continue;
                }

                unsafe {
                    let ev = &*event;
                    if ev.event_id == MPV_EVENT_SHUTDOWN {
                        mpv_destroy(handle);
                        break;
                    }

                    if ev.event_id == MPV_EVENT_PROPERTY_CHANGE {
                        let prop = &*(ev.data as *mut mpv_event_property);
                        if !prop.name.is_null() && !prop.data.is_null() {
                            let name = CStr::from_ptr(prop.name).to_string_lossy();
                            if name == "time-pos" {
                                let time = *(prop.data as *mut f64);
                                *current_time_clone.lock().unwrap() = time;
                                let _ = app_handle.emit("player:time", time);
                            } else if name == "duration" {
                                let dur = *(prop.data as *mut f64);
                                *duration_clone.lock().unwrap() = dur;
                                let _ = app_handle.emit("player:duration", dur);
                            } else if name == "pause" {
                                let paused = *(prop.data as *mut c_int) != 0;
                                let _ = app_handle.emit("player:pause", paused);
                            } else if name == "eof-reached" {
                                let is_eof = *(prop.data as *mut c_int) != 0;
                                if is_eof {
                                    let dur = *duration_clone.lock().unwrap();
                                    // 仅当文件已有效加载且总时长大于 0 时，才派发自然播放结束事件
                                    if dur > 0.0 {
                                        let mut guard = eof_reached_clone.lock().unwrap();
                                        if !*guard {
                                            *guard = true;
                                            let _ = app_handle.emit("player:ended", ());
                                        }
                                    }
                                } else {
                                    *eof_reached_clone.lock().unwrap() = false;
                                }
                            }
                        }
                    } else if ev.event_id == MPV_EVENT_END_FILE {
                        if !ev.data.is_null() {
                            let end_file = &*(ev.data as *mut mpv_event_end_file);
                            // 仅当文件真正自然播放至末尾 (EOF) 时派发 ended 事件
                            // 严禁在切换文件 (STOP) 或错误退出时派发，杜绝启动切集误触发
                            if end_file.reason == MPV_END_FILE_REASON_EOF {
                                let mut guard = eof_reached_clone.lock().unwrap();
                                if !*guard {
                                    *guard = true;
                                    let _ = app_handle.emit("player:ended", ());
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            raw,
            current_path,
            current_time,
            duration,
            eof_reached,
        })
    }

    unsafe fn set_opt(handle: *mut mpv_handle, name: &str, value: &str) {
        if let (Ok(c_name), Ok(c_val)) = (CString::new(name), CString::new(value)) {
            mpv_set_option_string(handle, c_name.as_ptr(), c_val.as_ptr());
        }
    }

    unsafe fn observe(handle: *mut mpv_handle, id: u64, name: &str, format: c_int) {
        if let Ok(c_name) = CString::new(name) {
            mpv_observe_property(handle, id, c_name.as_ptr(), format);
        }
    }

    fn get_hflip_shader_path() -> PathBuf {
        let dir = if let Ok(app_data) = std::env::var("APPDATA") {
            PathBuf::from(app_data).join("Pulse-Player")
        } else {
            std::env::temp_dir().join("Pulse-Player")
        };
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("hflip.glsl");
        if !file_path.exists() {
            let shader_content = "//!HOOK MAIN\n//!BIND HOOKED\n//!DESC Horizontally Mirror Video\nvec4 hook() {\n    vec2 pos = HOOKED_pos;\n    pos.x = 1.0 - pos.x;\n    return HOOKED_tex(pos);\n}\n";
            let _ = fs::write(&file_path, shader_content);
        }
        file_path
    }

    /// 获取当前播放记录，用于窗口关闭时同步兜底保存
    pub fn get_current_record(&self) -> Option<HistoryRecord> {
        let path = self.current_path.lock().unwrap().clone()?;
        let is_eof = *self.eof_reached.lock().unwrap();
        let cur_time = *self.current_time.lock().unwrap();
        let duration = *self.duration.lock().unwrap();
        // 自然播放至末尾或最后 1.5 秒内关闭窗口时，进度归零从头开始
        let timestamp = if is_eof || (duration > 0.0 && cur_time >= duration - 1.5) {
            0.0
        } else {
            cur_time
        };
        let p = std::path::Path::new(&path);
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path)
            .to_string();
        Some(HistoryRecord {
            name,
            path,
            timestamp,
            duration,
            last_played_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        })
    }

    /// 加载视频文件并支持指定起始跳转时间
    pub fn load_file(&self, path: &str, start_time: Option<f64>) -> Result<(), String> {
        if self.raw.is_null() {
            return Err("mpv 句柄已销毁".into());
        }

        *self.current_path.lock().unwrap() = Some(path.to_string());
        *self.current_time.lock().unwrap() = start_time.unwrap_or(0.0);
        *self.duration.lock().unwrap() = 0.0;
        *self.eof_reached.lock().unwrap() = false;

        unsafe {
            let c_cmd = CString::new("loadfile").map_err(|e| e.to_string())?;
            let c_path = CString::new(path).map_err(|e| e.to_string())?;

            let res = if let Some(t) = start_time {
                if t > 0.0 {
                    let c_replace = CString::new("replace").map_err(|e| e.to_string())?;
                    let c_index = CString::new("-1").map_err(|e| e.to_string())?;
                    let c_start = CString::new(format!("start={:.3}", t)).map_err(|e| e.to_string())?;
                    let args = [
                        c_cmd.as_ptr(),
                        c_path.as_ptr(),
                        c_replace.as_ptr(),
                        c_index.as_ptr(),
                        c_start.as_ptr(),
                        std::ptr::null(),
                    ];
                    mpv_command(self.raw, args.as_ptr())
                } else {
                    let args = [c_cmd.as_ptr(), c_path.as_ptr(), std::ptr::null()];
                    mpv_command(self.raw, args.as_ptr())
                }
            } else {
                let args = [c_cmd.as_ptr(), c_path.as_ptr(), std::ptr::null()];
                mpv_command(self.raw, args.as_ptr())
            };

            if res < 0 {
                return Err(format!("加载视频失败: {}", res));
            }

            // 无论之前是否处于暂停状态，加载新视频时显式解除暂停 (pause=no)，保证起播即播与播放按钮状态对齐
            let c_pause = CString::new("pause").map_err(|e| e.to_string())?;
            let c_no = CString::new("no").map_err(|e| e.to_string())?;
            mpv_set_property_string(self.raw, c_pause.as_ptr(), c_no.as_ptr());
        }
        Ok(())
    }

    /// 切换 播放 / 暂停
    pub fn toggle_pause(&self) -> Result<bool, String> {
        unsafe {
            let prop = CString::new("pause").unwrap();
            let val_ptr = mpv_get_property_string(self.raw, prop.as_ptr());
            let is_paused = if !val_ptr.is_null() {
                let s = CStr::from_ptr(val_ptr).to_string_lossy();
                let paused = s == "yes";
                mpv_free(val_ptr as *mut c_void);
                paused
            } else {
                false
            };

            let next_paused = !is_paused;
            let next_val = CString::new(if next_paused { "yes" } else { "no" }).unwrap();
            mpv_set_property_string(self.raw, prop.as_ptr(), next_val.as_ptr());

            Ok(next_paused)
        }
    }

    /// 明确设置暂停状态
    pub fn set_pause(&self, paused: bool) -> Result<(), String> {
        unsafe {
            let prop = CString::new("pause").unwrap();
            let val = CString::new(if paused { "yes" } else { "no" }).unwrap();
            mpv_set_property_string(self.raw, prop.as_ptr(), val.as_ptr());
        }
        Ok(())
    }

    /// 执行 Seek 跳转
    pub fn seek(&self, seconds: f64, relative: bool) -> Result<(), String> {
        *self.eof_reached.lock().unwrap() = false;
        unsafe {
            let c_cmd = CString::new("seek").map_err(|e| e.to_string())?;
            let c_sec = CString::new(format!("{:.3}", seconds)).map_err(|e| e.to_string())?;
            let c_mode = CString::new(if relative { "relative" } else { "absolute" }).map_err(|e| e.to_string())?;
            let args = [c_cmd.as_ptr(), c_sec.as_ptr(), c_mode.as_ptr(), std::ptr::null()];
            mpv_command(self.raw, args.as_ptr());
        }
        Ok(())
    }

    /// 设置播放倍速
    pub fn set_speed(&self, speed: f64) -> Result<(), String> {
        unsafe {
            let prop = CString::new("speed").unwrap();
            let val = CString::new(format!("{:.2}", speed)).unwrap();
            mpv_set_property_string(self.raw, prop.as_ptr(), val.as_ptr());
        }
        Ok(())
    }

    /// 设置应用独立音量 (0 - 100)
    pub fn set_volume(&self, volume: f64) -> Result<(), String> {
        unsafe {
            let prop = CString::new("volume").unwrap();
            let val = CString::new(format!("{:.1}", volume)).unwrap();
            mpv_set_property_string(self.raw, prop.as_ptr(), val.as_ptr());
        }
        Ok(())
    }

    /// 设置视频比例 ("auto", "4:3", "16:9", "fill" 等) - GPU 零开销硬件缩放
    pub fn set_aspect_ratio(&self, ratio: &str) -> Result<(), String> {
        unsafe {
            let (keepaspect, aspect_override) = match ratio {
                "4:3" => ("yes", "4:3"),
                "16:9" => ("yes", "16:9"),
                "fill" => ("no", "-1"), // 完全填满视口，禁用 keepaspect
                _ => ("yes", "-1"),     // 自动/原始比例
            };
            let prop_keep = CString::new("keepaspect").unwrap();
            let val_keep = CString::new(keepaspect).unwrap();
            mpv_set_property_string(self.raw, prop_keep.as_ptr(), val_keep.as_ptr());

            let prop_aspect = CString::new("video-aspect-override").unwrap();
            let val_aspect = CString::new(aspect_override).unwrap();
            mpv_set_property_string(self.raw, prop_aspect.as_ptr(), val_aspect.as_ptr());
        }
        Ok(())
    }

    /// 设置画面顺时针旋转角度 (0, 90, 180, 270) - GPU 顶点变换
    pub fn set_rotation(&self, degree: i32) -> Result<(), String> {
        unsafe {
            let prop = CString::new("video-rotate").unwrap();
            let val = CString::new(format!("{}", degree)).unwrap();
            mpv_set_property_string(self.raw, prop.as_ptr(), val.as_ptr());
        }
        Ok(())
    }

    /// 设置水平镜像 (翻转) - 采用 GPU 零开销着色器，不打断硬件解码管线
    pub fn set_mirror(&self, enabled: bool) -> Result<(), String> {
        unsafe {
            let prop = CString::new("glsl-shaders").unwrap();
            let val = if enabled {
                let shader_path = Self::get_hflip_shader_path();
                let path_str = shader_path.to_string_lossy().replace('\\', "/");
                CString::new(path_str).map_err(|e| e.to_string())?
            } else {
                CString::new("").unwrap()
            };
            let ret = mpv_set_property_string(self.raw, prop.as_ptr(), val.as_ptr());
            if ret < 0 {
                return Err(format!("设置水平镜像着色器失败: {}", ret));
            }
        }
        Ok(())
    }

    /// 切换解码模式 ("auto-safe", "auto-copy", "no")
    pub fn set_hwdec(&self, mode: &str) -> Result<(), String> {
        unsafe {
            let prop = CString::new("hwdec").unwrap();
            let val = CString::new(mode).unwrap();
            mpv_set_property_string(self.raw, prop.as_ptr(), val.as_ptr());
        }
        Ok(())
    }

    /// 设置视频顶部边距比例 (0.0 - 0.5)，用于窗口模式下避免画面被物理顶栏 (TitleBar) 遮挡
    pub fn set_top_margin_ratio(&self, ratio: f64) -> Result<(), String> {
        if self.raw.is_null() {
            return Err("mpv 句柄已销毁".into());
        }
        unsafe {
            let prop = CString::new("video-margin-ratio-top").unwrap();
            let safe_ratio = ratio.max(0.0).min(0.5);
            let val = CString::new(format!("{:.6}", safe_ratio)).unwrap();
            let ret = mpv_set_property_string(self.raw, prop.as_ptr(), val.as_ptr());
            if ret < 0 {
                return Err(format!("设置视频顶部边距比例失败: {}", ret));
            }
        }
        Ok(())
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                mpv_terminate_destroy(self.raw);
            }
            self.raw = std::ptr::null_mut();
        }
    }
}
