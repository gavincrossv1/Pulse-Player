use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_void};
use std::path::PathBuf;

#[repr(C)]
struct mpv_handle {
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

const MPV_END_FILE_REASON_EOF: c_int = 0;
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

#[test]
fn test_mpv_loading() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        let dur_prop = CString::new("duration").unwrap();
        mpv_observe_property(raw, 2, dur_prop.as_ptr(), MPV_FORMAT_DOUBLE);

        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let video_path = manifest_dir.parent().unwrap().join("tests").join("fixtures").join("test_video.mp4");
        let video_str = video_path.to_str().unwrap();

        let cmd_name = CString::new("loadfile").unwrap();
        let file_arg = CString::new(video_str).unwrap();
        let args = [cmd_name.as_ptr(), file_arg.as_ptr(), std::ptr::null()];
        let load_res = mpv_command(raw, args.as_ptr());
        assert_eq!(load_res, 0);

        let start = std::time::Instant::now();
        let mut got_duration = false;
        while start.elapsed().as_secs() < 3 {
            let event = mpv_wait_event(raw, 0.1);
            if event.is_null() {
                continue;
            }
            let ev = &*event;
            if ev.event_id == MPV_EVENT_PROPERTY_CHANGE {
                let prop = &*(ev.data as *mut mpv_event_property);
                if !prop.name.is_null() && !prop.data.is_null() {
                    let name = CStr::from_ptr(prop.name).to_string_lossy();
                    if name == "duration" {
                        let d = *(prop.data as *mut f64);
                        assert!(d > 0.0);
                        got_duration = true;
                        break;
                    }
                }
            }
        }

        mpv_terminate_destroy(raw);
        assert!(got_duration, "Should have received duration for test video");
    }
}

#[test]
fn test_mpv_unicode_and_spaces_path() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        let dur_prop = CString::new("duration").unwrap();
        mpv_observe_property(raw, 2, dur_prop.as_ptr(), MPV_FORMAT_DOUBLE);

        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let src_video = manifest_dir.parent().unwrap().join("tests").join("fixtures").join("test_video.mp4");
        
        // 创建包含中文和空格的真实测试文件
        let temp_dir = std::env::temp_dir();
        let unicode_video = temp_dir.join("徐嘉文 智算13班 3023244339.mp4");
        let _ = std::fs::copy(&src_video, &unicode_video);

        let video_str = unicode_video.to_string_lossy().to_string();

        let cmd_name = CString::new("loadfile").unwrap();
        let file_arg = CString::new(video_str.as_str()).unwrap();
        let args = [cmd_name.as_ptr(), file_arg.as_ptr(), std::ptr::null()];
        let load_res = mpv_command(raw, args.as_ptr());
        assert_eq!(load_res, 0);

        let start = std::time::Instant::now();
        let mut got_duration = false;
        while start.elapsed().as_secs() < 3 {
            let event = mpv_wait_event(raw, 0.1);
            if event.is_null() {
                continue;
            }
            let ev = &*event;
            if ev.event_id == MPV_EVENT_PROPERTY_CHANGE {
                let prop = &*(ev.data as *mut mpv_event_property);
                if !prop.name.is_null() && !prop.data.is_null() {
                    let name = CStr::from_ptr(prop.name).to_string_lossy();
                    if name == "duration" {
                        let d = *(prop.data as *mut f64);
                        assert!(d > 0.0);
                        got_duration = true;
                        break;
                    }
                }
            }
        }

        mpv_terminate_destroy(raw);
        let _ = std::fs::remove_file(unicode_video);
        assert!(got_duration, "Should load video with spaces and non-ASCII chars");
    }
}

