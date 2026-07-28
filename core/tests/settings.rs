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
    assert_eq!(settings.notes_directory, None);
}

#[test]
fn настройки_переживают_запись_и_чтение() {
    let directory = scratch("round");
    let settings = Settings {
        disk_budget_mb: 512,
        notes_directory: Some(PathBuf::from("/данные/конспекты")),
        locale: Locale::En,
        theme: Theme::Dark,
    };

    assert_eq!(kept(&directory, &settings), settings);
}

#[test]
fn свой_каталог_конспектов_пишется_пустым_значением() {
    let directory = scratch("own");

    let settings = kept(&directory, &Settings::default());

    let body = std::fs::read_to_string(directory.join("settings.yaml")).unwrap();
    assert!(body.contains("notes_directory: null"), "{body}");
    assert_eq!(settings.notes_directory, None);
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
         notes_directory: null\n\
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
         notes_directory: null\n\
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
         notes_directory: null\n\
         locale: ru\n\
         theme: system\n",
    );

    assert!(Settings::read(&file).is_err());
}

#[test]
fn путь_с_кавычками_переживает_запись() {
    let directory = scratch("quoted");
    let settings = Settings {
        notes_directory: Some(PathBuf::from("/данные/\"мои\" конспекты")),
        ..Settings::default()
    };

    assert_eq!(kept(&directory, &settings), settings);
}
