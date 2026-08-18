use tolearn_core::topic::Topic;

pub const OVER: &str =
    "Вопросы кончились. Можно завершать зачёт — или добавить, если есть что добавить.";

pub fn opening(topic: &Topic) -> String {
    format!(
        "Зачёт по теме «{}». Вопросов: {}{}. Подсказки снижают итог.",
        topic.title,
        topic.questions.len(),
        if topic.exam.artifact_required {
            ", плюс разбор практики"
        } else {
            ""
        }
    )
}

pub fn practice(topic: &Topic) -> String {
    let mut asked = format!(
        "Сначала практика.\n\nЗадание: {}\n\nЧто предъявить: {}",
        topic.practice.task.trim(),
        topic.practice.deliverable.trim()
    );
    if !topic.practice.acceptance.is_empty() {
        asked.push_str("\n\nПриложи вывод команд — запускаешь их сам:");
        for check in &topic.practice.acceptance {
            asked.push_str(&format!(
                "\n\n- `{}`\n  {}",
                check.check.trim(),
                check.claim.trim()
            ));
        }
    }
    asked
}
