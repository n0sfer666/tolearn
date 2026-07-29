#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_provider::{Remembered, Vault};

const PHRASE: &str = "длинная парольная фраза";
const TODAY: &str = "2026-07-29";

struct Case {
    context: Context,
    data: PathBuf,
    bundle: PathBuf,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!("tolearn-crypt-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    Case {
        context: Context::with_vaults(&data, Arc::new(Remembered::default()), vault),
        data,
        bundle: copied(&format!("crypt-{name}")),
    }
}

impl Case {
    fn roadmap(&self) -> String {
        self.bundle.display().to_string()
    }

    fn note(&self, topic: &str, body: &str) {
        let out = call(
            &self.context,
            "save_note",
            &json!({
                "bundle": self.roadmap(),
                "topic": topic,
                "body": body,
                "stamp": null,
                "directory": null,
            }),
        )
        .unwrap();
        assert_eq!(out["saved"], json!(true));
    }

    fn body(&self, topic: &str) -> String {
        call(
            &self.context,
            "note",
            &json!({ "bundle": self.roadmap(), "topic": topic, "directory": null }),
        )
        .unwrap()["body"]
            .as_str()
            .unwrap()
            .trim()
            .to_owned()
    }

    fn switch(&self, enable: Option<bool>, phrase: Option<&str>) -> Result<Value, IpcError> {
        call(
            &self.context,
            "encryption",
            &json!({ "enable": enable, "phrase": phrase }),
        )
    }

    fn topic(&self) -> String {
        let out = call(&self.context, "scan", &json!({ "bundle": self.roadmap() })).unwrap();
        out["topics"][0].as_str().unwrap().to_owned()
    }
}

fn plain(root: &std::path::Path) -> Vec<String> {
    let mut found = Vec::new();
    let Ok(listing) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in listing.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(plain(&path));
        } else {
            found.push(String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned());
        }
    }
    found
}

#[test]
fn включение_прячет_конспекты_и_чтение_через_приложение_работает() {
    let case = case("on");
    let topic = case.topic();
    case.note(&topic, "тайный абзац");

    let out = case.switch(Some(true), Some(PHRASE)).unwrap();

    assert_eq!(out["enabled"], json!(true));
    assert_eq!(case.body(&topic), "тайный абзац");
    for text in plain(&case.data.join("notes")) {
        assert!(!text.contains("тайный абзац"), "текст виден на диске");
    }
}

#[test]
fn выключение_возвращает_конспекты_в_открытый_вид() {
    let case = case("off");
    let topic = case.topic();
    case.note(&topic, "тайный абзац");
    case.switch(Some(true), Some(PHRASE)).unwrap();

    let out = case.switch(Some(false), None).unwrap();

    assert_eq!(out["enabled"], json!(false));
    assert_eq!(case.body(&topic), "тайный абзац");
    assert!(
        plain(&case.data.join("notes"))
            .iter()
            .any(|text| text.contains("тайный абзац")),
        "открытых конспектов на диске нет"
    );
}

#[test]
fn правка_в_запертом_хранилище_сохраняется() {
    let case = case("save");
    let topic = case.topic();
    case.note(&topic, "первый абзац");
    case.switch(Some(true), Some(PHRASE)).unwrap();

    case.note(&topic, "переписанный абзац");

    assert_eq!(case.body(&topic), "переписанный абзац");
}

#[test]
fn короткая_фраза_шифрование_не_включает() {
    let case = case("short");

    let refused = case.switch(Some(true), Some("три")).unwrap_err();

    assert_eq!(refused.code, "encryption.short-phrase");
    assert_eq!(case.switch(None, None).unwrap()["enabled"], json!(false));
}

#[test]
fn внешний_каталог_и_шифрование_взаимоисключающи() {
    let case = case("external");
    let outside = case.data.join("снаружи");
    let stored = call(&case.context, "settings", &json!({ "save": null })).unwrap();
    let mut settings = stored.clone();
    settings["notes_directory"] = json!(outside.display().to_string());
    call(&case.context, "settings", &json!({ "save": settings })).unwrap();

    let refused = case.switch(Some(true), Some(PHRASE)).unwrap_err();
    assert_eq!(refused.code, "encryption.external");

    call(&case.context, "settings", &json!({ "save": stored })).unwrap();
    case.switch(Some(true), Some(PHRASE)).unwrap();
    let mut back = call(&case.context, "settings", &json!({ "save": null })).unwrap();
    back["notes_directory"] = json!(outside.display().to_string());
    let denied = call(&case.context, "settings", &json!({ "save": back })).unwrap_err();
    assert_eq!(denied.code, "settings.sealed");
}

