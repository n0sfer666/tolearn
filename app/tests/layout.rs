#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tolearn_app::ipc::layout::{Places, migrate, places};

fn home(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("tolearn-layout-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn old(root: &Path) -> PathBuf {
    let directory = root.join("Library/Application Support/dev.tolearn.app");
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, body).unwrap();
}

fn rooms(root: &Path) -> Places {
    let made = Places {
        config: root.join("config/tolearn"),
        data: root.join("data/tolearn"),
    };
    std::fs::create_dir_all(&made.config).unwrap();
    std::fs::create_dir_all(&made.data).unwrap();
    made
}

#[test]
fn настройки_лежат_в_конфиге_а_хранилище_в_данных() {
    let root = home("split");

    let made = places(&root);

    if std::env::var_os("XDG_CONFIG_HOME").is_none() {
        assert_eq!(made.config, root.join(".config/tolearn"));
    }
    if std::env::var_os("XDG_DATA_HOME").is_none() {
        assert_eq!(made.data, root.join(".local/share/tolearn"));
    }
    assert_ne!(made.config, made.data);
}

#[test]
fn переезд_разносит_старую_папку_по_двум_корзинам() {
    let root = home("move");
    let was = old(&root);
    file(&was.join("settings.yaml"), "locale: ru\n");
    file(&was.join("provider.yaml"), "kind: ollama\n");
    file(&was.join("registry.yaml"), "programs: []\n");
    file(&was.join("notes/local-runtime.md"), "конспект\n");
    file(&was.join("offline/index.yaml"), "urls: []\n");
    file(
        &was.join("unpacked/llm-agents-base/roadmap.yaml"),
        "id: x\n",
    );
    file(&was.join("history/llm-agents-base/1.yaml"), "at: вчера\n");
    file(&was.join("search-llm-agents-base.yaml"), "terms: []\n");
    let made = rooms(&root);

    migrate(&was, &made).unwrap();

    assert_eq!(
        std::fs::read_to_string(made.config.join("settings.yaml")).unwrap(),
        "locale: ru\n"
    );
    assert!(made.config.join("provider.yaml").is_file());
    assert!(made.config.join("registry.yaml").is_file());
    assert!(made.data.join("notes/local-runtime.md").is_file());
    assert!(made.data.join("offline/index.yaml").is_file());
    assert!(
        made.data
            .join("unpacked/llm-agents-base/roadmap.yaml")
            .is_file()
    );
    assert!(made.data.join("history/llm-agents-base/1.yaml").is_file());
    assert!(made.data.join("search-llm-agents-base.yaml").is_file());
    assert!(!was.join("settings.yaml").exists(), "старое не убрано");
    assert!(!was.join("notes").exists(), "старое не убрано");
}

#[test]
fn переезд_не_затирает_то_что_уже_на_новом_месте() {
    let root = home("keep");
    let was = old(&root);
    file(&was.join("settings.yaml"), "locale: en\n");
    let made = rooms(&root);
    file(&made.config.join("settings.yaml"), "locale: ru\n");

    migrate(&was, &made).unwrap();

    assert_eq!(
        std::fs::read_to_string(made.config.join("settings.yaml")).unwrap(),
        "locale: ru\n"
    );
    assert!(
        was.join("settings.yaml").is_file(),
        "старое стоило оставить"
    );
}

#[test]
fn переезда_без_старой_папки_нет_и_это_не_ошибка() {
    let root = home("fresh");
    let made = rooms(&root);

    migrate(
        &root.join("Library/Application Support/dev.tolearn.app"),
        &made,
    )
    .unwrap();

    assert!(!made.config.join("settings.yaml").exists());
}

#[test]
fn чужие_файлы_старой_папки_остаются_на_месте() {
    let root = home("alien");
    let was = old(&root);
    file(&was.join("crash.log"), "паника\n");
    file(&was.join("search.yaml"), "не индекс\n");
    let made = rooms(&root);

    migrate(&was, &made).unwrap();

    assert!(was.join("crash.log").is_file());
    assert!(was.join("search.yaml").is_file());
    assert!(!made.data.join("search.yaml").exists());
}
