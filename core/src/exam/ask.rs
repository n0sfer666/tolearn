use crate::prompt::{examining, question};
use crate::topic::{Check, Question, Topic};

use super::types::{Line, Side};

pub fn turn(topic: &Topic, asked: &Question, lines: &[Line], hinted: bool) -> String {
    let told = [
        about(topic),
        format!(
            "## Вопрос\n\n{}\n- Подсказка уже выдана: {}",
            question(asked),
            told(hinted)
        ),
        talk(lines),
    ];
    format!(
        "{}\n\n{}\n",
        examining::turn().trim_end(),
        told.join("\n\n")
    )
}

pub fn hint(topic: &Topic, asked: &Question, lines: &[Line]) -> String {
    let told = [
        about(topic),
        format!("## Вопрос\n\n{}", question(asked)),
        talk(lines),
    ];
    format!(
        "{}\n\n{}\n",
        examining::hint().trim_end(),
        told.join("\n\n")
    )
}

pub fn practice(topic: &Topic, lines: &[Line]) -> String {
    let work = [
        format!("- Задание: {}", flow(&topic.practice.task)),
        format!(
            "- Ожидаемый результат: {}",
            flow(&topic.practice.deliverable)
        ),
        listed("Критерии приёмки", &checks(&topic.practice.acceptance)),
        listed("Ограничения задания", &checks(&topic.practice.constraints)),
    ];
    let told = [
        about(topic),
        format!("## Практика\n\n{}", work.join("\n")),
        talk(lines),
    ];
    format!(
        "{}\n\n{}\n",
        examining::practice().trim_end(),
        told.join("\n\n")
    )
}

fn about(topic: &Topic) -> String {
    let told = [
        format!("- `topic_id`: {}", topic.id),
        format!("- Тема: {}", topic.title),
        format!("- Фокус проверки: {}", flow(&topic.exam.focus)),
        listed("Ожидаемые умения", &topic.outcomes),
        listed("Известные заблуждения", &topic.misconceptions),
    ];
    format!("## Тема\n\n{}", told.join("\n"))
}

fn talk(lines: &[Line]) -> String {
    if lines.is_empty() {
        return "## Разговор\n\nСтудент ещё ничего не сказал.".to_owned();
    }
    let said = lines
        .iter()
        .map(|line| format!("**{}:** {}", who(line.side), flow(&line.text)))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("## Разговор\n\n{said}")
}

fn who(side: Side) -> &'static str {
    match side {
        Side::Examiner => "Экзаменатор",
        Side::Student => "Студент",
    }
}

fn checks(checks: &[Check]) -> Vec<String> {
    checks
        .iter()
        .map(|check| {
            format!(
                "`{}` — {}\n    команда: `{}`\n    ожидается: {}",
                check.id,
                flow(&check.claim),
                flow(&check.check),
                flow(&check.expect)
            )
        })
        .collect()
}

fn listed(title: &str, items: &[String]) -> String {
    if items.is_empty() {
        return format!("- {title}: нет");
    }
    let body = items
        .iter()
        .map(|item| format!("  - {item}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("- {title}:\n{body}")
}

fn told(yes: bool) -> &'static str {
    if yes { "да" } else { "нет" }
}

fn flow(text: &str) -> String {
    text.trim()
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n  ")
}
