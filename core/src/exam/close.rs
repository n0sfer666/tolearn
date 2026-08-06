use crate::progress::Answer;
use crate::topic::Topic;

use super::types::Artifact;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collected {
    pub graded: Vec<Answer>,
    pub hinted: Vec<String>,
    pub artifact: Artifact,
    pub failed_checks: Vec<String>,
    pub today: String,
}

pub fn verdict(examiner: &str, topic: &Topic, got: &Collected) -> String {
    let told = [
        format!("- `topic_id`: {}", topic.id),
        format!("- Сегодняшняя дата: {}", got.today),
        format!("- Практика: {}", got.artifact.told()),
        listed("Несошедшиеся критерии приёмки", &got.failed_checks),
    ];
    format!(
        "{}\n\n{RULE}\n\n## Диалог уже проведён\n\n{TOLD}\n\n{}\n\n### Оценки по вопросам\n\n{}\
         \n{}\n## Что вывести\n\n{OUTPUT}\n",
        examiner.trim_end(),
        told.join("\n"),
        graded(got),
        skipped(topic, got),
    )
}

const RULE: &str = "---";

const TOLD: &str = "Зачёт состоялся: приложение задавало вопросы по одному, ты оценивал ответы \
по ходу.\nНиже — готовые оценки. Новых вопросов не задавай, прежние оценки не переигрывай, \
вопросов,\nкоторых нет в теме, не выдумывай: твоё дело — собрать вердикт по цепочке из раздела\n\
«Правило вердикта» и пройти калибровочный гейт.";

const OUTPUT: &str = "Ровно то, что описано в разделе «Вывод»: разбор для человека и один блок \
json после\nнего. `per_question` заполни оценками выше слово в слово — `id`, `result`, цитаты и\n\
`missed` менять нельзя, своих записей не добавляй. `hinted` — `true`, если подсказка\nпонадобилась \
хоть по одному вопросу. `date` — сегодняшняя дата со входа.";

fn graded(got: &Collected) -> String {
    if got.graded.is_empty() {
        return "Ни один вопрос не оценён: студент завершил зачёт до первого ответа.\n".to_owned();
    }
    got.graded
        .iter()
        .map(|answer| told(answer, got.hinted.contains(&answer.id)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn told(answer: &Answer, hinted: bool) -> String {
    let mut block = format!(
        "- `{}` — `{}`; подсказка: {}",
        answer.id,
        answer.outcome.label(),
        if hinted { "да" } else { "нет" }
    );
    if let Some(quote) = &answer.quote {
        block.push_str(&format!("\n  цитата: «{}»", quote.trim()));
    }
    if !answer.missed.is_empty() {
        block.push_str(&format!("\n  не прозвучало: {}", answer.missed.join("; ")));
    }
    if answer.signal_extension {
        block.push_str("\n  сигнал передан своими словами");
    }
    block.push('\n');
    block
}

fn skipped(topic: &Topic, got: &Collected) -> String {
    let left: Vec<String> = topic
        .questions
        .iter()
        .filter(|asked| !got.graded.iter().any(|answer| answer.id == asked.id))
        .map(|asked| format!("- `{}`", asked.id))
        .collect();
    if left.is_empty() {
        return String::new();
    }
    format!(
        "\n### Не заданы\n\n{}\n\nЭти вопросы не задавались — зачёт завершён раньше. `miss` за \
         них\nставить нельзя, в `per_question` они не идут: перечисли их в `notes`.\n",
        left.join("\n")
    )
}

fn listed(title: &str, items: &[String]) -> String {
    if items.is_empty() {
        return format!("- {title}: нет");
    }
    format!("- {title}: {}", items.join(", "))
}
