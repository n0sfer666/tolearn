#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "manifest gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

fn app(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(name)
}

fn read(name: &str) -> String {
    let path = app(name);
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(reason) => panic!("{}: {reason}", path.display()),
    }
}

#[test]
fn причина_доступа_к_микрофону_объявлена() {
    let plist = read("Info.plist");

    assert!(
        plist.contains("NSMicrophoneUsageDescription"),
        "в Info.plist нет причины доступа к микрофону"
    );
    assert!(
        plist.contains("не отправляется"),
        "причина не говорит, что речь остаётся на этом компьютере"
    );
}

#[test]
fn право_на_вход_звука_выдано_и_подключено() {
    let entitlements = read("entitlements.plist");
    let config = read("tauri.conf.json");

    assert!(
        entitlements.contains("com.apple.security.device.audio-input"),
        "в entitlements.plist нет права на вход звука"
    );
    assert!(
        config.contains("\"entitlements\": \"entitlements.plist\""),
        "tauri.conf.json не подключает entitlements.plist"
    );
}