#[test]
fn test_mpv_playback_controls() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        // 倍速
        let speed_prop = CString::new("speed").unwrap();
        let speed_val = CString::new("1.50").unwrap();
        mpv_set_property_string(raw, speed_prop.as_ptr(), speed_val.as_ptr());
        let speed_ret = mpv_get_property_string(raw, speed_prop.as_ptr());
        assert!(!speed_ret.is_null());
        assert_eq!(CStr::from_ptr(speed_ret).to_string_lossy(), "1.500000");
        mpv_free(speed_ret as *mut c_void);

        // 音量
        let vol_prop = CString::new("volume").unwrap();
        let vol_val = CString::new("65.0").unwrap();
        mpv_set_property_string(raw, vol_prop.as_ptr(), vol_val.as_ptr());
        let vol_ret = mpv_get_property_string(raw, vol_prop.as_ptr());
        assert!(!vol_ret.is_null());
        assert_eq!(CStr::from_ptr(vol_ret).to_string_lossy(), "65.000000");
        mpv_free(vol_ret as *mut c_void);

        // 旋转
        let rot_prop = CString::new("video-rotate").unwrap();
        let rot_val = CString::new("90").unwrap();
        mpv_set_property_string(raw, rot_prop.as_ptr(), rot_val.as_ptr());
        let rot_ret = mpv_get_property_string(raw, rot_prop.as_ptr());
        assert!(!rot_ret.is_null());
        assert_eq!(CStr::from_ptr(rot_ret).to_string_lossy(), "90");
        mpv_free(rot_ret as *mut c_void);

        // 比例
        let aspect_prop = CString::new("video-aspect-override").unwrap();
        let aspect_val = CString::new("16:9").unwrap();
        mpv_set_property_string(raw, aspect_prop.as_ptr(), aspect_val.as_ptr());
        let aspect_ret = mpv_get_property_string(raw, aspect_prop.as_ptr());
        assert!(!aspect_ret.is_null());
        let aspect_str = CStr::from_ptr(aspect_ret).to_string_lossy();
        assert!(aspect_str == "1.777778" || aspect_str == "16:9");
        mpv_free(aspect_ret as *mut c_void);

        mpv_terminate_destroy(raw);
    }
}

#[test]
fn test_mpv_start_time_resume() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        let time_prop = CString::new("time-pos").unwrap();
        mpv_observe_property(raw, 1, time_prop.as_ptr(), MPV_FORMAT_DOUBLE);

        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let video_path = manifest_dir.parent().unwrap().join("tests").join("fixtures").join("test_video.mp4");
        let video_str = video_path.to_str().unwrap();

        let cmd_name = CString::new("loadfile").unwrap();
        let file_arg = CString::new(video_str).unwrap();
        let replace_arg = CString::new("replace").unwrap();
        let index_arg = CString::new("-1").unwrap();
        let start_arg = CString::new("start=3.000").unwrap();
        let args = [
            cmd_name.as_ptr(),
            file_arg.as_ptr(),
            replace_arg.as_ptr(),
            index_arg.as_ptr(),
            start_arg.as_ptr(),
            std::ptr::null(),
        ];
        let load_res = mpv_command(raw, args.as_ptr());
        assert_eq!(load_res, 0);

        let start = std::time::Instant::now();
        let mut got_start_time = false;
        while start.elapsed().as_secs() < 3 {
            let event = mpv_wait_event(raw, 0.1);
            if event.is_null() {
                continue;
            }
            let ev = &*event;
            if ev.event_id == MPV_EVENT_PROPERTY_CHANGE {
                let prop = &*(ev.data as *mut mpv_event_property);
                if !prop.name.is_null() && !prop.data.is_null() {
                    let name = CStr::from_ptr(prop.name).to_string_lossy();
                    if name == "time-pos" {
                        let t = *(prop.data as *mut f64);
                        if t >= 2.5 {
                            got_start_time = true;
                            break;
                        }
                    }
                }
            }
        }

        mpv_terminate_destroy(raw);
        assert!(got_start_time, "Should have started playback at specified start_time (>= 2.5s)");
    }
}

