// src-tauri/tests/test_updater.rs
// 验证 Tauri 2 原生静默更新配置、权限模型及 latest.json 清单规范

use serde_json::Value;

#[test]
fn test_tauri_conf_updater_configuration() {
    let conf_str = include_str!("../tauri.conf.json");
    let conf: Value = serde_json::from_str(conf_str).expect("tauri.conf.json 必须是合法的 JSON 格式");

    // 1. 验证生成更新产物配置
    assert_eq!(
        conf["bundle"]["createUpdaterArtifacts"],
        true,
        "bundle.createUpdaterArtifacts 必须为 true"
    );

    // 2. 验证 updater 插件配置
    let updater = &conf["plugins"]["updater"];
    assert!(
        updater.is_object(),
        "tauri.conf.json 必须配置 plugins.updater 对象"
    );

    // 3. 验证 Minisign 公钥
    let pubkey = updater["pubkey"].as_str().expect("updater.pubkey 必须为字符串");
    assert!(
        pubkey.starts_with("dW50cnVzdGVkIGNvbW1lbnQ"),
        "Minisign 公钥格式必须以 base64 untrusted comment 开头"
    );

    // 4. 验证更新检查端点
    let endpoints = updater["endpoints"]
        .as_array()
        .expect("updater.endpoints 必须为数组");
    assert!(!endpoints.is_empty(), "updater.endpoints 不能为空");
    let endpoint_url = endpoints[0].as_str().unwrap();
    assert!(
        endpoint_url.ends_with("/latest.json"),
        "更新端点必须指向规范 latest.json 清单"
    );

    // 5. 验证 Windows 下静默/被动更新模式
    assert_eq!(
        updater["windows"]["installMode"],
        "passive",
        "Windows 下必须配置 installMode = passive 以支持原地覆盖更新与进度展示"
    );
}

#[test]
fn test_capabilities_updater_and_process_permissions() {
    let cap_str = include_str!("../capabilities/default.json");
    let cap: Value = serde_json::from_str(cap_str).expect("capabilities/default.json 必须是合法的 JSON");

    let perms: Vec<&str> = cap["permissions"]
        .as_array()
        .expect("permissions 必须为数组")
        .iter()
        .filter_map(|p| p.as_str())
        .collect();

    assert!(
        perms.contains(&"updater:default"),
        "主窗口 Capability 必须授予 updater:default 权限以允许前端执行 check 与 download"
    );
    assert!(
        perms.contains(&"process:default"),
        "主窗口 Capability 必须授予 process:default 权限以允许前端在更新完成后调用 relaunch"
    );
}

#[test]
fn test_updater_manifest_schema_compliance() {
    // 模拟符合 Tauri 2 规范的标准 latest.json 清单
    let manifest_sample = r#"{
        "version": "0.3.0",
        "notes": "v0.3.0 更新测试：全新内核升级与 Bug 修复",
        "pub_date": "2026-10-21T18:00:00Z",
        "platforms": {
            "windows-x86_64": {
                "signature": "dW50cnVzdGVkIGNvbW1lbnQ6...",
                "url": "https://github.com/gavincrossv1/Pulse-Player/releases/download/v0.3.0/Pulse-Player_0.3.0_x64-setup.nsis.zip"
            }
        }
    }"#;

    let v: Value = serde_json::from_str(manifest_sample).expect("样本清单反序列化失败");

    // 验证核心字段
    assert!(v.get("version").is_some());
    assert!(v.get("notes").is_some());
    assert!(v.get("pub_date").is_some());

    let platforms = v.get("platforms").and_then(|p| p.as_object()).expect("platforms 必须是对象");
    assert!(
        platforms.contains_key("windows-x86_64"),
        "platforms 必须包含 windows-x86_64 平台条目"
    );

    let win_platform = &platforms["windows-x86_64"];
    assert!(win_platform.get("signature").is_some());
    assert!(win_platform.get("url").is_some());
}

#[test]
fn test_version_consistency() {
    let conf_str = include_str!("../tauri.conf.json");
    let conf: Value = serde_json::from_str(conf_str).expect("tauri.conf.json 格式错误");
    assert_eq!(conf["version"], "0.3.2", "tauri.conf.json 版本应为 0.3.2");
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.3.2", "Cargo.toml 版本应为 0.3.2");
}
