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

fn product(config: &str) -> String {
    let parsed: Value = serde_json::from_str(&read(config)).unwrap();
    parsed["productName"].as_str().unwrap().to_owned()
}

#[test]
fn обе_цели_зовут_существующие_скрипты() {
    let makefile = read("Makefile");

    for (target, script) in [
        ("dev:", "scripts/check.sh"),
        ("install:", "scripts/install-macos.sh"),
    ] {
        assert!(makefile.contains(target), "в Makefile нет цели `{target}`");
        assert!(
            makefile.contains(script),
            "цель `{target}` не зовёт `{script}`"
        );
        assert!(root(script).exists(), "скрипта `{script}` нет на диске");
    }
}

#[test]
fn набор_проверок_не_продублирован_в_скрипте() {
    let gate = read("scripts/check.sh");
    let listed: Value = serde_json::from_str(&read(".context/checks.json")).unwrap();

    assert!(
        gate.contains(".context/checks.json"),
        "`make dev` не читает список проверок и разойдётся с хуком"
    );
    for command in listed.as_object().unwrap().values() {
        let command = command.as_str().unwrap();
        assert!(
            !gate.contains(command),
            "проверка `{command}` вписана в скрипт вторым экземпляром"
        );
    }
}

#[test]
fn установка_знает_оба_продукта_по_их_конфигам() {
    let script = read("scripts/install-macos.sh");

    for config in ["app/tauri.conf.json", "app/tauri.with-speech.conf.json"] {
        let name = product(config);
        assert!(
            script.contains(&format!("product={name}")),
            "установщик не знает продукт `{name}` из `{config}`"
        );
    }
    assert!(
        script.contains("hdiutil attach"),
        "установка идёт мимо `.dmg`"
    );
}

#[test]
fn цели_описаны_в_документации() {
    for (page, mention) in [
        (".context/checks.md", "make dev"),
        (".context/checks.md", "make install"),
        ("docs/ru/install/source.md", "make install"),
        ("docs/en/install/source.md", "make install"),
        ("CLAUDE.md", "make dev"),
    ] {
        assert!(
            read(page).contains(mention),
            "`{mention}` не описан в `{page}`"
        );
    }
}
