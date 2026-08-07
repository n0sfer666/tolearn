#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::repository;
use support::speaking::{Speaking, breaking, speaking};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_provider::{Remembered, Vault};

static CASES: AtomicUsize = AtomicUsize::new(0);

const TODAY: &str = "2026-08-06";
const BROKEN: &str = "```yaml\nschema: learning-roadmap/v1\nid: [\n```\n";
const SLOW: Duration = Duration::from_secs(3);

struct Case {
    context: Context,
    data: PathBuf,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!(
        "tolearn-generate-{name}-{}-{}",
        std::process::id(),
        CASES.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    Case {
        context: Context::with_vault(&data, vault),
        data,
    }
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repository().join(path)).unwrap()
}

fn skeleton() -> String {
    let map =
        read("fixtures/valid/roadmap/minimal.yaml").replace("generated: true", "generated: false");
    let progress = read("fixtures/valid/progress/minimal.yaml");
    format!("```yaml\n{map}```\n\nи прогресс:\n\n```yaml\n{progress}```\n")
}

fn topic() -> String {
    format!(
        "```yaml\n{}```\n",
        read("fixtures/valid/topic/minimal-topic.yaml")
    )
}

fn two() -> (String, String) {
    let map = format!(
        "{}  - id: second-topic\n    title: Вторая тема\n    stage: 1\n    \
         file: topics/second-topic.yaml\n    est_hours: [1, 2]\n    priority: core\n",
        read("fixtures/valid/roadmap/minimal.yaml").replace("generated: true", "generated: false")
    );
    let progress = format!(
        "{}  second-topic:\n    status: todo\n    attempts: []\n    passed_at: null\n    \
         next_review_at: null\n    gaps: []\n",
        read("fixtures/valid/progress/minimal.yaml")
    );
    (map, progress)
}

fn paired() -> String {
    let (map, progress) = two();
    format!("```yaml\n{map}```\n\nи прогресс:\n\n```yaml\n{progress}```\n")
}

fn looped(case: &Case) {
    let (map, progress) = two();
    let root = case.data.join("draft");
    std::fs::create_dir_all(root.join("topics")).unwrap();
    std::fs::write(root.join("roadmap.yaml"), map).unwrap();
    std::fs::write(root.join("progress.yaml"), progress).unwrap();
    std::fs::write(
        root.join("topics/minimal-topic.yaml"),
        written(
            "minimal-topic",
            "Минимальная тема",
            "depends_on:\n  - second-topic",
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("topics/second-topic.yaml"),
        written(
            "second-topic",
            "Вторая тема",
            "depends_on:\n  - minimal-topic",
        ),
    )
    .unwrap();
}

fn written(id: &str, title: &str, depends: &str) -> String {
    read("fixtures/valid/topic/minimal-topic.yaml")
        .replace("id: minimal-topic", &format!("id: {id}"))
        .replace("title: Минимальная тема", &format!("title: {title}"))
        .replace("depends_on: []", depends)
}

fn answered(id: &str, title: &str, depends: &str) -> String {
    format!("```yaml\n{}```\n", written(id, title, depends))
}

fn enable(case: &Case, endpoint: &str) {
    call(
        &case.context,
        "provider",
        &json!({
            "save": {
                "enabled": true,
                "active": "local",
                "local": {
                    "endpoint": endpoint,
                    "api": "ollama",
                    "model": "llama3:8b",
                    "num_ctx": 0,
                    "temperature_tenths": 7,
                },
                "remote": {
                    "endpoint": endpoint,
                    "api": "openai",
                    "model": "gpt",
                    "num_ctx": 0,
                    "temperature_tenths": 7,
                },
                "harness": {
                    "id": "claude",
                    "command": "claude",
                    "args": ["-p"],
                    "timeout_secs": 180,
                },
            },
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap();
}

fn started(case: &Case, heard: &Speaking) -> String {
    enable(case, &heard.endpoint);
    let out = call(
        &case.context,
        "generate",
        &json!({
            "subject": "поднять локальную модель",
            "level": "basics",
            "weekly_hours": 6,
            "weeks": 10,
        }),
    )
    .unwrap();
    out["job"].as_str().unwrap().to_owned()
}

fn until(case: &Case, job: &str, ready: impl Fn(&Value) -> bool) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let live = state(case, job);
        if ready(&live) {
            return live;
        }
        assert!(Instant::now() < deadline, "генерация застряла: {live}");
        sleep(Duration::from_millis(20));
    }
}

fn state(case: &Case, job: &str) -> Value {
    call(&case.context, "generate_state", &json!({ "job": job })).unwrap()
}

fn accept(case: &Case, job: &str) -> Result<Value, IpcError> {
    call(
        &case.context,
        "generate_accept",
        &json!({ "job": job, "today": TODAY }),
    )
}

fn draft(case: &Case, take: bool) -> Value {
    call(
        &case.context,
        "generate_draft",
        &json!({ "take": take, "drop": false }),
    )
    .unwrap()
}

fn bundles(case: &Case) -> PathBuf {
    case.data.join("unpacked")
}

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

    let waiting = until(&case, &job, |live| live["waiting"] == json!(true));
    assert_eq!(waiting["total"], json!(1));
    assert_eq!(waiting["step"], json!("confirm"));
    assert!(!bundles(&case).join("minimal-program").exists());

    call(&case.context, "generate_go", &json!({ "job": job })).unwrap();
    let done = until(&case, &job, |live| live["finished"] == json!(true));
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

    until(&case, &job, |live| live["waiting"] == json!(true));
    call(&case.context, "generate_go", &json!({ "job": job })).unwrap();
    let done = until(&case, &job, |live| live["finished"] == json!(true));

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

    until(&case, &job, |live| live["waiting"] == json!(true));
    call(&case.context, "generate_go", &json!({ "job": job })).unwrap();
    let done = until(&case, &job, |live| live["finished"] == json!(true));

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["done"], json!(1));
    assert_eq!(heard.heard().len(), 3, "оборванный запрос не повторили");
}

