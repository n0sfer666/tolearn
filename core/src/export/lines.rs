use super::escape::{opened, titled};

#[derive(Debug, Default)]
pub struct Doc {
    lines: Vec<String>,
}

impl Doc {
    pub fn heading(&mut self, level: usize, text: &str) {
        let text = titled(&flat(text));
        if text.is_empty() {
            return;
        }
        self.gap();
        self.lines.push(format!("{} {text}", "#".repeat(level)));
    }

    pub fn line(&mut self, text: &str) {
        self.gap();
        self.lines.push(opened(&flat(text)));
    }

    pub fn bullets(&mut self, items: &[String]) {
        self.raw(
            items
                .iter()
                .map(|item| format!("- {}", opened(item)))
                .collect(),
        );
    }

    pub fn numbered(&mut self, items: &[String]) {
        self.raw(
            items
                .iter()
                .enumerate()
                .map(|(place, item)| format!("{}. {}", place + 1, opened(item)))
                .collect(),
        );
    }

    pub fn raw(&mut self, lines: Vec<String>) {
        if lines.is_empty() {
            return;
        }
        self.gap();
        self.lines.extend(lines);
    }

    pub fn text(mut self) -> String {
        while self.lines.last().is_some_and(String::is_empty) {
            self.lines.pop();
        }
        let mut out = self.lines.join("\n");
        out.push('\n');
        out
    }

    fn gap(&mut self) {
        if self.lines.last().is_some_and(|line| !line.is_empty()) {
            self.lines.push(String::new());
        }
    }
}

pub fn flat(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut spanned = false;
    for word in text.split_whitespace() {
        out.push(if spanned {
            word.to_owned()
        } else {
            linked(word)
        });
        if word.matches('`').count() % 2 == 1 {
            spanned = !spanned;
        }
    }
    out.join(" ")
}

pub fn label(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

pub fn spanned(code: &str) -> String {
    let code = code.lines().collect::<Vec<&str>>().join(" ");
    let ticks = "`".repeat(longest(&code) + 1);
    let pad = if code.starts_with('`') || code.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{ticks}{pad}{code}{pad}{ticks}")
}

pub fn longest(text: &str) -> usize {
    text.split(|letter| letter != '`')
        .map(str::len)
        .max()
        .unwrap_or(0)
}

fn linked(word: &str) -> String {
    let Some(head) = ["http://", "https://"]
        .iter()
        .filter_map(|scheme| word.find(scheme))
        .min()
    else {
        return word.to_owned();
    };
    if word[..head].contains("](") || word[..head].contains('<') {
        return word.to_owned();
    }
    let bare = &word[head..];
    let url = bare.trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '»', '"', '\'']);
    format!("{}<{url}>{}", &word[..head], &bare[url.len()..])
}
