use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Anchors {
    taken: BTreeMap<String, u32>,
}

impl Anchors {
    pub fn take(&mut self, heading: &str) -> String {
        let base = slug(heading);
        let seen = self.taken.entry(base.clone()).or_insert(0);
        *seen += 1;
        if *seen == 1 {
            return base;
        }
        format!("{base}-{}", *seen - 1)
    }
}

fn slug(heading: &str) -> String {
    let mut out = String::new();
    for letter in heading.chars() {
        if letter.is_alphanumeric() || letter == '_' {
            out.extend(letter.to_lowercase());
        } else if letter.is_whitespace() || letter == '-' {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_owned();
    if trimmed.is_empty() {
        return "section".to_owned();
    }
    trimmed
}
