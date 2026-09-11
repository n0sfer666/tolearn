use crate::repo::read;

pub const ROLES: [&str; 3] = ["рендер", "логика", "игнор"];

const LISTS: [(&str, &[(&str, &str)]); 2] = [
    (
        "docs/fields.md",
        &[
            ("## roadmap.yaml", "roadmap"),
            ("## topics/*.yaml", "topic"),
            ("## progress.yaml", "progress"),
        ],
    ),
    (
        "docs/format.md",
        &[
            ("## program.yaml", "program"),
            ("## stages/*.yaml", "stage"),
        ],
    ),
];

pub struct Row {
    pub kind: String,
    pub path: String,
    pub role: String,
}

pub fn list_of(kind: &str) -> &'static str {
    LISTS
        .iter()
        .find(|(_, sections)| sections.iter().any(|(_, listed)| *listed == kind))
        .map(|(doc, _)| *doc)
        .unwrap_or_else(|| panic!("no field list classifies `{kind}`"))
}

pub fn rows() -> Vec<Row> {
    LISTS
        .iter()
        .flat_map(|(doc, sections)| rows_in(doc, sections))
        .collect()
}

fn rows_in(doc: &str, sections: &[(&str, &str)]) -> Vec<Row> {
    let text = read(doc);
    let mut kind: Option<&str> = None;
    let mut rows = Vec::new();
    for line in text.lines() {
        if let Some(section) = sections.iter().find(|(heading, _)| line.trim() == *heading) {
            kind = Some(section.1);
            continue;
        }
        if line.starts_with("## ") {
            kind = None;
            continue;
        }
        let Some(kind) = kind else { continue };
        let Some(cells) = table_row(line) else {
            continue;
        };
        rows.push(Row {
            kind: kind.to_owned(),
            path: cells.0,
            role: cells.1,
        });
    }
    assert!(!rows.is_empty(), "{doc}: no field rows found");
    rows
}

fn table_row(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if !line.starts_with('|') {
        return None;
    }
    let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
    if cells.len() < 3 || cells[0].starts_with("---") || !cells[0].starts_with('`') {
        return None;
    }
    Some((cells[0].trim_matches('`').to_owned(), cells[1].to_owned()))
}
