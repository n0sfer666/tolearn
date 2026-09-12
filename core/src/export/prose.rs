use super::escape::{headed, info, numbered, ruled, titled};
use super::lines::{flat, longest};

struct Fence {
    mark: char,
    size: usize,
    open: usize,
    info: String,
}

#[derive(Default)]
struct Prose {
    lines: Vec<String>,
    fence: Option<Fence>,
    listed: bool,
}

pub fn prose(text: &str, depth: usize) -> Vec<String> {
    let mut prose = Prose::default();
    for line in text.lines() {
        prose.take(line, depth);
    }
    prose.close();
    while prose.lines.last().is_some_and(String::is_empty) {
        prose.lines.pop();
    }
    prose.lines
}

impl Prose {
    fn take(&mut self, line: &str, depth: usize) {
        if let Some(fence) = &self.fence {
            let bare = line.trim();
            if bare.len() >= fence.size && bare.chars().all(|letter| letter == fence.mark) {
                self.close();
                self.gap();
            } else {
                self.lines.push(line.trim_end().to_owned());
            }
            return;
        }
        if self.opened(line.trim_start()) {
            return;
        }
        let line = flat(line);
        let after = self.lines.last().is_some_and(|last| !last.is_empty());
        if line.is_empty() || headed(&line) || ruled(&line) {
            self.gap();
            self.listed = false;
            self.block(&line, depth);
            return;
        }
        if line.chars().all(|letter| letter == '=' || letter == '-') {
            self.lines.push(format!("\\{line}"));
            return;
        }
        let item = line
            .strip_prefix(['-', '*', '+'])
            .and_then(|rest| rest.strip_prefix(' '));
        let ordered = numbered(&line);
        let starts = item.is_some() || ordered.is_some_and(|(_, number)| number == 1);
        if !self.listed && ordered.is_some() && !starts {
            let digits = ordered.map_or(0, |(digits, _)| digits);
            self.lines
                .push(format!("{}\\{}", &line[..digits], &line[digits..]));
            return;
        }
        if starts && after && !self.listed {
            self.gap();
        }
        self.listed |= starts;
        self.lines.push(match item {
            Some(rest) => format!("- {rest}"),
            None => line,
        });
    }

    fn block(&mut self, line: &str, depth: usize) {
        if headed(line) {
            let title = titled(
                line.trim_start_matches('#')
                    .trim()
                    .trim_end_matches('#')
                    .trim_end(),
            );
            if !title.is_empty() {
                self.lines.push(format!("{} {title}", "#".repeat(depth)));
            }
        } else if ruled(line) {
            self.lines.push("---".to_owned());
        }
        self.gap();
    }

    fn opened(&mut self, line: &str) -> bool {
        let Some(mark) = line
            .chars()
            .next()
            .filter(|mark| *mark == '`' || *mark == '~')
        else {
            return false;
        };
        let rest = line.trim_start_matches(mark);
        let size = line.len() - rest.len();
        if size < 3 || (mark == '`' && rest.contains('`')) {
            return false;
        }
        self.gap();
        self.listed = false;
        self.fence = Some(Fence {
            mark,
            size,
            open: self.lines.len(),
            info: info(Some(rest.trim())).to_owned(),
        });
        self.lines.push(String::new());
        true
    }

    fn close(&mut self) {
        let Some(fence) = self.fence.take() else {
            return;
        };
        let inner = self.lines[fence.open + 1..].join("\n");
        let ticks = "`".repeat(longest(&inner).max(2) + 1);
        self.lines[fence.open] = format!("{ticks}{}", fence.info);
        self.lines.push(ticks);
    }

    fn gap(&mut self) {
        if self.lines.last().is_some_and(|last| !last.is_empty()) {
            self.lines.push(String::new());
        }
    }
}
