use std::path::PathBuf;

fn main() {
    // 获取项目根目录下的 libs/mpv 目录并添加至 rustc 静态/导入库搜索路径
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let mpv_lib_dir = manifest_dir.parent().unwrap().join("libs").join("mpv");
    
    if mpv_lib_dir.exists() {
        println!("cargo:rustc-link-search=native={}", mpv_lib_dir.display());
        println!("cargo:rustc-link-lib=mpv");

        // 自动将 libmpv-2.dll 拷贝到 target 目录下，保证本地调试与运行开箱即用
        let src_dll = mpv_lib_dir.join("libmpv-2.dll");
        if src_dll.exists() {
            if let Ok(out_dir) = std::env::var("OUT_DIR") {
                // OUT_DIR 通常形如 target/debug/build/pulse-player-.../out
                let out_path = PathBuf::from(out_dir);
                if let Some(target_dir) = out_path.ancestors().nth(3) {
                    let dest_dll = target_dir.join("libmpv-2.dll");
                    let _ = std::fs::copy(&src_dll, &dest_dll);
                }
            }
        }
    }

    tauri_build::build()
}