#[test]
fn поиск_по_запертым_конспектам_находит_и_индекс_на_диске_закрыт() {
    let case = case("search");
    let topic = case.topic();
    case.note(&topic, "мнемоника про кэш");
    case.switch(Some(true), Some(PHRASE)).unwrap();

    let out = call(
        &case.context,
        "search",
        &json!({ "bundle": case.roadmap(), "query": "мнемоника", "directory": null, "limit": 5 }),
    )
    .unwrap();

    let hits = out["hits"].as_array().unwrap();
    assert!(
        hits.iter().any(|hit| hit["kind"] == json!("note")),
        "конспект не найден: {hits:?}"
    );
    for text in plain(&case.data.join("notes")) {
        assert!(!text.contains("мнемоника"), "индекс виден на диске");
    }
}

#[test]
fn индекс_поиска_ложится_в_хранилище_рядом_с_конспектами() {
    let case = case("blob");
    let topic = case.topic();
    case.note(&topic, "мнемоника про кэш");
    case.switch(Some(true), Some(PHRASE)).unwrap();

    call(
        &case.context,
        "search",
        &json!({ "bundle": case.roadmap(), "query": "мнемоника", "directory": null, "limit": 5 }),
    )
    .unwrap();

    let blobs = std::fs::read_dir(case.data.join("notes").join("blobs"))
        .map(|listing| listing.flatten().count())
        .unwrap_or_default();
    assert!(blobs > 0, "индекс не сохранён в хранилище");
}

#[test]
fn внешний_каталог_читается_открытым_даже_с_ключом_рядом() {
    let case = case("outside");
    let topic = case.topic();
    let outside = case.data.join("снаружи");
    let where_to = outside.display().to_string();
    call(
        &case.context,
        "save_note",
        &json!({
            "bundle": case.roadmap(),
            "topic": topic,
            "body": "внешний абзац",
            "stamp": null,
            "directory": where_to,
        }),
    )
    .unwrap();
    std::fs::write(outside.join("identity.age"), "чужой ключ").unwrap();

    let out = call(
        &case.context,
        "note",
        &json!({ "bundle": case.roadmap(), "topic": topic, "directory": where_to }),
    )
    .unwrap();

    assert_eq!(out["body"].as_str().unwrap().trim(), "внешний абзац");
}

#[test]
fn прерванное_переключение_чинится_при_обращении_к_шифрованию() {
    let case = case("settle");
    let topic = case.topic();
    case.note(&topic, "тайный абзац");
    let notes = case.data.join("notes");
    std::fs::rename(&notes, case.data.join("notes.retired")).unwrap();

    let out = case.switch(None, None).unwrap();

    assert_eq!(out["enabled"], json!(false));
    assert!(!case.data.join("notes.retired").exists());
    assert_eq!(case.body(&topic), "тайный абзац");
}

#[test]
fn экспорт_из_запертого_хранилища_предупреждает_об_открытом_тексте() {
    let case = case("export");
    let topic = case.topic();
    case.note(&topic, "тайный абзац");
    case.switch(Some(true), Some(PHRASE)).unwrap();

    let out = call(
        &case.context,
        "export",
        &json!({
            "bundle": case.roadmap(),
            "today": TODAY,
            "path": case.data.join("программа.md").display().to_string(),
            "directory": null,
        }),
    )
    .unwrap();

    assert_eq!(out["plaintext"], json!(true));
    let text = std::fs::read_to_string(out["path"].as_str().unwrap()).unwrap();
    assert!(text.contains("тайный абзац"), "конспект не попал в экспорт");
}

#[test]
fn без_ключа_устройства_парольная_фраза_расшифровывает() {
    let case = case("phrase");
    let topic = case.topic();
    case.note(&topic, "тайный абзац");
    case.switch(Some(true), Some(PHRASE)).unwrap();
    case.context.keys().forget().unwrap();

    let lost = call(
        &case.context,
        "note",
        &json!({ "bundle": case.roadmap(), "topic": topic, "directory": null }),
    )
    .unwrap_err();
    assert_eq!(lost.code, "notes.locked");

    case.switch(Some(false), Some(PHRASE)).unwrap();
    assert_eq!(case.body(&topic), "тайный абзац");
}