#[test]
fn test_dual_client_decoupled_events() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        // 创建独立的事件客户端
        let c_client = CString::new("events_test").unwrap();
        let event_raw = mpv_create_client(raw, c_client.as_ptr());
        assert!(!event_raw.is_null());

        // 在事件客户端注册监听
        let pause_prop = CString::new("pause").unwrap();
        mpv_observe_property(event_raw, 3, pause_prop.as_ptr(), MPV_FORMAT_FLAG);

        let event_raw_usize = event_raw as usize;
        let (tx, rx) = std::sync::mpsc::channel();

        // 像生产代码一样在独立后台线程中轮询事件
        let event_thread = std::thread::spawn(move || {
            let handle = event_raw_usize as *mut mpv_handle;
            loop {
                let event = mpv_wait_event(handle, 0.1);
                if event.is_null() {
                    continue;
                }
                let ev = &*event;
                if ev.event_id == MPV_EVENT_SHUTDOWN {
                    mpv_destroy(handle);
                    break;
                }
                if ev.event_id == MPV_EVENT_PROPERTY_CHANGE {
                    let prop = &*(ev.data as *mut mpv_event_property);
                    if !prop.name.is_null() && !prop.data.is_null() {
                        let name = CStr::from_ptr(prop.name).to_string_lossy();
                        if name == "pause" {
                            let paused = *(prop.data as *mut c_int) != 0;
                            if paused {
                                let _ = tx.send(true);
                            }
                        }
                    }
                }
            }
        });

        // 主句柄设置 pause=yes
        let yes_val = CString::new("yes").unwrap();
        mpv_set_property_string(raw, pause_prop.as_ptr(), yes_val.as_ptr());

        let received = rx.recv_timeout(std::time::Duration::from_secs(3)).unwrap_or(false);
        assert!(received, "事件客户端后台线程应能独立收到主句柄发起的 pause 属性变化");

        // 终止主句柄：后台线程收到 MPV_EVENT_SHUTDOWN 并调用 mpv_destroy，解除主句柄等待
        mpv_terminate_destroy(raw);
        let _ = event_thread.join();
    }
}

#[test]
fn test_gpu_aspect_ratio_modes() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());
        let res = mpv_initialize(raw);
        assert!(res >= 0);

        // 铺满模式：keepaspect=no, video-aspect-override=-1
        let keep_prop = CString::new("keepaspect").unwrap();
        let no_val = CString::new("no").unwrap();
        mpv_set_property_string(raw, keep_prop.as_ptr(), no_val.as_ptr());
        let keep_ret = mpv_get_property_string(raw, keep_prop.as_ptr());
        assert_eq!(CStr::from_ptr(keep_ret).to_string_lossy(), "no");
        mpv_free(keep_ret as *mut c_void);

        // 16:9 模式：keepaspect=yes, video-aspect-override=16:9
        let yes_val = CString::new("yes").unwrap();
        mpv_set_property_string(raw, keep_prop.as_ptr(), yes_val.as_ptr());
        let aspect_prop = CString::new("video-aspect-override").unwrap();
        let val_16_9 = CString::new("16:9").unwrap();
        mpv_set_property_string(raw, aspect_prop.as_ptr(), val_16_9.as_ptr());
        let aspect_ret = mpv_get_property_string(raw, aspect_prop.as_ptr());
        let aspect_str = CStr::from_ptr(aspect_ret).to_string_lossy();
        assert!(aspect_str == "16:9" || aspect_str == "1.777778");
        mpv_free(aspect_ret as *mut c_void);

        mpv_terminate_destroy(raw);
    }
}

static HISTORY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 单元测试历史记录沙箱守卫：将数据重定向至系统临时目录，并在 Drop 时彻底自毁，防止污染生产环境
struct HistoryTestSandbox {
    dir: PathBuf,
}

impl HistoryTestSandbox {
    fn new(test_name: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pulse_history_sandbox_{}_{}", test_name, unique));
        let _ = std::fs::create_dir_all(&dir);
        std::env::set_var("PULSE_DATA_DIR", &dir);
        Self { dir }
    }
}

