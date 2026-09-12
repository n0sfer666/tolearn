#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

mod generating;

use generating::Desk;
use generating::answers::{forking, told};
use tolearn_cli::CliError;

#[test]
fn выбор_до_открытой_развилки_отказывает_без_сети_и_модели() {
    let model = told(forking());
    let mut desk = Desk::new(&model);
    desk.begun();
    desk.up = false;

    let error = desk
        .run(&["next", desk.out(), "--choice", "1"])
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("развилка после этапа «tracker» не открыта"),
        "{error}"
    );
    assert_eq!(model.heard().len(), 3);
}

#[test]
fn номер_вне_развилки_отказывает_без_сети_и_модели() {
    let model = told(forking());
    let mut desk = Desk::new(&model);
    desk.begun();
    desk.run(&["next", desk.out()]).unwrap();
    desk.up = false;

    let error = desk
        .run(&["next", desk.out(), "--choice", "5"])
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("варианта №5 нет: в развилке их 2"),
        "{error}"
    );
    assert_eq!(model.heard().len(), 4);
    assert_eq!(
        desk.program().stages.keys().collect::<Vec<_>>(),
        ["tracker"]
    );
}

#[test]
fn без_сети_новую_развилку_не_открыть() {
    let model = told(forking());
    let mut desk = Desk::new(&model);
    desk.begun();
    desk.up = false;

    let error = desk.run(&["next", desk.out()]).unwrap_err().to_string();

    assert!(error.contains("generate.offline"), "{error}");
    assert_eq!(model.heard().len(), 3);
}

#[test]
fn next_без_программы_отказывает() {
    let model = told(forking());
    let desk = Desk::new(&model);

    let error = desk.run(&["next", desk.out()]).unwrap_err().to_string();

    assert!(error.contains("нет программы"), "{error}");
    assert!(error.contains("tolearn new"), "{error}");
    assert!(model.heard().is_empty());
}

#[test]
fn номер_и_флаги_next_проверяются_до_работы() {
    let model = told(forking());
    let desk = Desk::new(&model);
    let out = desk.out();

    for (args, told) in [
        (vec!["next", out, "--choice", "0"], "`--choice`"),
        (vec!["next", out, "--choice", "два"], "`--choice`"),
        (
            vec!["next", out, "--level", "x"],
            "неизвестный флаг `--level`",
        ),
        (vec!["next"], "каталог"),
    ] {
        let error = desk.run(&args).unwrap_err();

        assert!(matches!(error, CliError::Usage(_)), "{args:?}: {error}");
        assert!(error.to_string().contains(told), "{args:?}: {error}");
    }
    assert!(model.heard().is_empty());
}

#[test]
fn в_хранилище_приложения_next_не_ходит() {
    let model = told(forking());
    let desk = Desk::new(&model);

    let error = desk
        .run(&["next", desk.data().to_str().unwrap()])
        .unwrap_err();

    assert!(matches!(error, CliError::Data(_)), "{error}");
    assert!(
        error.to_string().contains("хранилище приложения"),
        "{error}"
    );
    assert!(model.heard().is_empty());
}
