use crate::Hours;
use crate::program::{Book, Tree};

use super::lines::{Doc, flat, label};
use super::words::Words;
use super::{file, folder};

pub fn page(tree: &Tree, parent: Option<&str>, words: &Words) -> String {
    let program = &tree.program;
    let mut doc = Doc::default();
    doc.heading(1, &program.title);
    if let Some(parent) = parent {
        doc.raw(vec![format!("← [{}](../index.md)", label(parent))]);
    }
    doc.bullets(&[
        format!("{}: {}", words.goal, flat(&program.goal)),
        format!("{}: {}", words.level, flat(&program.level)),
        format!("{}: {}", words.estimate, hours(program.map.hours(), words)),
    ]);
    let stages: Vec<String> = program
        .map
        .stages
        .iter()
        .enumerate()
        .map(|(place, row)| {
            let target = tree
                .stages
                .contains_key(&row.id)
                .then(|| file(place, &row.id));
            entry(&row.title, target, row.hours, words)
        })
        .collect();
    listed(&mut doc, words.stages, &stages);
    let children: Vec<String> = program
        .map
        .children
        .iter()
        .enumerate()
        .map(|(place, row)| {
            let target = tree
                .children
                .get(&row.uuid)
                .map(|child| format!("{}/index.md", folder(place, &child.program.slug)));
            entry(&row.title, target, row.hours, words)
        })
        .collect();
    listed(&mut doc, words.children, &children);
    sources(&mut doc, tree, words);
    doc.text()
}

fn listed(doc: &mut Doc, title: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    doc.heading(2, title);
    doc.numbered(items);
}

fn entry(title: &str, target: Option<String>, span: Hours, words: &Words) -> String {
    let span = hours(span, words);
    match target {
        Some(target) => format!("[{}]({target}) — {span}", label(title)),
        None => format!("{} — {span}, {}", flat(title), words.pending),
    }
}

fn hours(span: Hours, words: &Words) -> String {
    if span.min == span.max {
        format!("{} {}", span.min, words.hour)
    } else {
        format!("{}–{} {}", span.min, span.max, words.hour)
    }
}

fn sources(doc: &mut Doc, tree: &Tree, words: &Words) {
    let sources = &tree.program.sources;
    let mut items: Vec<String> = sources
        .books
        .iter()
        .map(|found| book(found, words))
        .collect();
    items.extend(sources.pages.iter().map(|page| {
        joined(&[
            format!("[{}]({})", label(&page.title), address(&page.url)),
            checked(&page.checked_at, words),
        ])
    }));
    if items.is_empty() {
        return;
    }
    doc.heading(2, words.sources);
    doc.bullets(&items);
}

fn book(book: &Book, words: &Words) -> String {
    let lead = if book.authors.is_empty() {
        book.title.clone()
    } else {
        format!("{} — {}", book.authors.join(", "), book.title)
    };
    let isbn = if book.isbn.is_empty() {
        String::new()
    } else {
        format!("ISBN {}", book.isbn)
    };
    joined(&[
        flat(&lead),
        flat(&book.chapter),
        isbn,
        checked(&book.checked_at, words),
    ])
}

fn checked(date: &str, words: &Words) -> String {
    if date.is_empty() {
        String::new()
    } else {
        format!("{} {date}", words.checked)
    }
}

fn joined(parts: &[String]) -> String {
    parts
        .iter()
        .filter(|part| !part.is_empty())
        .cloned()
        .collect::<Vec<String>>()
        .join(", ")
}

fn address(url: &str) -> String {
    let url = url.replace('<', "%3C").replace('>', "%3E");
    if url.contains([' ', '(', ')']) {
        format!("<{url}>")
    } else {
        url
    }
}
