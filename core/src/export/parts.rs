use crate::topic::{Check, Liveness, Material, Practice, Topic};

use super::lines::Doc;

pub fn materials(doc: &mut Doc, topic: &Topic) {
    if topic.materials.is_empty() {
        return;
    }
    doc.heading(4, "Материалы");
    let items: Vec<String> = topic.materials.iter().map(material).collect();
    doc.bullets(&items);
}

fn material(material: &Material) -> String {
    let mut said = format!(
        "[{}]({})",
        plain(material.title.trim()),
        material.url.trim()
    );
    said.push_str(&format!(
        " — {}, {}, {}",
        material.kind.label(),
        material.tier.label(),
        material.lang
    ));
    if material.liveness != Liveness::Ok {
        said.push_str(&format!(", доступ: {}", material.liveness.label()));
    }
    if let Some(version) = &material.covers_version {
        said.push_str(&format!(", покрывает {version}"));
    }
    if material.stale {
        said.push_str(", устарел");
    }
    if let Some(delta) = &material.delta {
        said.push_str(&format!(" ({delta})"));
    }
    if !material.note.is_empty() {
        said.push_str(&format!(". {}", material.note));
    }
    said
}

fn plain(title: &str) -> String {
    title
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

pub fn practice(doc: &mut Doc, topic: &Topic) {
    let practice = &topic.practice;
    doc.heading(4, "Практика");
    doc.line(&practice.task);
    doc.bullets(&about(practice));
    checks(doc, "Ограничения", &practice.constraints);
    checks(doc, "Приёмка", &practice.acceptance);
    if let Some(fallback) = &practice.fallback {
        doc.line(&format!("Подсказка: {fallback}"));
    }
}

fn about(practice: &Practice) -> Vec<String> {
    let mut items = vec![
        format!("Результат: {}", practice.deliverable),
        format!(
            "Вид: {}, уровень: {}",
            practice.kind.label(),
            practice.tier.label()
        ),
        format!("Таймбокс: {} мин", practice.time_box_min),
    ];
    if let Some(start) = &practice.starting_point {
        items.push(format!("Отправная точка: {start}"));
    }
    items
}

fn checks(doc: &mut Doc, title: &str, checks: &[Check]) {
    if checks.is_empty() {
        return;
    }
    doc.heading(5, title);
    let items: Vec<String> = checks
        .iter()
        .map(|check| {
            format!(
                "{} — проверка: `{}`, ожидается: {}",
                check.claim.trim(),
                check.check.trim(),
                check.expect.trim()
            )
        })
        .collect();
    doc.bullets(&items);
}