impl Drop for HistoryTestSandbox {
    fn drop(&mut self) {
        std::env::remove_var("PULSE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn test_history_path_normalization() {
    let _lock = HISTORY_TEST_LOCK.lock().unwrap();
    let _sandbox = HistoryTestSandbox::new("path_norm");
    use pulse_player_lib::commands::is_same_path;
    use pulse_player_lib::history::{save_history_record, load_history, HistoryRecord};

    let p1 = "C:\\test_video_sample.mp4";
    let p2 = "c:/test_video_sample.mp4";

    let r1 = HistoryRecord {
        name: "test1".into(),
        path: p1.into(),
        timestamp: 10.0,
        duration: 100.0,
        last_played_at: 1000,
    };
    save_history_record(r1);

    // 覆盖更新同名但大小写/斜杠不同的同一路径
    let r2 = HistoryRecord {
        name: "test2".into(),
        path: p2.into(),
        timestamp: 25.0,
        duration: 100.0,
        last_played_at: 2000,
    };
    save_history_record(r2);

    let records = load_history();
    let matches: Vec<_> = records.iter().filter(|r| is_same_path(&r.path, p2)).collect();
    assert_eq!(matches.len(), 1, "同一路径不同斜杠/大小写应被归一化去重并仅保留一条");
    assert_eq!(matches[0].timestamp, 25.0);
    assert_eq!(matches[0].name, "test2");
    assert!(!records.iter().any(|r| r.name == "test1"), "旧记录 test1 应被完全剔除");

    // 验证 duration 保护逻辑：当传入 duration <= 0 时，自动继承已有记录中的有效总时长
    let r3 = HistoryRecord {
        name: "test2".into(),
        path: p2.into(),
        timestamp: 30.0,
        duration: 0.0,
        last_played_at: 3000,
    };
    save_history_record(r3);
    let records_after = load_history();
    let updated = records_after.iter().find(|r| is_same_path(&r.path, p2)).unwrap();
    assert_eq!(updated.timestamp, 30.0);
    assert_eq!(updated.duration, 100.0, "当传入 duration <= 0 时，应自动继承已有记录中的有效时长 100.0");
}

#[test]
fn test_path_normalization_and_is_same_path() {
    use pulse_player_lib::commands::{is_same_path, normalize_path};

    assert!(is_same_path("C:\\foo\\bar.mp4", "c:/foo/bar.mp4"));
    assert!(is_same_path("C:/folder/movie.mkv ", "c:\\folder\\MOVIE.MKV"));
    assert!(!is_same_path("C:\\foo\\bar.mp4", "C:\\foo\\baz.mp4"));
    assert_eq!(normalize_path("  c:/videos/test.mp4  "), "c:\\videos\\test.mp4");
}

#[test]
fn test_mpv_end_file_reason_on_replace() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let video_path = manifest_dir.parent().unwrap().join("tests").join("fixtures").join("test_video.mp4");
        let video_str = video_path.to_str().unwrap();

        // 1. 加载第一个视频
        let cmd_name = CString::new("loadfile").unwrap();
        let file_arg = CString::new(video_str).unwrap();
        let args = [cmd_name.as_ptr(), file_arg.as_ptr(), std::ptr::null()];
        assert_eq!(mpv_command(raw, args.as_ptr()), 0);

        // 2. 等待第一个视频起播
        std::thread::sleep(std::time::Duration::from_millis(200));

        // 3. 用新视频 replace 当前视频，模拟用户在主页点击新视频切流
        let c_replace = CString::new("replace").unwrap();
        let replace_args = [cmd_name.as_ptr(), file_arg.as_ptr(), c_replace.as_ptr(), std::ptr::null()];
        assert_eq!(mpv_command(raw, replace_args.as_ptr()), 0);

        // 4. 轮询并捕获被卸载文件的 MPV_EVENT_END_FILE 事件
        let start = std::time::Instant::now();
        let mut got_stop_reason = false;
        while start.elapsed().as_secs() < 3 {
            let event = mpv_wait_event(raw, 0.1);
            if event.is_null() {
                continue;
            }
            let ev = &*event;
            if ev.event_id == MPV_EVENT_END_FILE {
                assert!(!ev.data.is_null());
                let end_file = &*(ev.data as *mut mpv_event_end_file);
                // 验证在 loadfile replace 时，前一个视频的 end-file 原因必须是 STOP (2) 而非 EOF (0)
                // 这正是防止误切集的核心断言！
                if end_file.reason == MPV_END_FILE_REASON_STOP {
                    got_stop_reason = true;
                    assert_ne!(end_file.reason, MPV_END_FILE_REASON_EOF, "loadfile replace 触发的事件严禁为 EOF");
                    break;
                }
            }
        }

        mpv_terminate_destroy(raw);
        assert!(got_stop_reason, "应收到 reason=STOP (2) 的 MPV_EVENT_END_FILE 事件");
    }
}

