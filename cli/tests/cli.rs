#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

mod support;

use support::{code, copied, dual, json, stderr, stdout, tolearn};

#[test]
fn the_reference_bundle_passes_validation() {
    let bundle = copied("reference");

    let out = tolearn(&["validate", bundle.to_str().unwrap()]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("нарушений нет"), "{}", stdout(&out));
}

#[test]
fn a_bundle_with_a_violation_fails_and_names_it() {
    let bundle = copied("violation");
    std::fs::remove_file(bundle.join("topics/cp-gateway.yaml")).unwrap();

    let out = tolearn(&["validate", bundle.to_str().unwrap()]);

    assert_ne!(code(&out), 0);
    assert!(
        stdout(&out).contains("bundle.missing-topic-file"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn validation_in_json_lists_the_violations_by_code() {
    let bundle = copied("violation-json");
    std::fs::remove_file(bundle.join("topics/cp-gateway.yaml")).unwrap();

    let out = tolearn(&["validate", bundle.to_str().unwrap(), "--json"]);

    assert_ne!(code(&out), 0);
    let report = json(&out);
    assert_eq!(report["ok"], false);
    assert_eq!(report["violations"][0]["code"], "bundle.missing-topic-file");
}

#[test]
fn a_scan_of_the_reference_bundle_reports_every_topic() {
    let bundle = copied("scan");

    let out = tolearn(&["scan", bundle.to_str().unwrap(), "--json"]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let report = json(&out);
    assert_eq!(report["topics"].as_array().unwrap().len(), 7);
    assert!(report["broken"].as_array().unwrap().is_empty());
    assert!(
        report["absent"]
            .as_array()
            .unwrap()
            .iter()
            .all(|absent| absent["generated"] == false),
        "тема сгенерированного этапа потерялась"
    );
}

#[test]
fn a_topic_that_will_not_parse_is_scanned_as_broken_not_as_a_failure() {
    let bundle = copied("broken-topic");
    std::fs::write(bundle.join("topics/cp-gateway.yaml"), "schema: [\n").unwrap();

    let out = tolearn(&["scan", bundle.to_str().unwrap(), "--json"]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(json(&out)["broken"][0]["id"], "cp-gateway");
}

#[test]
fn a_directory_that_is_not_a_bundle_is_an_error_with_a_reason() {
    let bundle = copied("not-a-bundle");
    std::fs::remove_file(bundle.join("roadmap.yaml")).unwrap();

    let out = tolearn(&["scan", bundle.to_str().unwrap()]);

    assert_ne!(code(&out), 0);
    assert!(!stderr(&out).is_empty(), "молчаливый отказ");
}

#[test]
fn a_bundle_that_lies_in_both_formats_at_once_is_refused_rather_than_guessed() {
    let bundle = dual("dual");

    let out = tolearn(&["scan", bundle.to_str().unwrap()]);

    assert_ne!(code(&out), 0);
    assert!(stderr(&out).contains("roadmap.json"), "{}", stderr(&out));
}

#[test]
fn progress_of_a_fresh_bundle_is_nothing_done_out_of_everything() {
    let bundle = copied("progress-json");

    let out = tolearn(&["progress", bundle.to_str().unwrap(), "--json"]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let report = json(&out);
    assert_eq!(report["program"]["done"], 0);
    assert!(report["program"]["total"].as_u64().unwrap() > 0);
    assert_eq!(report["stages"].as_array().unwrap().len(), 3);
}

#[test]
fn progress_in_words_names_the_stages() {
    let bundle = copied("progress-words");

    let out = tolearn(&["progress", bundle.to_str().unwrap()]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("Этап 1"), "{}", stdout(&out));
}

#[test]
fn exam_prints_the_prompt_of_the_topic_it_was_asked_for() {
    let bundle = copied("exam");

    let out = tolearn(&["exam", bundle.to_str().unwrap(), "local-runtime"]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let prompt = stdout(&out);
    assert!(prompt.contains("Локальный рантайм"), "{prompt}");
    assert!(
        !prompt.contains("Копируется в бандл"),
        "служебная шапка шаблона попала в промпт"
    );
}

#[test]
fn exam_of_a_topic_the_roadmap_does_not_know_is_an_error() {
    let bundle = copied("exam-unknown");

    let out = tolearn(&["exam", bundle.to_str().unwrap(), "no-such-topic"]);

    assert_ne!(code(&out), 0);
    assert!(stderr(&out).contains("no-such-topic"), "{}", stderr(&out));
}

#[test]
fn a_verdict_applied_by_exam_lands_in_the_progress_file() {
    let bundle = copied("verdict");
    let verdict = bundle.join("verdict.json");
    std::fs::write(&verdict, passing_verdict()).unwrap();

    let out = tolearn(&[
        "exam",
        bundle.to_str().unwrap(),
        "local-runtime",
        "--verdict",
        verdict.to_str().unwrap(),
    ]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let saved = std::fs::read_to_string(bundle.join("progress.yaml")).unwrap();
    assert!(saved.contains("status: passed"), "{saved}");
}

#[test]
fn a_verdict_for_another_topic_is_refused_and_the_file_is_left_alone() {
    let bundle = copied("wrong-topic");
    let verdict = bundle.join("verdict.json");
    std::fs::write(&verdict, passing_verdict()).unwrap();
    let before = std::fs::read_to_string(bundle.join("progress.yaml")).unwrap();

    let out = tolearn(&[
        "exam",
        bundle.to_str().unwrap(),
        "cp-gateway",
        "--verdict",
        verdict.to_str().unwrap(),
    ]);

    assert_ne!(code(&out), 0);
    assert_eq!(
        std::fs::read_to_string(bundle.join("progress.yaml")).unwrap(),
        before,
        "отвергнутый вердикт всё-таки тронул файл"
    );
}

#[test]
fn merge_after_a_regeneration_reports_what_it_did() {
    let bundle = copied("merge");
    let was = copied("merge-was");

    let out = tolearn(&[
        "merge",
        bundle.to_str().unwrap(),
        "--was",
        was.to_str().unwrap(),
        "--json",
    ]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let report = json(&out);
    assert!(report["stale"].as_array().unwrap().is_empty());
    assert_eq!(report["orphaned"].as_array().unwrap().len(), 0);
}

#[test]
fn a_command_nobody_knows_is_a_usage_error_not_a_panic() {
    let bundle = copied("unknown-command");

    let out = tolearn(&["fly", bundle.to_str().unwrap()]);

    assert_ne!(code(&out), 0);
    assert!(
        stderr(&out).contains("неизвестная команда `fly`"),
        "{}",
        stderr(&out)
    );
    assert!(!stderr(&out).contains("panicked"), "{}", stderr(&out));
}

#[test]
fn exam_does_not_run_the_checks_of_the_bundle_unless_it_is_told_to() {
    let bundle = tripwired("no-checks");

    let out = tolearn(&["exam", bundle.to_str().unwrap(), "local-runtime"]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !bundle.join("tripwire").exists(),
        "check-команда бандла выполнилась сама"
    );
}

#[test]
fn exam_asked_to_run_the_checks_runs_them_and_reports_each_one() {
    let bundle = tripwired("checks");

    let out = tolearn(&[
        "exam",
        bundle.to_str().unwrap(),
        "local-runtime",
        "--run-checks",
        "--json",
    ]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        bundle.join("tripwire").exists(),
        "check-команда не запущена"
    );
    let checks = json(&out)["checks"].as_array().unwrap().clone();
    assert!(!checks.is_empty());
    assert_eq!(checks[0]["passed"], true);
}

fn tripwired(name: &str) -> std::path::PathBuf {
    let bundle = copied(name);
    let topic = bundle.join("topics/local-runtime.yaml");
    let text = std::fs::read_to_string(&topic).unwrap();
    let mut out = String::new();
    let mut skipping = false;
    for line in text.lines() {
        if let Some(indent) = line.find("check: |") {
            out.push_str(&" ".repeat(indent));
            out.push_str("check: touch tripwire && echo OK\n");
            skipping = true;
            continue;
        }
        if skipping {
            if line.trim().is_empty() || line.starts_with(&" ".repeat(9)) {
                continue;
            }
            skipping = false;
        }
        out.push_str(line);
        out.push('\n');
    }
    std::fs::write(&topic, out).unwrap();
    bundle
}

fn passing_verdict() -> &'static str {
    r#"{
      "topic_id": "local-runtime",
      "verdict": "pass",
      "date": "2026-07-27",
      "per_question": [
        {"id": "q1", "result": "ok"},
        {"id": "q2", "result": "ok"},
        {"id": "q3", "result": "ok"},
        {"id": "q4", "result": "ok"}
      ]
    }"#
}

#[test]
fn an_export_writes_one_markdown_file() {
    let bundle = copied("export");
    let out_file = bundle.parent().unwrap().join("program.md");

    let out = tolearn(&[
        "export",
        bundle.to_str().unwrap(),
        "--out",
        out_file.to_str().unwrap(),
        "--today",
        "2026-07-29",
    ]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = std::fs::read_to_string(&out_file).unwrap();
    assert!(text.starts_with("# "), "{}", &text[..40.min(text.len())]);
    assert!(text.contains("## Оглавление"));
}

#[test]
fn an_export_into_the_bundle_is_refused() {
    let bundle = copied("export-inside");
    let out_file = bundle.join("program.md");

    let out = tolearn(&[
        "export",
        bundle.to_str().unwrap(),
        "--out",
        out_file.to_str().unwrap(),
    ]);

    assert_ne!(code(&out), 0);
    assert!(stderr(&out).contains("бандл"), "{}", stderr(&out));
    assert!(!out_file.exists(), "файл всё-таки записан в бандл");
}
