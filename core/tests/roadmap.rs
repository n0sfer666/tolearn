#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::Hours;
use tolearn_core::roadmap::{
    Calibration, CalibrationMethod, Defaults, Priority, RevalidateAfterDays, Roadmap, Stage,
    TopicEntry, parse,
};

use support::{every_fixture_parses, read};

const REFERENCE: &str = "examples/llm-agents-base/roadmap.yaml";
const VALID: &str = "fixtures/valid/roadmap";
const EXTRA_FIELD: &str =
    "fixtures/broken/roadmap/additionalProperties__roadmap-with-an-extra-field.yaml";

#[test]
fn the_reference_roadmap_parses_field_by_field() {
    let Roadmap {
        schema,
        id,
        title,
        subject,
        goal,
        generated_at,
        generated_by,
        locale,
        weekly_hours,
        env_constraints,
        version_pins,
        calibration,
        defaults,
        stages,
        topics,
    } = parse(&read(REFERENCE)).unwrap();

    assert_eq!(schema, "learning-roadmap/v1");
    assert_eq!(id, "llm-agents-base");
    assert_eq!(
        title,
        "Работа с агентами LLM — база на готовых инструментах"
    );
    assert_eq!(
        subject,
        "Агенты LLM — CLI-агенты, MCP, RAG, локальные модели, подключение сторонних провайдеров\n"
    );
    assert_eq!(goal.lines().count(), 5);
    assert!(goal.ends_with("повёл себя именно так.\n"), "{goal}");
    assert_eq!(generated_at, "2026-07-26");
    assert_eq!(generated_by, "claude-opus-5");
    assert_eq!(locale, "ru");
    assert_eq!(weekly_hours, 6);

    assert_eq!(env_constraints.len(), 6);
    assert_eq!(
        env_constraints[0],
        "Docker и тулчейны ставить можно, ограничений нет"
    );
    assert_eq!(env_constraints[2].lines().count(), 2);

    assert_eq!(version_pins.len(), 32);
    assert_eq!(version_pins["ollama"], "v0.32.4");
    assert_eq!(version_pins["owasp_agentic_top10"], "2026");

    assert_eq!(
        calibration,
        Calibration {
            method: CalibrationMethod::DiagnosticProbe,
            probed: 3,
            passed_out: Vec::new(),
            interrupted: true,
        }
    );
    assert_eq!(
        defaults,
        Defaults {
            revalidate_after_days: RevalidateAfterDays {
                stable: 730,
                evolving: 365,
                volatile: 90,
            },
        }
    );

    assert_eq!(stages.len(), 3);
    assert_eq!(
        stages[1],
        Stage {
            n: 2,
            title: "Агенты и инструменты".to_owned(),
            generated: false,
            checkpoint: "cp-agent-stack".to_owned(),
        }
    );

    assert_eq!(topics.len(), 23);
    assert_eq!(
        topics[0],
        TopicEntry {
            id: "local-runtime".to_owned(),
            title: "Локальный рантайм — поднять модель и понять, что съело память".to_owned(),
            stage: 1,
            file: "topics/local-runtime.yaml".to_owned(),
            est_hours: Hours { min: 4, max: 6 },
            priority: Priority::Core,
        }
    );
    assert_eq!(topics[22].est_hours, Hours { min: 3, max: 3 });
    assert_eq!(count(&topics, Priority::Recommended), 4);
    assert_eq!(count(&topics, Priority::Optional), 2);
}

fn count(topics: &[TopicEntry], priority: Priority) -> usize {
    topics
        .iter()
        .filter(|topic| topic.priority == priority)
        .count()
}

#[test]
fn every_valid_fixture_parses() {
    every_fixture_parses(VALID, parse, 2);
}

#[test]
fn calibration_carries_the_topics_it_passed_out() {
    let corpus = parse(&read(&format!("{VALID}/corpus-program.yaml"))).unwrap();

    assert_eq!(
        corpus.calibration,
        Calibration {
            method: CalibrationMethod::SelfReport,
            probed: 2,
            passed_out: vec!["offline-edge".to_owned()],
            interrupted: false,
        }
    );
    assert_eq!(corpus.topics.len(), 5);
    assert_eq!(corpus.stages.len(), 2);
}

#[test]
fn an_unknown_field_does_not_break_the_parse() {
    let source = read(EXTRA_FIELD);
    assert!(
        source.contains("notes:"),
        "the fixture lost its extra field"
    );

    let parsed = parse(&source).unwrap();

    assert_eq!(parsed.id, "minimal-program");
    assert_eq!(parsed.topics.len(), 1);
}

#[test]
fn a_byte_order_mark_does_not_hide_the_first_field() {
    let source = format!("\u{feff}{}", read(&format!("{VALID}/minimal.yaml")));

    let parsed = parse(&source).unwrap();

    assert_eq!(parsed.schema, "learning-roadmap/v1");
}