#[test]
fn test_mpv_natural_eof_reached_and_loop() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let keep_open = CString::new("keep-open").unwrap();
        let yes = CString::new("yes").unwrap();
        mpv_set_option_string(raw, keep_open.as_ptr(), yes.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        let eof_prop = CString::new("eof-reached").unwrap();
        mpv_observe_property(raw, 99, eof_prop.as_ptr(), MPV_FORMAT_FLAG);

        let dur_prop = CString::new("duration").unwrap();
        mpv_observe_property(raw, 101, dur_prop.as_ptr(), MPV_FORMAT_DOUBLE);

        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let video_path = manifest_dir.parent().unwrap().join("tests").join("fixtures").join("test_video.mp4");
        let video_str = video_path.to_str().unwrap();

        let cmd_name = CString::new("loadfile").unwrap();
        let file_arg = CString::new(video_str).unwrap();
        let args = [cmd_name.as_ptr(), file_arg.as_ptr(), std::ptr::null()];
        assert_eq!(mpv_command(raw, args.as_ptr()), 0);

        let start = std::time::Instant::now();
        let mut got_eof_reached_true = false;
        let mut loop_success = false;
        let mut seeked = false;

        while start.elapsed().as_secs() < 5 {
            let event = mpv_wait_event(raw, 0.1);
            if event.is_null() {
                continue;
            }
            let ev = &*event;
            if ev.event_id == MPV_EVENT_PROPERTY_CHANGE {
                let prop = &*(ev.data as *mut mpv_event_property);
                if !prop.name.is_null() && !prop.data.is_null() {
                    let name = CStr::from_ptr(prop.name).to_string_lossy();
                    if name == "eof-reached" {
                        let flag = *(prop.data as *mut c_int) != 0;
                        if flag {
                            got_eof_reached_true = true;
                            // 模拟单集循环 / 切换切流：跳转到开头并解除暂停
                            let seek_cmd = CString::new("seek").unwrap();
                            let seek_sec = CString::new("0").unwrap();
                            let seek_mode = CString::new("absolute").unwrap();
                            let seek_args = [seek_cmd.as_ptr(), seek_sec.as_ptr(), seek_mode.as_ptr(), std::ptr::null()];
                            mpv_command(raw, seek_args.as_ptr());

                            let pause_prop = CString::new("pause").unwrap();
                            let pause_val = CString::new("no").unwrap();
                            mpv_set_property_string(raw, pause_prop.as_ptr(), pause_val.as_ptr());
                        } else if got_eof_reached_true {
                            // 成功从 EOF 状态重置回播放态
                            loop_success = true;
                            break;
                        }
                    } else if name == "duration" {
                        let d = *(prop.data as *mut f64);
                        if !seeked && d > 0.0 {
                            seeked = true;
                            let target = (d - 0.15).max(0.0);
                            let seek_cmd = CString::new("seek").unwrap();
                            let seek_sec = CString::new(format!("{:.3}", target)).unwrap();
                            let seek_mode = CString::new("absolute").unwrap();
                            let seek_args = [seek_cmd.as_ptr(), seek_sec.as_ptr(), seek_mode.as_ptr(), std::ptr::null()];
                            mpv_command(raw, seek_args.as_ptr());
                        }
                    }
                }
            }
        }

        mpv_terminate_destroy(raw);
        assert!(got_eof_reached_true, "在 keep-open=yes 下播完必须能捕获到 eof-reached 为 true");
        assert!(loop_success, "执行重播或切流后 eof-reached 必须成功重置为 false");
    }
}

#[test]
fn test_history_delete_and_clear() {
    let _lock = HISTORY_TEST_LOCK.lock().unwrap();
    let _sandbox = HistoryTestSandbox::new("delete_clear");
    use pulse_player_lib::history::{clear_history, delete_history_record, load_history, save_history_record, HistoryRecord};

    // 先清空已有历史
    clear_history();
    assert!(load_history().is_empty(), "清空后历史记录应为空");

    // 添加 3 条测试记录
    let r1 = HistoryRecord {
        name: "video1.mp4".into(),
        path: "C:\\videos\\video1.mp4".into(),
        timestamp: 10.0,
        duration: 100.0,
        last_played_at: 1000,
    };
    let r2 = HistoryRecord {
        name: "video2.mp4".into(),
        path: "C:/videos/video2.mp4".into(),
        timestamp: 20.0,
        duration: 200.0,
        last_played_at: 2000,
    };
    let r3 = HistoryRecord {
        name: "video3.mp4".into(),
        path: "C:\\videos\\video3.mp4".into(),
        timestamp: 30.0,
        duration: 300.0,
        last_played_at: 3000,
    };

    save_history_record(r1);
    save_history_record(r2);
    save_history_record(r3);
    assert_eq!(load_history().len(), 3);

    // 删除单项 (使用正反斜杠不同与大小写不同的路径比对)
    delete_history_record("c:\\videos\\VIDEO2.mp4");
    let after_delete = load_history();
    assert_eq!(after_delete.len(), 2, "删除特定一项后应剩余 2 条");
    assert!(!after_delete.iter().any(|r| r.name == "video2.mp4"), "video2 应已被彻底删除");

    // 全部清空
    clear_history();
    assert!(load_history().is_empty(), "清空所有后历史列表应为 0 条");
}

