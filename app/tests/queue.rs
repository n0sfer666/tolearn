#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::{copied, repository};
use tolearn_app::ipc::{Context, call};

const TODAY: &str = "2026-07-28";

fn data(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("tolearn-queue-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn dated(name: &str) -> PathBuf {
    let bundle = copied(name);
    let queued = repository().join("fixtures/valid/progress/review-queue.yaml");
    std::fs::copy(queued, bundle.join("progress.yaml")).unwrap();
    bundle
}

fn renamed(bundle: &Path, id: &str, title: &str) {
    edit(
        &bundle.join("roadmap.yaml"),
        "id: llm-agents-base",
        &format!("id: {id}"),
    );
    edit(
        &bundle.join("roadmap.yaml"),
        "title: Работа с агентами LLM — база на готовых инструментах",
        &format!("title: {title}"),
    );
    edit(
        &bundle.join("progress.yaml"),
        "roadmap_id: llm-agents-base",
        &format!("roadmap_id: {id}"),
    );
}

fn edit(file: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(file).unwrap();
    assert!(text.contains(from), "нечего менять в {}", file.display());
    std::fs::write(file, text.replacen(from, to, 1)).unwrap();
}

fn import(context: &Context, bundle: &Path) {
    let imported = call(
        context,
        "import",
        &json!({ "path": bundle.display().to_string(), "today": TODAY }),
    )
    .unwrap();
    assert_eq!(imported["ok"], json!(true), "{imported}");
}

fn queue(context: &Context) -> Vec<Value> {
    call(context, "queue", &json!({ "today": TODAY }))
        .unwrap()
        .get("due")
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

fn pairs(due: &[Value]) -> Vec<(String, String)> {
    due.iter()
        .map(|item| {
            (
                item["program"].as_str().unwrap().to_owned(),
                item["topic"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn очередь_собирается_поперёк_всех_программ() {
    let root = data("across");
    let context = Context::new(&root);
    let first = dated("across-a");
    let second = dated("across-b");
    renamed(&second, "llm-agents-extra", "Продолжение про агентов");
    import(&context, &first);
    import(&context, &second);

    let due = queue(&context);

    let seen = pairs(&due);
    assert!(
        seen.contains(&(
            "llm-agents-base".to_owned(),
            "tokens-context-cost".to_owned()
        )),
        "{seen:?}"
    );
    assert!(
        seen.contains(&(
            "llm-agents-extra".to_owned(),
            "tokens-context-cost".to_owned()
        )),
        "{seen:?}"
    );
    let named = due
        .iter()
        .find(|item| item["program"] == json!("llm-agents-extra"))
        .unwrap();
    assert_eq!(named["title"], json!("Продолжение про агентов"));
    assert!(named["topic_title"].as_str().unwrap().len() > 5, "{named}");
    assert_eq!(
        named["bundle"].as_str().unwrap(),
        second.display().to_string()
    );
}

#[test]
fn просроченное_отделено_от_сегодняшнего() {
    let root = data("overdue");
    let context = Context::new(&root);
    let bundle = dated("overdue");
    import(&context, &bundle);

    let due = queue(&context);

    let overdue: Vec<&str> = due
        .iter()
        .filter(|item| item["overdue"] == json!(true))
        .map(|item| item["topic"].as_str().unwrap())
        .collect();
    let today: Vec<&str> = due
        .iter()
        .filter(|item| item["overdue"] == json!(false))
        .map(|item| item["topic"].as_str().unwrap())
        .collect();
    assert_eq!(overdue, vec!["tokens-context-cost", "local-runtime"]);
    assert_eq!(today, vec!["openai-compatible-api"]);
}

#[test]
fn повторение_двигает_дату_и_убирает_тему_из_очереди() {
    let root = data("repeat");
    let context = Context::new(&root);
    let bundle = dated("repeat");
    import(&context, &bundle);

    let repeated = call(
        &context,
        "repeat",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "local-runtime",
            "today": TODAY,
        }),
    )
    .unwrap();

    assert_eq!(repeated["next_review_at"], json!("2026-10-26"));
    let left = pairs(&queue(&context));
    assert!(
        !left.iter().any(|(_, topic)| topic == "local-runtime"),
        "{left:?}"
    );
}

#[test]
fn повторение_не_трогает_ни_статус_ни_попытки() {
    let root = data("keeps");
    let context = Context::new(&root);
    let bundle = dated("keeps");
    import(&context, &bundle);
    let before = std::fs::read_to_string(bundle.join("progress.yaml")).unwrap();

    call(
        &context,
        "repeat",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "tokens-context-cost",
            "today": TODAY,
        }),
    )
    .unwrap();

    let after = std::fs::read_to_string(bundle.join("progress.yaml")).unwrap();
    assert_eq!(
        after.replacen("2026-10-26", "2026-07-09", 1),
        before,
        "изменилась не только дата"
    );
}

#[test]
fn недостижимая_программа_очередь_не_валит() {
    let root = data("gone");
    let context = Context::new(&root);
    let kept = dated("gone-kept");
    let lost = dated("gone-lost");
    renamed(&lost, "llm-agents-lost", "Пропавшая программа");
    import(&context, &kept);
    import(&context, &lost);
    std::fs::remove_dir_all(&lost).unwrap();

    let due = queue(&context);

    assert!(
        due.iter()
            .all(|item| item["program"] == json!("llm-agents-base")),
        "{due:?}"
    );
    assert!(!due.is_empty(), "уцелевшая программа пропала из очереди");
}

#[test]
fn неизвестная_тема_в_повторении_отвергается() {
    let root = data("stranger");
    let context = Context::new(&root);
    let bundle = dated("stranger");
    import(&context, &bundle);

    let refused = call(
        &context,
        "repeat",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "нет-такой",
            "today": TODAY,
        }),
    )
    .unwrap_err();

    assert_eq!(refused.code, "topic.unknown");
}

#[test]
fn кривая_дата_в_повторении_отвергается() {
    let root = data("date");
    let context = Context::new(&root);
    let bundle = dated("date");
    import(&context, &bundle);

    let refused = call(
        &context,
        "repeat",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "local-runtime",
            "today": "вчера",
        }),
    )
    .unwrap_err();

    assert_eq!(refused.code, "date.malformed");
}
