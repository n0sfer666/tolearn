#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generation gate: a panic here is the report"
)]

mod support;

use tolearn_core::generate::{
    Level, PROGRESS_SCHEMA, ROADMAP_SCHEMA, Request, TOPIC_SCHEMA, fenced, generated, pick, repair,
    roadmap, stamped, topic,
};

use support::bundles;

const TODAY: &str = "2026-08-08";

fn request() -> Request {
    Request {
        subject: "поднять локальную модель и говорить с ней из своего кода".to_owned(),
        level: Level::Basics,
        weekly_hours: 6,
        weeks: Some(10),
        locale: "ru".to_owned(),
        today: TODAY.to_owned(),
    }
}

#[test]
fn the_skeleton_prompt_carries_the_request_under_its_own_heading() {
    let asked = roadmap(&request());
    let (rules, told) = asked.rsplit_once("## Запрос").expect("no request section");
    assert!(
        rules.contains("roadmap.yaml"),
        "the rules of the skeleton step are gone from the prompt"
    );
    assert!(told.contains("поднять локальную модель"));
    assert!(told.contains("знает основы"));
    assert!(told.contains("- Часов в неделю: 6"));
    assert!(told.contains("10 недель"));
    assert!(told.contains("- Язык программы: ru"));
}

#[test]
fn a_request_without_a_deadline_says_so_instead_of_leaving_a_hole() {
    let asked = roadmap(&Request {
        weeks: None,
        ..request()
    });
    assert!(asked.contains("- Срок: срока нет"));
}

#[test]
fn the_topic_prompt_repeats_what_the_skeleton_already_fixed() {
    let (map, _) = bundles::reference();
    let entry = map.topics[bundles::entry(&map, "local-runtime")].clone();
    let asked = topic(&map, &entry, TODAY);
    let (rules, told) = asked.rsplit_once("## Тема").expect("no topic section");
    assert!(
        rules.contains("`questions[].id`"),
        "the rules of the topic step are gone from the prompt"
    );
    assert!(told.contains("- `id`: local-runtime"));
    assert!(told.contains(&format!("- `title`: {}", entry.title)));
    assert!(told.contains(&format!(
        "- `est_hours`: [{}, {}]",
        entry.est_hours.min, entry.est_hours.max
    )));
    assert!(told.contains(&format!("- `priority`: {}", entry.priority.label())));
}

#[test]
fn the_topic_prompt_lists_the_other_topics_so_dependencies_have_somewhere_to_point() {
    let (map, _) = bundles::reference();
    let entry = map.topics[bundles::entry(&map, "local-runtime")].clone();
    let told = topic(&map, &entry, TODAY);
    for other in &map.topics {
        assert_eq!(
            told.contains(&format!("`{}` — {}", other.id, other.title)),
            other.id != entry.id,
            "`{}` is listed to itself or lost from the list",
            other.id
        );
    }
}

#[test]
fn the_repair_prompt_keeps_the_rules_the_answer_and_the_complaints_together() {
    let asked = roadmap(&request());
    let mended = repair(
        &asked,
        "schema: learning-roadmap/v1\n",
        &["bundle.empty-stages: в `stages` пусто".to_owned()],
    );
    assert!(mended.contains("## Запрос"), "the rules were dropped");
    assert!(mended.contains("schema: learning-roadmap/v1"));
    assert!(mended.contains("- bundle.empty-stages: в `stages` пусто"));
    assert!(mended.contains("целиком"));
}

#[test]
fn the_answer_is_taken_by_the_schema_it_declares_not_by_its_place() {
    let said = "Вот прогресс:\n\n```yaml\nschema: learning-roadmap/progress/v1\nroadmap_id: x\n```\n\
                И шапка:\n\n```yaml\nschema: learning-roadmap/v1\nid: x\n```\n";
    assert_eq!(fenced(said).len(), 2);
    assert!(pick(said, ROADMAP_SCHEMA).unwrap().contains("id: x"));
    assert!(
        pick(said, PROGRESS_SCHEMA)
            .unwrap()
            .contains("roadmap_id: x")
    );
    assert!(pick(said, TOPIC_SCHEMA).is_none());
}

#[test]
fn a_bare_answer_without_fences_is_still_read() {
    let said = "schema: learning-roadmap/topic/v1\nid: local-runtime\n";
    assert_eq!(pick(said, TOPIC_SCHEMA).unwrap(), said);
}

#[test]
fn an_answer_cut_off_before_the_closing_fence_is_kept_for_the_validator_to_judge() {
    let said = "```yaml\nschema: learning-roadmap/v1\nid: x\n";
    let taken = pick(said, ROADMAP_SCHEMA).expect("the truncated block was thrown away");
    assert!(taken.contains("id: x"));
}

#[test]
fn the_finished_program_says_its_stages_are_generated_and_changes_nothing_else() {
    let skeleton = "stages:\n  - n: 1\n    title: generated: false в заголовке\n    \
                    generated: false\n  - n: 2\n    generated: false\ntopics: []\n";
    let marked = generated(skeleton);
    assert_eq!(marked.matches("generated: true").count(), 2);
    assert!(marked.contains("title: generated: false в заголовке"));
    assert!(marked.ends_with("topics: []\n"));
}

#[test]
fn a_stage_written_as_one_line_is_marked_too() {
    let marked = generated("stages:\n  - generated: false\n");
    assert_eq!(marked, "stages:\n  - generated: true\n");
}

#[test]
fn both_prompts_hand_the_model_todays_date_because_its_own_clock_is_wrong() {
    assert!(roadmap(&request()).contains(&format!("- Сегодня: {TODAY}")));
    let (map, _) = bundles::reference();
    let entry = map.topics[bundles::entry(&map, "local-runtime")].clone();
    assert!(topic(&map, &entry, TODAY).contains(&format!("- Сегодня: {TODAY}")));
}

#[test]
fn a_date_the_model_made_up_is_overwritten_by_the_one_we_were_given() {
    let said = "schema: learning-roadmap/v1\ngenerated_at: 2024-01-15\n\
                title: generated_at: 2024-01-15 в заголовке\ngenerated_by: claude\n";
    let fixed = stamped(said, TODAY);
    assert!(fixed.contains(&format!("generated_at: {TODAY}\n")));
    assert!(!fixed.contains("generated_at: 2024-01-15\n"));
    assert!(fixed.contains("title: generated_at: 2024-01-15 в заголовке"));
    assert!(fixed.ends_with("generated_by: claude\n"));
}

#[test]
fn a_roadmap_without_a_stamp_is_left_exactly_as_it_came() {
    let said = "schema: learning-roadmap/v1\nid: x\n";
    assert_eq!(stamped(said, TODAY), said);
}
