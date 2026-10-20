use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_void};
use std::path::PathBuf;
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EnumChildWindows,
    PostQuitMessage, RegisterClassW, TranslateMessage, MSG, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};

#[repr(C)]
struct mpv_handle {
    _unused: [u8; 0],
}

#[link(name = "mpv")]
extern "C" {
    fn mpv_create() -> *mut mpv_handle;
    fn mpv_initialize(ctx: *mut mpv_handle) -> c_int;
    fn mpv_command(ctx: *mut mpv_handle, args: *const *const c_char) -> c_int;
    fn mpv_set_option_string(ctx: *mut mpv_handle, name: *const c_char, data: *const c_char) -> c_int;
    fn mpv_wait_event(ctx: *mut mpv_handle, timeout: c_double) -> *mut c_void;
    fn mpv_terminate_destroy(ctx: *mut mpv_handle);
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        windows_sys::Win32::UI::WindowsAndMessaging::WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "system" fn enum_child(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    let mut class_buf = [0u16; 256];
    let len = windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, class_buf.as_mut_ptr(), 256);
    let class_str = String::from_utf16_lossy(&class_buf[..len as usize]);
    println!("Found child window: HWND {:?}, class: {}", hwnd, class_str);
    1
}

#[test]
fn test_mpv_child_window() {
    unsafe {
        let class_name: Vec<u16> = "MpvChildTest\0".encode_utf16().collect();
        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as _,
            hIcon: 0 as _,
            hCursor: 0 as _,
            hbrBackground: 0 as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wnd_class);

        let window_title: Vec<u16> = "MPV Child Test\0".encode_utf16().collect();
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            100,
            100,
            640,
            480,
            0 as _,
            0 as _,
            0 as _,
            std::ptr::null(),
        );

        let raw = mpv_create();
        let wid_str = format!("{}", hwnd as isize);
        let c_wid = CString::new("wid").unwrap();
        let c_wid_val = CString::new(wid_str).unwrap();
        mpv_set_option_string(raw, c_wid.as_ptr(), c_wid_val.as_ptr());

        let c_osc = CString::new("osc").unwrap();
        let c_no = CString::new("no").unwrap();
        mpv_set_option_string(raw, c_osc.as_ptr(), c_no.as_ptr());

        let init_res = mpv_initialize(raw);
        assert!(init_res >= 0);

        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let video_path = manifest_dir.parent().unwrap().join("tests").join("fixtures").join("test_video.mp4");
        let video_str = video_path.to_str().unwrap();

        let c_cmd = CString::new("loadfile").unwrap();
        let c_path = CString::new(video_str).unwrap();
        let args = [c_cmd.as_ptr(), c_path.as_ptr(), std::ptr::null()];
        mpv_command(raw, args.as_ptr());

        let start = std::time::Instant::now();
        while start.elapsed().as_millis() < 500 {
            let mut msg: MSG = std::mem::zeroed();
            while windows_sys::Win32::UI::WindowsAndMessaging::PeekMessageW(
                &mut msg,
                0 as _,
                0,
                0,
                windows_sys::Win32::UI::WindowsAndMessaging::PM_REMOVE,
            ) != 0
            {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            mpv_wait_event(raw, 0.05);
        }

        println!("Enumerating child windows of HWND {:?}:", hwnd);
        EnumChildWindows(hwnd, Some(enum_child), 0);

        mpv_terminate_destroy(raw);
        DestroyWindow(hwnd);
    }
}
