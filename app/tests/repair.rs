#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::json;
use support::answers::{BROKEN, answered, looped, paired, skeleton, topic};
use support::generating::{
    accept, bundles, case, draft, enable, finished, go, started, until, waiting,
};
use support::speaking::{breaking, speaking};

#[test]
fn брак_возвращается_модели_с_перечнем_нарушений() {
    let case = case("repair");
    let heard = speaking(|prompt, turn| {
        if prompt.contains("## Тема") {
            topic()
        } else if turn == 0 {
            BROKEN.to_owned()
        } else {
            skeleton()
        }
    });
    let job = started(&case, &heard);

    waiting(&case, &job);
    go(&case, &job);
    let done = finished(&case, &job);

    assert_eq!(done["refused"], json!([]));
    let mended = &heard.heard()[1];
    assert!(mended.contains("## Что в нём не так"), "{mended}");
    assert!(mended.contains("## Прошлый ответ"), "{mended}");
}

#[test]
fn оборванный_запрос_повторяется_и_сборка_доходит_до_конца() {
    let case = case("broken-link");
    let heard = breaking(|prompt, turn| match turn {
        0 => None,
        _ if prompt.contains("## Тема") => Some(topic()),
        _ => Some(skeleton()),
    });
    let job = started(&case, &heard);

    waiting(&case, &job);
    go(&case, &job);
    let done = finished(&case, &job);

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["done"], json!(1));
    assert_eq!(heard.heard().len(), 3, "оборванный запрос не повторили");
}

#[test]
fn три_обрыва_подряд_останавливают_генерацию() {
    let case = case("broken-dead");
    let heard = breaking(|_, _| None);
    let job = started(&case, &heard);

    let done = finished(&case, &job);

    assert_eq!(done["cancelled"], json!(false));
    assert!(!done["refused"].as_array().unwrap().is_empty(), "{done}");
    assert_eq!(heard.heard().len(), 3);
}

#[test]
fn несобранная_тема_ждёт_повтора_и_не_рушит_задание() {
    let case = case("missed");
    let heard = speaking(|_, turn| match turn {
        0 => skeleton(),
        1..=3 => BROKEN.to_owned(),
        _ => topic(),
    });
    let job = started(&case, &heard);

    waiting(&case, &job);
    go(&case, &job);
    let stuck = until(&case, &job, |live| {
        !live["missed"].as_array().unwrap().is_empty()
    });

    assert_eq!(stuck["finished"], json!(false));
    assert_eq!(stuck["done"], json!(0));
    assert_eq!(stuck["step"], json!("missed"));

    go(&case, &job);
    let done = finished(&case, &job);

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["missed"], json!([]));
    assert_eq!(done["done"], json!(1));
    assert_eq!(heard.heard().len(), 5);
    assert_eq!(accept(&case, &job).unwrap()["ok"], json!(true));
}

#[test]
fn три_круга_брака_останавливают_генерацию_без_файлов() {
    let case = case("stuck");
    let heard = speaking(|_, _| BROKEN.to_owned());
    let job = started(&case, &heard);

    let done = finished(&case, &job);

    assert_eq!(done["cancelled"], json!(false));
    assert!(!done["refused"].as_array().unwrap().is_empty(), "{done}");
    assert_eq!(heard.heard().len(), 3);
    assert!(accept(&case, &job).is_err());
    assert!(!bundles(&case).join("minimal-program").exists());
}

#[test]
fn зависимость_вперёд_не_доходит_до_черновика_и_тема_переспрашивается() {
    let case = case("forward");
    let heard = speaking(|prompt, _| {
        if !prompt.contains("## Тема") {
            return paired();
        }
        if prompt.contains("- `id`: second-topic") {
            return answered(
                "second-topic",
                "Вторая тема",
                "depends_on:\n  - minimal-topic",
            );
        }
        if prompt.contains("## Прошлый ответ") {
            return answered("minimal-topic", "Минимальная тема", "depends_on: []");
        }
        answered(
            "minimal-topic",
            "Минимальная тема",
            "depends_on:\n  - second-topic",
        )
    });
    let job = started(&case, &heard);

    waiting(&case, &job);
    go(&case, &job);
    let done = finished(&case, &job);

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["missed"], json!([]));
    assert_eq!(done["done"], json!(2));
    let told = heard.heard();
    assert!(
        told.iter()
            .any(|said| said.contains("bundle.forward-dependency")),
        "{told:?}"
    );
    assert_eq!(accept(&case, &job).unwrap()["ok"], json!(true));
}

#[test]
fn кольцо_в_черновике_чинится_кругом_починки_без_человека() {
    let case = case("cycle");
    looped(&case);
    let heard = speaking(|prompt, _| {
        if prompt.contains("- `id`: second-topic") {
            return answered(
                "second-topic",
                "Вторая тема",
                "depends_on:\n  - minimal-topic",
            );
        }
        answered("minimal-topic", "Минимальная тема", "depends_on: []")
    });
    enable(&case, &heard.endpoint);

    let taken = draft(&case, true);
    let job = taken["job"].as_str().unwrap().to_owned();
    let done = finished(&case, &job);

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["missed"], json!([]));
    assert_eq!(done["done"], json!(2));
    assert_eq!(heard.heard().len(), 2, "переспросили лишнее");
    assert_eq!(accept(&case, &job).unwrap()["ok"], json!(true));
}
