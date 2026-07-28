use crate::roadmap::Roadmap;

use super::Chapter;
use super::lines::Doc;

pub fn head(doc: &mut Doc, roadmap: &Roadmap, chapters: &[Chapter]) {
    doc.heading(1, &roadmap.title);
    doc.line(&roadmap.goal);
    doc.bullets(&about(roadmap));
    doc.heading(2, "Оглавление");
    doc.block(&contents(chapters));
}

fn about(roadmap: &Roadmap) -> Vec<String> {
    let mut items = vec![
        format!("Предмет: {}", roadmap.subject),
        format!(
            "Сгенерировано {} — {}",
            roadmap.generated_at, roadmap.generated_by
        ),
        format!("Норма: {} ч в неделю", roadmap.weekly_hours),
    ];
    if !roadmap.env_constraints.is_empty() {
        items.push(format!("Окружение: {}", roadmap.env_constraints.join("; ")));
    }
    if !roadmap.version_pins.is_empty() {
        let pins: Vec<String> = roadmap
            .version_pins
            .iter()
            .map(|(name, version)| format!("{name} {version}"))
            .collect();
        items.push(format!("Версии: {}", pins.join(", ")));
    }
    items
}

fn contents(chapters: &[Chapter]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for chapter in chapters {
        lines.push(format!("- [{}](#{})", chapter.title, chapter.anchor));
        for placed in &chapter.topics {
            lines.push(format!("  - [{}](#{})", placed.title, placed.anchor));
        }
    }
    lines.join("\n")
}
