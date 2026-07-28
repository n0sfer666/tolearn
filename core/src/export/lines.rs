#[derive(Debug, Default)]
pub struct Doc {
    lines: Vec<String>,
}

impl Doc {
    pub fn heading(&mut self, level: usize, text: &str) {
        self.gap();
        self.lines
            .push(format!("{} {}", "#".repeat(level), flat(text)));
    }

    pub fn line(&mut self, text: &str) {
        self.gap();
        self.lines.push(flat(text));
    }

    pub fn bullets(&mut self, items: &[String]) {
        if items.is_empty() {
            return;
        }
        self.gap();
        for item in items {
            self.lines.push(format!("- {}", flat(item)));
        }
    }

    pub fn numbered(&mut self, items: &[String]) {
        if items.is_empty() {
            return;
        }
        self.gap();
        for (place, item) in items.iter().enumerate() {
            self.lines.push(format!("{}. {}", place + 1, flat(item)));
        }
    }

    pub fn block(&mut self, text: &str) {
        self.gap();
        for line in text.lines() {
            self.lines.push(line.trim_end().to_owned());
        }
    }

    pub fn text(self) -> String {
        let mut out = String::new();
        let mut blank = true;
        for line in &self.lines {
            if line.is_empty() && blank {
                continue;
            }
            blank = line.is_empty();
            out.push_str(line);
            out.push('\n');
        }
        while out.ends_with("\n\n") {
            out.pop();
        }
        out
    }

    fn gap(&mut self) {
        if !self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }
}

fn flat(text: &str) -> String {
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

fn linked(word: &str) -> String {
    let Some(head) = word.find("http://").or_else(|| word.find("https://")) else {
        return word.to_owned();
    };
    if word[..head].contains("](") || word[..head].contains('<') {
        return word.to_owned();
    }
    let bare = &word[head..];
    let url = bare.trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '»', '"', '\'']);
    format!("{}<{url}>{}", &word[..head], &bare[url.len()..])
}
