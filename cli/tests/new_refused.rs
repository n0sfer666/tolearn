#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

mod generating;

use generating::answers::{flat, told};
use generating::{Desk, LEVEL, REQUEST, provided};
use tolearn_cli::CliError;
use tolearn_core::library::Library;

#[test]
fn без_сети_new_отказывает_с_причиной_и_модель_не_зовёт() {
    let model = told(flat());
    let mut desk = Desk::new(&model);
    desk.up = false;

    let error = desk
        .run(&["new", REQUEST, "--level", LEVEL, "--out", desk.out()])
        .unwrap_err()
        .to_string();

    assert!(error.contains("generate.offline"), "{error}");
    assert!(model.heard().is_empty());
    assert!(Library::at(&desk.out).list().unwrap().is_empty());
}

#[test]
fn без_уровня_или_папки_new_отказывает_как_ошибка_использования() {
    let model = told(flat());
    let desk = Desk::new(&model);
    let out = desk.out();

    for (args, what) in [
        (vec!["new", REQUEST, "--out", out], "уровень"),
        (
            vec!["new", REQUEST, "--level", "  ", "--out", out],
            "уровень",
        ),
        (vec!["new", REQUEST, "--level", LEVEL], "каталог"),
        (vec!["new", "--level", LEVEL, "--out", out], "запрос"),
    ] {
        let error = desk.run(&args).unwrap_err();

        assert!(matches!(error, CliError::Usage(_)), "{args:?}: {error}");
        assert!(error.to_string().contains(what), "{args:?}: {error}");
    }
    assert!(model.heard().is_empty());
}

#[test]
fn флаги_new_проверяются_до_работы() {
    let model = told(flat());
    let desk = Desk::new(&model);
    let out = desk.out();

    for (args, told) in [
        (
            vec![
                "new", REQUEST, "--level", LEVEL, "--out", out, "--choice", "1",
            ],
            "неизвестный флаг `--choice`",
        ),
        (
            vec!["new", REQUEST, "--lvl", LEVEL, "--out", out],
            "неизвестный флаг `--lvl`",
        ),
        (
            vec!["new", REQUEST, "--out", out, "--level"],
            "флаг `--level` без значения",
        ),
        (
            vec![
                "new", REQUEST, "--level", LEVEL, "--level", LEVEL, "--out", out,
            ],
            "флаг `--level` назван дважды",
        ),
        (
            vec!["new", "Хочу", "чиптюн", "--level", LEVEL, "--out", out],
            "в кавычках",
        ),
    ] {
        let error = desk.run(&args).unwrap_err();

        assert!(matches!(error, CliError::Usage(_)), "{args:?}: {error}");
        assert!(error.to_string().contains(told), "{args:?}: {error}");
    }
    assert!(model.heard().is_empty());
    assert!(!desk.out.exists());
}

#[test]
fn папку_с_программой_new_не_трогает() {
    let model = told(flat());
    let desk = Desk::new(&model);
    desk.begun();
    let before = desk.program();

    let error = desk
        .run(&["new", REQUEST, "--level", LEVEL, "--out", desk.out()])
        .unwrap_err()
        .to_string();

    assert!(error.contains("уже лежит программа"), "{error}");
    assert!(error.contains("tolearn next"), "{error}");
    assert_eq!(model.heard().len(), 3);
    assert_eq!(desk.program(), before);
}

#[test]
fn сбой_посреди_генерации_не_оставляет_программы_а_повтор_проходит() {
    let answers = flat();
    let broken = told(vec![
        answers[0].clone(),
        answers[1].clone(),
        "не этап".to_owned(),
    ]);
    let desk = Desk::new(&broken);

    let error = desk
        .run(&["new", REQUEST, "--level", LEVEL, "--out", desk.out()])
        .unwrap_err();

    assert!(matches!(error, CliError::Generate(_)), "{error}");
    assert!(Library::at(&desk.out).list().unwrap().is_empty());

    let model = told(flat());
    provided(&model.endpoint)
        .save(&desk.config.join("provider.yaml"))
        .unwrap();
    desk.begun();

    assert!(desk.path().is_dir());
}

#[test]
fn в_хранилище_приложения_new_не_пишет() {
    let model = told(flat());
    let desk = Desk::new(&model);
    let inside = desk.data().join("мой");
    let around = desk.root.join("out/../data/мой");

    for out in [&inside, &around] {
        let error = desk
            .run(&[
                "new",
                REQUEST,
                "--level",
                LEVEL,
                "--out",
                out.to_str().unwrap(),
            ])
            .unwrap_err();

        assert!(matches!(error, CliError::Data(_)), "{out:?}: {error}");
        assert!(
            error.to_string().contains("хранилище приложения"),
            "{error}"
        );
    }
    assert!(model.heard().is_empty());
    assert!(!desk.data().exists());
}
