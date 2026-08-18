use crate::progress::{Attempt, Outcome};
use crate::topic::Topic;

use super::types::Reviewed;

pub(super) fn request(topic: &Topic, questions: &[Reviewed], failures: &[&Attempt]) -> String {
    let mut text = format!(
        "Тема `{}` — «{}» не сдаётся: {} провала подряд.\n\n",
        topic.id,
        topic.title,
        failures.len()
    );
    text.push_str(&format!(
        "Оценка объёма: {}–{} ч.\n\n",
        topic.est_hours.min, topic.est_hours.max
    ));
    text.push_str(&block("Ожидаемые результаты темы", &topic.outcomes));
    text.push_str(&block("Вопросы, на которых я сыплюсь", &missed(questions)));
    text.push_str(&block("Накопленные пробелы", &gaps(failures)));
    text.push_str(
        "Раздели тему на две-три меньшие темы того же формата: у каждой свой `id`, \
         свои ожидаемые результаты, своя практика и свои вопросы. Скажи, в каком порядке \
         их проходить и какая из них закрывает пробелы выше.\n",
    );
    text
}

fn missed(questions: &[Reviewed]) -> Vec<String> {
    questions
        .iter()
        .filter(|question| {
            question
                .outcome
                .is_some_and(|outcome| outcome != Outcome::Ok)
        })
        .map(|question| format!("`{}` ({}) — {}", question.id, question.kind, question.text))
        .collect()
}

fn gaps(failures: &[&Attempt]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for gap in failures.iter().flat_map(|attempt| attempt.gaps.iter()) {
        if !seen.contains(gap) {
            seen.push(gap.clone());
        }
    }
    seen
}

fn block(title: &str, items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let lines = items
        .iter()
        .map(|item| format!("- {}", item.trim()))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{title}:\n{lines}\n\n")
}
