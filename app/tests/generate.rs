#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::thread::sleep;
use std::time::Instant;

use serde_json::json;
use support::answers::{BROKEN, skeleton, topic};
use support::generating::{
    SLOW, TODAY, accept, bundles, case, draft, enable, finished, go, started, until, waiting,
};
use support::speaking::speaking;
use tolearn_app::ipc::call;

#[test]
fn программа_собирается_двумя_шагами_и_ложится_на_диск_только_по_согласию() {
    let case = case("whole");
    let heard = speaking(|prompt, _| {
        if prompt.contains("## Тема") {
            topic()
        } else {
            skeleton()
        }
    });
    let job = started(&case, &heard);

    let waiting = waiting(&case, &job);
    assert_eq!(waiting["total"], json!(1));
    assert_eq!(waiting["step"], json!("confirm"));
    assert!(!bundles(&case).join("minimal-program").exists());

    go(&case, &job);
    let done = finished(&case, &job);
    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["done"], json!(1));
    assert_eq!(done["summary"]["id"], json!("minimal-program"));
    assert_eq!(done["summary"]["stages"][0]["topics"], json!(1));
    assert_eq!(done["tokens"], json!(36));

    let imported = accept(&case, &job).unwrap();
    assert_eq!(imported["ok"], json!(true));
    let home = bundles(&case).join("minimal-program");
    assert!(home.join("examiner.md").exists());
    assert!(home.join("topics/minimal-topic.yaml").exists());
    assert!(
        std::fs::read_to_string(home.join("roadmap.yaml"))
            .unwrap()
            .contains("generated: true")
    );
    let programs = call(&case.context, "programs", &json!({ "today": TODAY })).unwrap();
    assert_eq!(programs["programs"][0]["id"], json!("minimal-program"));
}

#[test]
fn выдуманная_моделью_дата_сборки_заменяется_на_сегодняшнюю() {
    let case = case("stamp");
    let heard = speaking(|prompt, _| {
        if prompt.contains("## Тема") {
            topic()
        } else {
            skeleton().replace("generated_at: 2026-07-27", "generated_at: 2024-01-15")
        }
    });
    let job = started(&case, &heard);

    waiting(&case, &job);
    go(&case, &job);
    finished(&case, &job);
    accept(&case, &job).unwrap();

    let map = std::fs::read_to_string(bundles(&case).join("minimal-program").join("roadmap.yaml"))
        .unwrap();
    assert!(map.contains(&format!("generated_at: {TODAY}")), "{map}");
    assert!(!map.contains("2024-01-15"), "{map}");
    let asked = heard.heard();
    assert!(
        asked[0].contains(&format!("- Сегодня: {TODAY}")),
        "{asked:?}"
    );
}

#[test]
fn черновик_переживает_обрыв_и_сборка_идёт_с_места_остановки() {
    let case = case("draft");
    let first = speaking(|_, turn| {
        if turn == 0 {
            skeleton()
        } else {
            BROKEN.to_owned()
        }
    });
    let job = started(&case, &first);
    waiting(&case, &job);
    go(&case, &job);
    until(&case, &job, |live| {
        !live["missed"].as_array().unwrap().is_empty()
    });

    let kept = draft(&case, false);
    assert_eq!(kept["draft"]["id"], json!("minimal-program"));
    assert_eq!(kept["draft"]["total"], json!(1));
    assert_eq!(kept["draft"]["done"], json!(0));

    let second = speaking(|_, _| topic());
    enable(&case, &second.endpoint);
    let taken = draft(&case, true);
    let next = taken["job"].as_str().unwrap().to_owned();
    let done = finished(&case, &next);

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["done"], json!(1));
    assert_eq!(second.heard().len(), 1, "скелет спросили заново");
    assert_eq!(accept(&case, &next).unwrap()["ok"], json!(true));
    assert_eq!(draft(&case, false)["draft"], json!(null));
}

#[test]
fn отмена_на_сводке_не_оставляет_ни_файла() {
    let case = case("stop");
    let heard = speaking(|prompt, _| {
        if prompt.contains("## Тема") {
            topic()
        } else {
            skeleton()
        }
    });
    let job = started(&case, &heard);

    waiting(&case, &job);
    call(&case.context, "generate_stop", &json!({ "job": job })).unwrap();
    let done = finished(&case, &job);

    assert_eq!(done["cancelled"], json!(true));
    assert_eq!(heard.heard().len(), 1);
    assert!(!bundles(&case).join("minimal-program").exists());
}

#[test]
fn секунды_идут_пока_модель_ещё_молчит() {
    let case = case("ticking");
    let heard = speaking(|prompt, _| {
        sleep(SLOW);
        if prompt.contains("## Тема") {
            topic()
        } else {
            skeleton()
        }
    });
    let asked = Instant::now();
    let job = started(&case, &heard);

    let live = until(&case, &job, |live| {
        live["seconds"].as_u64().unwrap_or(0) >= 1
    });

    assert!(asked.elapsed() < SLOW, "счётчик дождался ответа: {live}");
    assert_eq!(live["finished"], json!(false));
    call(&case.context, "generate_stop", &json!({ "job": job })).unwrap();
    finished(&case, &job);
}
