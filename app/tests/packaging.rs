#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "manifest gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use serde_json::Value;

fn root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name)
}

fn read(name: &str) -> String {
    let path = root(name);
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(reason) => panic!("{}: {reason}", path.display()),
    }
}

fn config() -> Value {
    serde_json::from_str(&read("app/tauri.conf.json")).unwrap()
}

#[test]
fn схема_объявлена_установщику() {
    let schemes = config()["plugins"]["deep-link"]["desktop"]["schemes"].clone();

    assert_eq!(
        schemes,
        serde_json::json!(["tolearn"]),
        "установщик не узнает про `tolearn://`: схемы нет в plugins > deep-link > desktop"
    );
}

#[test]
fn упаковка_объявлена_под_три_ос() {
    let targets = config()["bundle"]["targets"].clone();

    for target in ["app", "dmg", "msi", "deb"] {
        assert!(
            targets
                .as_array()
                .unwrap()
                .iter()
                .any(|listed| listed == target),
            "в целях упаковки нет `{target}`"
        );
    }
}

#[test]
fn иконки_лежат_под_каждый_установщик() {
    let icons = config()["bundle"]["icon"].clone();

    for icon in ["icons/icon.ico", "icons/icon.icns", "icons/32x32.png"] {
        assert!(
            icons
                .as_array()
                .unwrap()
                .iter()
                .any(|listed| listed == icon),
            "в списке иконок нет `{icon}`"
        );
        assert!(
            root("app").join(icon).exists(),
            "иконка `{icon}` объявлена, но её нет на диске"
        );
    }
}

#[test]
fn потолки_веса_совпадают_с_бюджетами() {
    let budgets = read("docs/architecture.md");
    let gate = read("scripts/weigh.sh");

    for (row, call) in [
        ("| Установщик, macOS arm64 | ≤ 12 МБ |", "weigh dmg 12"),
        ("| Установщик, Windows x64 | ≤ 14 МБ |", "weigh msi 14"),
        ("| Установщик, Linux x64 | ≤ 14 МБ |", "weigh deb 14"),
    ] {
        assert!(budgets.contains(row), "в бюджетах пропала строка `{row}`");
        assert!(
            gate.contains(call),
            "гейт веса разошёлся с бюджетом: нет `{call}`"
        );
    }
}
