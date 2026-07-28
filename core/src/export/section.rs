use std::collections::BTreeMap;

use crate::notes::Note;
use crate::progress::Status;
use crate::roadmap::TopicEntry;
use crate::topic::Topic;

use super::lines::Doc;
use super::{Link, ask, note, parts};

pub struct Sight<'a> {
    pub links: &'a BTreeMap<String, Link>,
    pub status: Option<Status>,
    pub notes: &'a [Note],
}

pub fn section(doc: &mut Doc, entry: &TopicEntry, document: Option<&Topic>, sight: &Sight<'_>) {
    doc.heading(3, &entry.title);
    doc.bullets(&about(entry, document, sight));
    let Some(topic) = document else {
        doc.line("_Тема ещё не сгенерирована: в бандле нет её файла._");
        return;
    };
    listed(doc, "Чему научит", &topic.outcomes);
    listed(doc, "Заблуждения", &topic.misconceptions);
    listed(doc, "Версии", &topic.version_context);
    parts::materials(doc, topic);
    parts::practice(doc, topic);
    let shown = ask::open(sight.status);
    ask::questions(doc, topic, shown);
    ask::exam(doc, topic, shown);
    note::note(doc, sight.notes, &topic.id);
}

fn about(entry: &TopicEntry, document: Option<&Topic>, sight: &Sight<'_>) -> Vec<String> {
    let mut items = vec![
        format!(
            "Статус: {}",
            sight.status.map_or("неизвестен", |status| status.label())
        ),
        format!("Оценка: {}–{} ч", entry.est_hours.min, entry.est_hours.max),
        format!("Приоритет: {}", entry.priority.label()),
    ];
    let Some(topic) = document else {
        return items;
    };
    items.push(format!(
        "Проверено {}, пересмотр через {} дн., волатильность: {}",
        topic.verified_at,
        topic.revalidate_after_days,
        topic.volatility.label()
    ));
    items.push(format!(
        "Уверенность: {}, удержание: {}",
        topic.confidence.label(),
        topic.retention.label()
    ));
    if !topic.depends_on.is_empty() {
        items.push(format!("Зависит от: {}", depends(topic, sight.links)));
    }
    items
}

fn depends(topic: &Topic, links: &BTreeMap<String, Link>) -> String {
    topic
        .depends_on
        .iter()
        .map(|id| match links.get(id) {
            Some(link) => format!("[{}](#{})", link.title, link.anchor),
            None => id.clone(),
        })
        .collect::<Vec<String>>()
        .join(", ")
}

fn listed(doc: &mut Doc, title: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    doc.heading(4, title);
    doc.bullets(items);
}
