use crate::prompt::generation;
use crate::roadmap::{Roadmap, TopicEntry};

use super::request::Request;

pub fn roadmap(request: &Request) -> String {
    let deadline = match request.weeks {
        Some(weeks) => format!("{weeks} недель"),
        None => "срока нет".to_owned(),
    };
    let asked = [
        format!("- Чему хочу научиться: {}", request.subject.trim()),
        format!("- Что уже знаю: {}", request.level.told()),
        format!("- Часов в неделю: {}", request.weekly_hours),
        format!("- Срок: {deadline}"),
        format!("- Язык программы: {}", request.locale),
        format!("- Сегодня: {}", request.today),
    ];
    section(generation::roadmap(), "## Запрос", &asked.join("\n"))
}

pub fn topic(roadmap: &Roadmap, entry: &TopicEntry, today: &str) -> String {
    let stage = roadmap
        .stages
        .iter()
        .find(|stage| stage.n == entry.stage)
        .map_or_else(|| entry.stage.to_string(), |stage| stage.title.clone());
    let told = [
        format!("- `id`: {}", entry.id),
        format!("- `title`: {}", entry.title),
        format!("- `stage`: {} ({stage})", entry.stage),
        format!(
            "- `est_hours`: [{}, {}]",
            entry.est_hours.min, entry.est_hours.max
        ),
        format!("- `priority`: {}", entry.priority.label()),
        format!("- Язык программы: {}", roadmap.locale),
        format!("- Цель программы: {}", roadmap.goal),
        format!("- Сегодня: {today}"),
        format!(
            "- `defaults.revalidate_after_days`: stable {}, evolving {}, volatile {}",
            roadmap.defaults.revalidate_after_days.stable,
            roadmap.defaults.revalidate_after_days.evolving,
            roadmap.defaults.revalidate_after_days.volatile
        ),
        listed("Закреплённые версии", &pins(roadmap)),
        listed(
            "Темы раньше этой по плану — только их и можно ставить в `depends_on`",
            &earlier(roadmap, entry),
        ),
        listed(
            "Темы позже этой по плану — ставить их в `depends_on` нельзя",
            &later(roadmap, entry),
        ),
    ];
    section(generation::topic(), "## Тема", &told.join("\n"))
}

pub fn repair(asked: &str, answered: &str, complaints: &[String]) -> String {
    let named = complaints
        .iter()
        .map(|complaint| format!("- {complaint}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{asked}\n\n## Прошлый ответ\n\nОн отвергнут, вот он целиком:\n\n{}\n\n\
         ## Что в нём не так\n\n{named}\n\n\
         Почини названное и выведи файл целиком заново — правок кусками приложение не принимает.\n",
        answered.trim()
    )
}

fn section(prompt: &str, title: &str, body: &str) -> String {
    format!("{}\n\n{title}\n\n{body}\n", prompt.trim_end())
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

fn pins(roadmap: &Roadmap) -> Vec<String> {
    roadmap
        .version_pins
        .iter()
        .map(|(name, version)| format!("`{name}`: {version}"))
        .collect()
}

fn earlier(roadmap: &Roadmap, entry: &TopicEntry) -> Vec<String> {
    named(
        roadmap
            .topics
            .iter()
            .take_while(|other| other.id != entry.id),
    )
}

fn later(roadmap: &Roadmap, entry: &TopicEntry) -> Vec<String> {
    named(
        roadmap
            .topics
            .iter()
            .skip_while(|other| other.id != entry.id)
            .skip(1),
    )
}

fn named<'a>(entries: impl Iterator<Item = &'a TopicEntry>) -> Vec<String> {
    entries
        .map(|other| format!("`{}` — {} (этап {})", other.id, other.title, other.stage))
        .collect()
}
