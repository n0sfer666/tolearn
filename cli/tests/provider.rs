#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

mod generating;

use generating::answers::{flat, told};
use generating::{Desk, LEVEL, REQUEST};
use tolearn_provider::{Kind, Provider};

#[test]
fn без_включённого_провайдера_new_отказывает_до_сети_и_модели() {
    let model = told(flat());
    let desk = Desk::new(&model);
    std::fs::remove_file(desk.config.join("provider.yaml")).unwrap();

    let error = desk
        .run(&["new", REQUEST, "--level", LEVEL, "--out", desk.out()])
        .unwrap_err()
        .to_string();

    assert!(error.contains("провайдер выключен"), "{error}");
    assert!(error.contains("--provider"), "{error}");
    assert!(model.heard().is_empty());
}

#[test]
fn названный_файл_провайдера_должен_существовать() {
    let model = told(flat());
    let desk = Desk::new(&model);
    let missing = desk.root.join("nowhere.yaml");

    let error = desk
        .run(&[
            "new",
            REQUEST,
            "--level",
            LEVEL,
            "--out",
            desk.out(),
            "--provider",
            missing.to_str().unwrap(),
        ])
        .unwrap_err()
        .to_string();

    assert!(error.contains(missing.to_str().unwrap()), "{error}");
    assert!(model.heard().is_empty());
}

#[test]
fn выключенный_провайдер_в_названном_файле_отказывает_по_файлу() {
    let model = told(flat());
    let desk = Desk::new(&model);
    let named = desk.root.join("off.yaml");
    Provider::default().save(&named).unwrap();

    let error = desk
        .run(&[
            "new",
            REQUEST,
            "--level",
            LEVEL,
            "--out",
            desk.out(),
            "--provider",
            named.to_str().unwrap(),
        ])
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("провайдер выключен (`enabled: false`)"),
        "{error}"
    );
    assert!(error.contains(named.to_str().unwrap()), "{error}");
    assert!(!error.contains("настройках приложения"), "{error}");
    assert!(model.heard().is_empty());
}

#[test]
fn удалённому_провайдеру_без_ключа_new_отказывает_до_модели() {
    let model = told(flat());
    let mut desk = Desk::new(&model);
    desk.locked = true;
    Provider {
        enabled: true,
        active: Kind::Remote,
        ..Provider::default()
    }
    .save(&desk.config.join("provider.yaml"))
    .unwrap();

    let error = desk
        .run(&["new", REQUEST, "--level", LEVEL, "--out", desk.out()])
        .unwrap_err()
        .to_string();

    assert!(error.contains("ключ провайдера не прочитан"), "{error}");
    assert!(model.heard().is_empty());
}

#[test]
fn локальной_модели_недоступное_хранилище_ключей_не_мешает() {
    let model = told(flat());
    let mut desk = Desk::new(&model);
    desk.locked = true;

    desk.begun();

    assert_eq!(model.heard().len(), 3);
}
