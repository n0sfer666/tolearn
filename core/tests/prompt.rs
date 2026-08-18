#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "prompt gate: a panic here is the report"
)]

mod support;

use tolearn_core::prompt::{RenderError, render};
use tolearn_core::roadmap::Roadmap;
use tolearn_core::topic::Topic;

use support::{bundles, read};

const TEMPLATE: &str = "examples/llm-agents-base/examiner.md";
const GOLDEN: &str = "fixtures/golden/examiner/local-runtime.md";

fn reference() -> (Roadmap, Vec<Topic>, String) {
    let (map, topics) = bundles::reference();
    (map, topics, read(TEMPLATE))
}

fn of(id: &str) -> (Roadmap, Topic, String) {
    let (map, topics, template) = reference();
    let topic = topics[bundles::document(&topics, id)].clone();
    (map, topic, template)
}

fn wrap(body: &str) -> String {
    format!(
        "# Заголовок\n\nСлужебное вступление.\n\n---\n\n{body}\n\n---\n\n## Что трекер делает\n\nХвост.\n"
    )
}

fn one(body: &str) -> String {
    let (map, topic, _) = of("local-runtime");
    render(&wrap(body), &topic, &map).unwrap()
}

#[test]
fn the_prompt_is_only_what_lies_between_the_first_and_the_second_rule() {
    let (map, topic, template) = of("local-runtime");

    let prompt = render(&template, &topic, &map).unwrap();

    assert!(!prompt.contains("Копируется в бандл"));
    assert!(!prompt.contains("Что трекер делает с результатом"));
    assert!(prompt.contains("Ты принимаешь зачёт по одному топику"));
}

#[test]
fn a_text_without_two_rules_is_no_prompt() {
    let (map, topic, _) = of("local-runtime");

    let error = render("# Просто заголовок\n\n---\n\nтело\n", &topic, &map).unwrap_err();

    assert!(matches!(error, RenderError::NoPrompt));
}

#[test]
fn nothing_of_the_rendered_prompt_is_left_unsubstituted() {
    let (map, topics, template) = reference();

    for topic in &topics {
        let prompt = render(&template, topic, &map).unwrap();
        assert!(
            !prompt.contains("{{"),
            "`{}` left a placeholder behind",
            topic.id
        );
    }
}

#[test]
fn a_scalar_of_the_topic_lands_in_the_line_that_asked_for_it() {
    let prompt = one("- `topic_id`: {{id}}");

    assert_eq!(prompt.trim(), "- `topic_id`: local-runtime");
}

#[test]
fn a_field_of_the_roadmap_is_substituted_too() {
    let (mut map, topic, _) = of("local-runtime");
    map.generated_at = "2026-01-31".to_owned();

    let prompt = render(
        &wrap("Актуальные факты на {{roadmap.generated_at}}."),
        &topic,
        &map,
    )
    .unwrap();

    assert_eq!(prompt.trim(), "Актуальные факты на 2026-01-31.");
}

#[test]
fn the_version_pins_of_the_roadmap_become_a_list() {
    let prompt = one("- Версии: {{roadmap.version_pins}}");

    assert!(prompt.starts_with("- Версии:\n"));
    assert!(prompt.contains("\n  - ollama: "));
}

#[test]
fn a_collection_is_a_list_nested_under_the_line() {
    let prompt = one("- Ожидаемые умения: {{outcomes}}");

    assert!(prompt.starts_with("- Ожидаемые умения:\n  - "));
    assert!(!prompt.contains("{{"));
}

#[test]
fn a_line_that_lost_its_only_value_is_cut_out_whole() {
    let (map, mut topic, _) = of("local-runtime");
    topic.misconceptions.clear();

    let prompt = render(
        &wrap("- Тема: {{title}}\n- Заблуждения: {{misconceptions}}"),
        &topic,
        &map,
    )
    .unwrap();

    assert!(prompt.contains("- Тема: "));
    assert!(!prompt.contains("Заблуждения"));
}

#[test]
fn an_empty_scalar_cuts_its_line_out_as_well() {
    let (map, mut topic, _) = of("local-runtime");
    topic.exam.focus = String::new();

    let prompt = render(&wrap("- Фокус: {{exam.focus}}"), &topic, &map).unwrap();

    assert!(!prompt.contains("Фокус"));
}

#[test]
fn a_boolean_that_is_false_is_a_value_and_not_an_emptiness() {
    let (map, mut topic, _) = of("local-runtime");
    topic.exam.artifact_required = false;

    let prompt = render(
        &wrap("- Артефакт: {{exam.artifact_required}}"),
        &topic,
        &map,
    )
    .unwrap();

    assert_eq!(prompt.trim(), "- Артефакт: false");
}

#[test]
fn two_scalars_on_one_line_are_both_substituted() {
    let prompt = one("- Практика: {{practice.task}} / {{practice.deliverable}}");

    assert!(prompt.contains(" / "));
    assert!(!prompt.contains("{{"));
}

#[test]
fn a_placeholder_the_render_does_not_know_is_an_error_that_names_it() {
    let (map, topic, _) = of("local-runtime");

    let error = render(&wrap("- Что-то: {{topic.mystery}}"), &topic, &map).unwrap_err();

    assert!(matches!(&error, RenderError::Unknown { name } if name == "topic.mystery"));
    assert!(error.to_string().contains("topic.mystery"));
}

#[test]
fn a_placeholder_that_is_never_closed_is_an_error() {
    let (map, topic, _) = of("local-runtime");

    let error = render(&wrap("- Что-то: {{id"), &topic, &map).unwrap_err();

    assert!(matches!(error, RenderError::Unclosed { .. }));
}

#[test]
fn a_question_carries_its_signals_red_flags_and_follow_up() {
    let prompt = one("- Вопросы: {{questions}}");

    assert!(prompt.contains("q1"));
    assert!(prompt.contains("сигналы:"));
    assert!(prompt.contains("добивочный:"));
}

#[test]
fn the_reference_topic_renders_exactly_as_the_golden_file() {
    let (map, topic, template) = of("local-runtime");

    let prompt = render(&template, &topic, &map).unwrap();

    assert_eq!(prompt, read(GOLDEN));
}
