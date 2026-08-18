#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "exam gate: a panic here is the report"
)]

mod support;

use tolearn_core::exam::{
    Artifact, Collected, Line, Next, Side, hint, practice, shown, step, turn, verdict,
};
use tolearn_core::progress::{Answer, Outcome};
use tolearn_core::prompt::render;
use tolearn_core::roadmap::Roadmap;
use tolearn_core::topic::{Question, Topic};

use support::{bundles, read};

const TEMPLATE: &str = "examples/llm-agents-base/examiner.md";

fn of(id: &str) -> (Roadmap, Topic) {
    let (map, topics) = bundles::reference();
    (map.clone(), topics[bundles::document(&topics, id)].clone())
}

fn topic() -> Topic {
    of("local-runtime").1
}

fn asked(topic: &Topic, id: &str) -> Question {
    topic
        .questions
        .iter()
        .find(|question| question.id == id)
        .unwrap_or_else(|| panic!("no question `{id}`"))
        .clone()
}

fn head(text: &str) -> &str {
    text.trim().lines().next().unwrap()
}

fn said(text: &str) -> Line {
    Line {
        side: Side::Student,
        text: text.to_owned(),
    }
}

fn answer(id: &str, outcome: Outcome) -> Answer {
    Answer {
        id: id.to_owned(),
        outcome,
        quote: Some("держит веса в оперативной памяти".to_owned()),
        missed: vec!["выгрузка слоёв".to_owned()],
        signal_extension: false,
    }
}

#[test]
fn the_turn_prompt_carries_one_question_and_only_its_own_talk() {
    let topic = topic();
    let one = asked(&topic, "q1");
    let other = asked(&topic, "q2");

    let prompt = turn(&topic, &one, &[said("веса лежат в памяти")], false);

    assert!(prompt.contains(head(&one.text)), "{prompt}");
    assert!(
        !prompt.contains(head(&other.text)),
        "второй вопрос утёк в шаг"
    );
    assert!(prompt.contains("веса лежат в памяти"));
    assert!(prompt.contains("- Подсказка уже выдана: нет"));
    assert!(prompt.contains(head(&topic.exam.focus)));
}

#[test]
fn the_turn_prompt_says_when_a_hint_was_given() {
    let topic = topic();
    let one = asked(&topic, "q1");

    let prompt = turn(&topic, &one, &[said("не знаю")], true);

    assert!(prompt.contains("- Подсказка уже выдана: да"), "{prompt}");
}

#[test]
fn a_fresh_question_says_the_student_kept_silent() {
    let topic = topic();

    let prompt = turn(&topic, &asked(&topic, "q1"), &[], false);

    assert!(prompt.contains("Студент ещё ничего не сказал"), "{prompt}");
}

#[test]
fn the_hint_prompt_asks_for_a_direction_not_for_a_grade() {
    let topic = topic();

    let prompt = hint(&topic, &asked(&topic, "q1"), &[said("застрял")]);

    assert!(prompt.contains("минимальную"), "{prompt}");
    assert!(prompt.contains("застрял"));
    assert!(!prompt.contains("\"result\""), "подсказку просят с оценкой");
}

#[test]
fn the_practice_prompt_carries_the_task_and_its_acceptance() {
    let topic = topic();

    let prompt = practice(&topic, &[]);

    assert!(
        prompt.contains(head(&topic.practice.deliverable)),
        "{prompt}"
    );
    for check in &topic.practice.acceptance {
        assert!(
            prompt.contains(head(&check.expect)),
            "критерий {} потерян",
            check.id
        );
    }
}

#[test]
fn a_step_carries_the_grade_and_what_to_say_next() {
    let topic = topic();
    let one = asked(&topic, "q1");

    let got = step(
        "Разбор.\n\n```json\n{\"id\": \"q1\", \"result\": \"partial\", \"quote\": \"веса в памяти\",\
         \n\"missed\": [\"выгрузка слоёв\"], \"next\": \"follow_up\", \"say\": \"А если не влезет?\"}\n```\n",
        &one,
    )
    .unwrap();

    assert_eq!(got.answer.outcome, Outcome::Partial);
    assert_eq!(got.answer.missed, vec!["выгрузка слоёв".to_owned()]);
    assert_eq!(got.next, Next::FollowUp);
    assert_eq!(got.say, "А если не влезет?");
}

#[test]
fn a_step_without_next_closes_the_question() {
    let topic = topic();

    let got = step(
        "{\"id\": \"q1\", \"result\": \"ok\", \"quote\": \"держит веса\"}",
        &asked(&topic, "q1"),
    )
    .unwrap();

    assert_eq!(got.next, Next::Close);
}

#[test]
fn a_step_about_another_question_is_refused() {
    let topic = topic();

    let failed = step(
        "{\"id\": \"q2\", \"result\": \"ok\", \"quote\": \"держит веса\"}",
        &asked(&topic, "q1"),
    )
    .unwrap_err();

    assert_eq!(failed.code(), "exam.wrong-question");
}

#[test]
fn a_step_without_json_is_refused() {
    let topic = topic();

    let failed = step("Принято, дальше.", &asked(&topic, "q1")).unwrap_err();

    assert_eq!(failed.code(), "exam.no-json");
}

#[test]
fn the_practice_answer_carries_the_artifact_and_the_failed_checks() {
    let got = shown(
        "```json\n{\"artifact\": \"shown\", \"failed_checks\": [\"a1\"], \"next\": \"close\",\
         \n\"say\": \"принято\"}\n```\n",
    )
    .unwrap();

    assert_eq!(got.artifact, Artifact::Shown);
    assert_eq!(got.failed_checks, vec!["a1".to_owned()]);
    assert_eq!(got.next, Next::Close);
}

#[test]
fn the_verdict_prompt_keeps_the_bundle_protocol_and_adds_the_grades() {
    let (map, topic) = of("local-runtime");
    let examiner = render(&read(TEMPLATE), &topic, &map).unwrap();

    let prompt = verdict(
        &examiner,
        &topic,
        &Collected {
            graded: vec![answer("q1", Outcome::Partial)],
            hinted: vec!["q1".to_owned()],
            artifact: Artifact::Passed,
            failed_checks: Vec::new(),
            today: "2026-08-06".to_owned(),
        },
    );

    assert!(prompt.starts_with(&examiner), "протокол бандла потерян");
    assert!(
        prompt.contains("- `q1` — `partial`; подсказка: да"),
        "{prompt}"
    );
    assert!(prompt.contains("держит веса в оперативной памяти"));
    assert!(prompt.contains("артефакт предъявлен и принят"));
    assert!(prompt.contains("- Сегодняшняя дата: 2026-08-06"));
}

#[test]
fn questions_left_unasked_are_named_and_forbidden_to_grade() {
    let (map, topic) = of("local-runtime");
    let examiner = render(&read(TEMPLATE), &topic, &map).unwrap();

    let prompt = verdict(
        &examiner,
        &topic,
        &Collected {
            graded: vec![answer("q1", Outcome::Ok)],
            hinted: Vec::new(),
            artifact: Artifact::None,
            failed_checks: vec!["a1".to_owned()],
            today: "2026-08-06".to_owned(),
        },
    );

    assert!(prompt.contains("### Не заданы"), "{prompt}");
    assert!(prompt.contains("- `q2`"));
    assert!(prompt.contains("`miss` за"));
    assert!(prompt.contains("- Несошедшиеся критерии приёмки: a1"));
}
