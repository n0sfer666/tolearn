#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "settings gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tolearn_core::settings::{Locale, Settings, Theme};

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("tolearn-settings-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn kept(directory: &Path, settings: &Settings) -> Settings {
    let file = directory.join("settings.yaml");
    settings.save(&file).unwrap();
    Settings::read(&file).unwrap()
}

fn written(directory: &Path, body: &str) -> PathBuf {
    let file = directory.join("settings.yaml");
    std::fs::write(&file, body).unwrap();
    file
}

#[test]
fn отсутствующий_файл_читается_как_значения_по_умолчанию() {
    let directory = scratch("absent");

    let settings = Settings::read(&directory.join("settings.yaml")).unwrap();

    assert_eq!(settings, Settings::default());
    assert_eq!(settings.locale, Locale::Ru);
    assert_eq!(settings.theme, Theme::System);
}

#[test]
fn настройки_переживают_запись_и_чтение() {
    let directory = scratch("round");
    let settings = Settings {
        disk_budget_mb: 512,
        locale: Locale::En,
        theme: Theme::Dark,
    };

    assert_eq!(kept(&directory, &settings), settings);
}

#[test]
fn бюджет_диска_переводится_в_байты() {
    let settings = Settings {
        disk_budget_mb: 3,
        ..Settings::default()
    };

    assert_eq!(settings.budget_bytes(), 3 * 1024 * 1024);
}

#[test]
fn неизвестная_тема_не_читается_молча() {
    let directory = scratch("theme");
    let file = written(
        &directory,
        "schema: tolearn/settings/v1\n\
         disk_budget_mb: 128\n\
         locale: ru\n\
         theme: неон\n",
    );

    let failure = Settings::read(&file).unwrap_err().to_string();

    assert!(failure.contains("theme"), "{failure}");
}

#[test]
fn неизвестный_язык_не_читается_молча() {
    let directory = scratch("locale");
    let file = written(
        &directory,
        "schema: tolearn/settings/v1\n\
         disk_budget_mb: 128\n\
         locale: fr\n\
         theme: system\n",
    );

    let failure = Settings::read(&file).unwrap_err().to_string();

    assert!(failure.contains("locale"), "{failure}");
}

#[test]
fn нулевой_бюджет_не_читается_молча() {
    let directory = scratch("budget");
    let file = written(
        &directory,
        "schema: tolearn/settings/v1\n\
         disk_budget_mb: 0\n\
         locale: ru\n\
         theme: system\n",
    );

    assert!(Settings::read(&file).is_err());
}

#[test]
fn поля_истории_из_v1_чтению_не_мешают_и_больше_не_пишутся() {
    let directory = scratch("history");
    let file = written(
        &directory,
        "schema: tolearn/settings/v1\n\
         disk_budget_mb: 128\n\
         locale: ru\n\
         theme: system\n\
         history_depth: 3\n\
         history_share_percent: 101\n",
    );

    let settings = Settings::read(&file).unwrap();
    settings.save(&file).unwrap();

    assert_eq!(settings.disk_budget_mb, 128);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(!text.contains("history"), "{text}");
}
