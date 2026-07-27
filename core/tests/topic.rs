#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::Hours;
use tolearn_core::topic::{
    Check, Confidence, Liveness, Material, MaterialTier, MaterialType, Practice, PracticeKind,
    PracticeTier, Question, QuestionType, Retention, Topic, Volatility, parse,
};

use support::{every_fixture_parses, read};

const REFERENCE: &str = "examples/llm-agents-base/topics/local-runtime.yaml";
const VALID: &str = "fixtures/valid/topic";

#[test]
fn the_reference_topic_parses_field_by_field() {
    let Topic {
        schema,
        id,
        title,
        stage,
        depends_on,
        est_hours,
        volatility,
        revalidate_after_days,
        verified_at,
        confidence,
        retention,
        version_context,
        outcomes,
        misconceptions,
        materials,
        practice,
        questions,
        exam,
    } = parse(&read(REFERENCE)).unwrap();

    assert_eq!(schema, "learning-roadmap/topic/v1");
    assert_eq!(id, "local-runtime");
    assert_eq!(
        title,
        "Локальный рантайм — поднять модель и понять, что съело память"
    );
    assert_eq!(stage, 1);
    assert!(depends_on.is_empty());
    assert_eq!(est_hours, Hours { min: 4, max: 6 });
    assert_eq!(volatility, Volatility::Volatile);
    assert_eq!(revalidate_after_days, 90);
    assert_eq!(verified_at, "2026-07-26");
    assert_eq!(confidence, Confidence::High);
    assert_eq!(retention, Retention::ByUse);
    assert_eq!(
        version_context,
        ["ollama", "llama_cpp", "lm_studio", "vllm"]
    );

    assert_eq!(outcomes.len(), 3);
    assert_eq!(misconceptions.len(), 6);
    assert_eq!(materials.len(), 7);
    assert_eq!(questions.len(), 4);
    assert_eq!(practice.constraints.len(), 3);
    assert_eq!(exam.traps.len(), 6);
    assert_eq!(exam.max_exchanges, 9);
    assert!(exam.artifact_required);
    assert!(exam.focus.starts_with("Проверяем"), "{}", exam.focus);
}

#[test]
fn a_material_of_the_reference_lands_field_in_field() {
    let Material {
        title,
        url,
        kind,
        tier,
        lang,
        liveness,
        published,
        covers_version,
        checked_at,
        stale,
        delta,
        note,
    } = parse(&read(REFERENCE)).unwrap().materials.remove(3);

    assert_eq!(
        title,
        "Why does llama.cpp use so much VRAM (and RAM)? · Discussion #9784"
    );
    assert_eq!(
        url,
        "https://github.com/ggml-org/llama.cpp/discussions/9784"
    );
    assert_eq!(kind, MaterialType::Article);
    assert_eq!(tier, MaterialTier::T2);
    assert_eq!(lang, "en");
    assert_eq!(liveness, Liveness::Ok);
    assert_eq!(published.as_deref(), Some("2024-10"));
    assert_eq!(covers_version, None);
    assert_eq!(checked_at, "2026-07-26");
    assert!(stale);
    assert!(
        delta.unwrap().starts_with("Абсолютные числа в разборе"),
        "delta holds something else"
    );
    assert!(
        note.starts_with("Единственный материал"),
        "note holds `{note}`"
    );
}

#[test]
fn the_practice_of_the_reference_lands_field_in_field() {
    let Practice {
        kind,
        tier,
        task,
        deliverable,
        starting_point,
        fallback,
        time_box_min,
        smoke_checked,
        mut constraints,
        acceptance,
    } = parse(&read(REFERENCE)).unwrap().practice;

    assert_eq!(kind, PracticeKind::Ops);
    assert_eq!(tier, PracticeTier::P1);
    assert!(task.starts_with("Собери в отдельном каталоге"), "{task}");
    assert!(
        deliverable.starts_with("Каталог с файлами"),
        "{deliverable}"
    );
    assert_eq!(starting_point, None);
    assert_eq!(fallback, None);
    assert_eq!(time_box_min, 75);
    assert!(!smoke_checked);
    assert_eq!(acceptance.len(), 6);

    let Check {
        id,
        claim,
        check,
        expect,
    } = constraints.remove(0);

    assert_eq!(id, "c1");
    assert!(claim.starts_with("Контекст в Ollama задан"), "{claim}");
    assert!(check.starts_with("bash -c 'test -s Modelfile"), "{check}");
    assert_eq!(expect, "MODELFILE_OK\n");
}

#[test]
fn a_question_of_the_reference_lands_field_in_field() {
    let Question {
        id,
        kind,
        text,
        expected_signals,
        red_flags,
        follow_up,
    } = parse(&read(REFERENCE)).unwrap().questions.remove(3);

    assert_eq!(id, "q4");
    assert_eq!(kind, QuestionType::Misconception);
    assert!(text.starts_with("Разработчик рассуждает так."), "{text}");
    assert_eq!(expected_signals.len(), 4);
    assert!(
        expected_signals[0].starts_with("Квантизация весов"),
        "{}",
        expected_signals[0]
    );
    assert_eq!(red_flags.len(), 3);
    assert!(
        red_flags[0].starts_with("Ответ пришёл с кодом 200"),
        "{}",
        red_flags[0]
    );
    assert!(
        follow_up.unwrap().starts_with("А если тот же сценарий"),
        "follow_up holds something else"
    );
}

#[test]
fn every_valid_fixture_parses() {
    every_fixture_parses(VALID, parse, 6);
}