#[test]
fn test_mpv_loadfile_unpauses() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        let pause_prop = CString::new("pause").unwrap();
        let yes_val = CString::new("yes").unwrap();
        let no_val = CString::new("no").unwrap();

        // 模拟先前播放器处于暂停状态
        mpv_set_property_string(raw, pause_prop.as_ptr(), yes_val.as_ptr());
        let pause_ptr = mpv_get_property_string(raw, pause_prop.as_ptr());
        assert_eq!(CStr::from_ptr(pause_ptr).to_string_lossy(), "yes");
        mpv_free(pause_ptr as *mut c_void);

        // 模拟 load_file 逻辑：加载文件后显式解除暂停
        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let video_path = manifest_dir.parent().unwrap().join("tests").join("fixtures").join("test_video.mp4");
        let video_str = video_path.to_str().unwrap();

        let cmd_name = CString::new("loadfile").unwrap();
        let file_arg = CString::new(video_str).unwrap();
        let args = [cmd_name.as_ptr(), file_arg.as_ptr(), std::ptr::null()];
        assert_eq!(mpv_command(raw, args.as_ptr()), 0);

        // 关键：解除暂停
        mpv_set_property_string(raw, pause_prop.as_ptr(), no_val.as_ptr());

        let pause_ptr_after = mpv_get_property_string(raw, pause_prop.as_ptr());
        assert_eq!(CStr::from_ptr(pause_ptr_after).to_string_lossy(), "no", "起播后 pause 属性应立即解除为 no");
        mpv_free(pause_ptr_after as *mut c_void);

        mpv_terminate_destroy(raw);
    }
}

#[test]
fn test_audio_format_extensions_supported() {
    use pulse_player_lib::playlist::SUPPORTED_EXTENSIONS;

    let required_audio = ["mp3", "flac", "wav", "m4a", "aac", "ogg", "ape", "opus", "wma"];
    for audio_ext in required_audio {
        assert!(
            SUPPORTED_EXTENSIONS.contains(&audio_ext),
            "SUPPORTED_EXTENSIONS 应包含纯音频格式: {}",
            audio_ext
        );
    }

    assert!(SUPPORTED_EXTENSIONS.contains(&"f4v"), "SUPPORTED_EXTENSIONS 应包含 f4v");
    assert!(SUPPORTED_EXTENSIONS.contains(&"ogv"), "SUPPORTED_EXTENSIONS 应包含 ogv");
}

