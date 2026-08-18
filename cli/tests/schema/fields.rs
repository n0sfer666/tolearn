use crate::repo::read;

pub const ROLES: [&str; 3] = ["рендер", "логика", "игнор"];

const SECTIONS: [(&str, &str); 3] = [
    ("## roadmap.yaml", "roadmap"),
    ("## topics/*.yaml", "topic"),
    ("## progress.yaml", "progress"),
];

pub struct Row {
    pub kind: String,
    pub path: String,
    pub role: String,
}

pub fn rows() -> Vec<Row> {
    let text = read("docs/fields.md");
    let mut kind: Option<&str> = None;
    let mut rows = Vec::new();
    for line in text.lines() {
        if let Some(section) = SECTIONS.iter().find(|(heading, _)| line.trim() == *heading) {
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
    assert!(!rows.is_empty(), "docs/fields.md: no field rows found");
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
