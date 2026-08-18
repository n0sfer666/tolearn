use crate::roadmap::Roadmap;
use crate::topic::{Check, Question, Topic};

pub(super) enum Value {
    Scalar(String),
    List(Vec<String>),
}

pub(super) fn value(name: &str, topic: &Topic, roadmap: &Roadmap) -> Option<Value> {
    let value = match name {
        "id" => Value::Scalar(topic.id.clone()),
        "title" => Value::Scalar(topic.title.clone()),
        "outcomes" => Value::List(topic.outcomes.clone()),
        "misconceptions" => Value::List(topic.misconceptions.clone()),
        "exam.focus" => Value::Scalar(topic.exam.focus.clone()),
        "exam.traps" => Value::List(topic.exam.traps.clone()),
        "exam.artifact_required" => Value::Scalar(topic.exam.artifact_required.to_string()),
        "exam.max_exchanges" => Value::Scalar(topic.exam.max_exchanges.to_string()),
        "practice.task" => Value::Scalar(topic.practice.task.clone()),
        "practice.deliverable" => Value::Scalar(topic.practice.deliverable.clone()),
        "practice.acceptance" => Value::List(checks(&topic.practice.acceptance)),
        "practice.constraints" => Value::List(checks(&topic.practice.constraints)),
        "questions" => Value::List(topic.questions.iter().map(question).collect()),
        "roadmap.generated_at" => Value::Scalar(roadmap.generated_at.clone()),
        "roadmap.version_pins" => Value::List(
            roadmap
                .version_pins
                .iter()
                .map(|(tool, version)| format!("{tool}: {version}"))
                .collect(),
        ),
        _ => return None,
    };
    Some(value)
}

fn checks(checks: &[Check]) -> Vec<String> {
    checks
        .iter()
        .map(|check| {
            format!(
                "`{}` — {}\n  команда: `{}`\n  ожидается: {}",
                check.id,
                flow(&check.claim),
                flow(&check.check),
                flow(&check.expect)
            )
        })
        .collect()
}

pub(crate) fn question(question: &Question) -> String {
    let mut block = format!(
        "`{}` ({}) — {}",
        question.id,
        question.kind.label(),
        flow(&question.text)
    );
    block.push_str(&part("сигналы", &question.expected_signals));
    block.push_str(&part("красные флаги", &question.red_flags));
    if let Some(follow_up) = &question.follow_up {
        block.push_str(&format!("\n  добивочный: {}", flow(follow_up)));
    }
    block
}

fn part(title: &str, items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let mut text = format!("\n  {title}:");
    for item in items {
        text.push_str(&format!("\n    - {}", flow(item)));
    }
    text
}

fn flow(text: &str) -> String {
    text.trim()
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n  ")
}
