use crate::notes::Note;

use super::lines::Doc;

const UNDER: usize = 4;

pub fn note(doc: &mut Doc, notes: &[Note], topic: &str) {
    let Some(found) = notes.iter().find(|note| note.topic == topic) else {
        return;
    };
    let body = deepened(&found.body);
    if body.trim().is_empty() {
        return;
    }
    doc.heading(UNDER, "Конспект");
    doc.block(&body);
}

fn deepened(body: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut fenced = false;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            out.push(line.to_owned());
            continue;
        }
        if fenced {
            out.push(line.to_owned());
            continue;
        }
        out.push(match hashes(trimmed) {
            Some(depth) => format!(
                "{} {}",
                "#".repeat((UNDER + depth).min(6)),
                trimmed[depth..].trim()
            ),
            None => line.to_owned(),
        });
    }
    out.join("\n")
}

fn hashes(line: &str) -> Option<usize> {
    let depth = line.chars().take_while(|letter| *letter == '#').count();
    if depth == 0 || depth > 6 {
        return None;
    }
    line[depth..].starts_with(' ').then_some(depth)
}