#[test]
fn три_обрыва_подряд_останавливают_генерацию() {
    let case = case("broken-dead");
    let heard = breaking(|_, _| None);
    let job = started(&case, &heard);

    let done = until(&case, &job, |live| live["finished"] == json!(true));

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

    until(&case, &job, |live| live["waiting"] == json!(true));
    call(&case.context, "generate_go", &json!({ "job": job })).unwrap();
    let stuck = until(&case, &job, |live| {
        !live["missed"].as_array().unwrap().is_empty()
    });

    assert_eq!(stuck["finished"], json!(false));
    assert_eq!(stuck["done"], json!(0));
    assert_eq!(stuck["step"], json!("missed"));

    call(&case.context, "generate_go", &json!({ "job": job })).unwrap();
    let done = until(&case, &job, |live| live["finished"] == json!(true));

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["missed"], json!([]));
    assert_eq!(done["done"], json!(1));
    assert_eq!(heard.heard().len(), 5);
    assert_eq!(accept(&case, &job).unwrap()["ok"], json!(true));
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
    until(&case, &job, |live| live["waiting"] == json!(true));
    call(&case.context, "generate_go", &json!({ "job": job })).unwrap();
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
    let done = until(&case, &next, |live| live["finished"] == json!(true));

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["done"], json!(1));
    assert_eq!(second.heard().len(), 1, "скелет спросили заново");
    assert_eq!(accept(&case, &next).unwrap()["ok"], json!(true));
    assert_eq!(draft(&case, false)["draft"], json!(null));
}

#[test]
fn три_круга_брака_останавливают_генерацию_без_файлов() {
    let case = case("stuck");
    let heard = speaking(|_, _| BROKEN.to_owned());
    let job = started(&case, &heard);

    let done = until(&case, &job, |live| live["finished"] == json!(true));

    assert_eq!(done["cancelled"], json!(false));
    assert!(!done["refused"].as_array().unwrap().is_empty(), "{done}");
    assert_eq!(heard.heard().len(), 3);
    assert!(accept(&case, &job).is_err());
    assert!(!bundles(&case).join("minimal-program").exists());
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

    until(&case, &job, |live| live["waiting"] == json!(true));
    call(&case.context, "generate_stop", &json!({ "job": job })).unwrap();
    let done = until(&case, &job, |live| live["finished"] == json!(true));

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
    until(&case, &job, |live| live["finished"] == json!(true));
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

    until(&case, &job, |live| live["waiting"] == json!(true));
    call(&case.context, "generate_go", &json!({ "job": job })).unwrap();
    let done = until(&case, &job, |live| live["finished"] == json!(true));

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
    let done = until(&case, &job, |live| live["finished"] == json!(true));

    assert_eq!(done["refused"], json!([]));
    assert_eq!(done["missed"], json!([]));
    assert_eq!(done["done"], json!(2));
    assert_eq!(heard.heard().len(), 2, "переспросили лишнее");
    assert_eq!(accept(&case, &job).unwrap()["ok"], json!(true));
}
