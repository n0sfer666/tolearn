use crate::progress::Status;
use crate::topic::{Question, Topic};

use super::lines::Doc;

const HIDDEN: &str = "Ответы, красные флаги и ловушки не экспортированы: тема не зачтена.";

pub fn open(status: Option<Status>) -> bool {
    matches!(status, Some(Status::Passed) | Some(Status::StalePassed))
}

pub fn questions(doc: &mut Doc, topic: &Topic, shown: bool) {
    if topic.questions.is_empty() {
        return;
    }
    doc.heading(4, "Вопросы");
    let asked: Vec<String> = topic
        .questions
        .iter()
        .map(|question| format!("{} _({})_", question.text, question.kind.label()))
        .collect();
    doc.numbered(&asked);
    if !shown {
        doc.line(&format!("_{HIDDEN}_"));
        return;
    }
    for question in &topic.questions {
        answers(doc, question);
    }
}

fn answers(doc: &mut Doc, question: &Question) {
    if question.expected_signals.is_empty()
        && question.red_flags.is_empty()
        && question.follow_up.is_none()
    {
        return;
    }
    doc.heading(5, &format!("Разбор: {}", question.text));
    if !question.expected_signals.is_empty() {
        doc.line("Ожидаемые сигналы:");
        doc.bullets(&question.expected_signals);
    }
    if !question.red_flags.is_empty() {
        doc.line("Красные флаги:");
        doc.bullets(&question.red_flags);
    }
    if let Some(follow) = &question.follow_up {
        doc.line(&format!("Уточняющий вопрос: {follow}"));
    }
}

pub fn exam(doc: &mut Doc, topic: &Topic, shown: bool) {
    doc.heading(4, "Экзамен");
    doc.line(&topic.exam.focus);
    doc.bullets(&[
        format!(
            "Артефакт обязателен: {}",
            if topic.exam.artifact_required {
                "да"
            } else {
                "нет"
            }
        ),
        format!("Обменов не больше: {}", topic.exam.max_exchanges),
    ]);
    if !shown || topic.exam.traps.is_empty() {
        return;
    }
    doc.line("Ловушки:");
    doc.bullets(&topic.exam.traps);
}