#[test]
fn test_scan_folder_audio_files() {
    use pulse_player_lib::playlist::scan_folder_videos;

    let temp_dir = std::env::temp_dir().join(format!("pulse_player_audio_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let song1 = temp_dir.join("01. track_intro.mp3");
    let song2 = temp_dir.join("02. theme_song.flac");
    let video = temp_dir.join("03. episode_preview.mp4");
    let text = temp_dir.join("notes.txt");

    let _ = std::fs::write(&song1, b"dummy audio content");
    let _ = std::fs::write(&song2, b"dummy flac content");
    let _ = std::fs::write(&video, b"dummy mp4 content");
    let _ = std::fs::write(&text, b"dummy text content");

    let items = scan_folder_videos(song1.to_str().unwrap());

    // 应该包含 3 个媒体文件 (2 个音频 + 1 个视频)，txt 文件应被过滤
    assert_eq!(items.len(), 3, "扫描结果应包含 3 个音视频媒体项");
    assert_eq!(items[0].name, "01. track_intro.mp3");
    assert_eq!(items[1].name, "02. theme_song.flac");
    assert_eq!(items[2].name, "03. episode_preview.mp4");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_history_near_eof_timestamp_handling() {
    let _lock = HISTORY_TEST_LOCK.lock().unwrap();
    let _sandbox = HistoryTestSandbox::new("near_eof");
    use pulse_player_lib::history::{clear_history, load_history, save_history_record, HistoryRecord};

    clear_history();
    let r = HistoryRecord {
        name: "test_movie.mp4".into(),
        path: "C:\\videos\\test_movie.mp4".into(),
        timestamp: 0.0, // 播完归零
        duration: 3600.0,
        last_played_at: 5000,
    };
    save_history_record(r);

    let list = load_history();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].timestamp, 0.0, "播完归零的时间戳应成功持久化为 0.0");
    clear_history();
}

#[test]
fn test_history_schema_version_migration() {
    let _lock = HISTORY_TEST_LOCK.lock().unwrap();
    let sandbox = HistoryTestSandbox::new("schema_version");
    use pulse_player_lib::history::{load_history, save_history_record, HistoryData, HistoryRecord};

    // 1. 模拟写入旧版本无 version 字段的裸数组 JSON
    let old_raw_json = r#"[
        {
            "name": "legacy_video.mp4",
            "path": "C:\\videos\\legacy_video.mp4",
            "timestamp": 120.5,
            "duration": 600.0,
            "last_played_at": 1000
        }
    ]"#;
    let file_path = sandbox.dir.join("history.json");
    std::fs::write(&file_path, old_raw_json).unwrap();

    // 2. 调用 load_history() 读取，验证能成功解析旧格式并自愈升级
    let records = load_history();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, "legacy_video.mp4");
    assert_eq!(records[0].timestamp, 120.5);

    // 3. 验证本地文件已被自动迁移为带 version: 1 的 HistoryData 格式
    let new_content = std::fs::read_to_string(&file_path).unwrap();
    let parsed: HistoryData = serde_json::from_str(&new_content).expect("应迁移为合法的 HistoryData 结构");
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.records.len(), 1);

    // 4. 测试再次保存记录时，依旧保持带 version 结构
    let new_r = HistoryRecord {
        name: "new_video.mp4".into(),
        path: "C:\\videos\\new_video.mp4".into(),
        timestamp: 45.0,
        duration: 300.0,
        last_played_at: 2000,
    };
    save_history_record(new_r);
    let reloaded = load_history();
    assert_eq!(reloaded.len(), 2);
}

#[test]
fn test_video_margin_ratio_top() {
    let raw = unsafe { mpv_create() };
    assert!(!raw.is_null());

    unsafe {
        let osc = CString::new("osc").unwrap();
        let no = CString::new("no").unwrap();
        mpv_set_option_string(raw, osc.as_ptr(), no.as_ptr());

        let vo = CString::new("vo").unwrap();
        let vo_val = CString::new("null").unwrap();
        mpv_set_option_string(raw, vo.as_ptr(), vo_val.as_ptr());

        let res = mpv_initialize(raw);
        assert!(res >= 0);

        // 测试设置非全屏顶部边距比例
        let prop = CString::new("video-margin-ratio-top").unwrap();
        let ratio_val = CString::new("0.0556").unwrap();
        let ret = mpv_set_property_string(raw, prop.as_ptr(), ratio_val.as_ptr());
        assert_eq!(ret, 0, "设置 video-margin-ratio-top 应当成功");

        // 验证读取出的属性值
        let read_val_ptr = mpv_get_property_string(raw, prop.as_ptr());
        assert!(!read_val_ptr.is_null());
        let read_str = CStr::from_ptr(read_val_ptr).to_string_lossy();
        let val_f64: f64 = read_str.parse().unwrap();
        assert!((val_f64 - 0.0556).abs() < 0.001);
        mpv_free(read_val_ptr as *mut c_void);

        // 测试重置为全屏 (0.0)
        let zero_val = CString::new("0.0000").unwrap();
        let ret_zero = mpv_set_property_string(raw, prop.as_ptr(), zero_val.as_ptr());
        assert_eq!(ret_zero, 0);

        mpv_terminate_destroy(raw);
    }
}



