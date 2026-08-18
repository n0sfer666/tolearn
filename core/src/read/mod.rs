mod types;

pub use types::{Block, Kind};

const FENCE: &str = "```";
const BULLETS: [&str; 3] = ["- ", "* ", "+ "];
const DEEPEST: usize = 6;

pub fn blocks(text: &str) -> Vec<Block> {
    let mut made: Vec<Block> = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let mut code: Option<Vec<String>> = None;

    for line in text.lines() {
        if let Some(kept) = code.as_mut() {
            if fenced(line) {
                made.push(Block::new(Kind::Code, 0, kept.join("\n")));
                code = None;
            } else {
                kept.push(line.to_string());
            }
            continue;
        }
        if fenced(line) {
            flush(&mut para, &mut made);
            code = Some(Vec::new());
            continue;
        }
        let bare = line.trim();
        if bare.is_empty() {
            flush(&mut para, &mut made);
            continue;
        }
        let Some(block) = marked(bare) else {
            para.push(bare.to_string());
            continue;
        };
        flush(&mut para, &mut made);
        made.push(block);
    }

    flush(&mut para, &mut made);
    if let Some(kept) = code {
        made.push(Block::new(Kind::Code, 0, kept.join("\n")));
    }
    made
}

fn fenced(line: &str) -> bool {
    line.trim_start().starts_with(FENCE)
}

fn flush(para: &mut Vec<String>, made: &mut Vec<Block>) {
    if para.is_empty() {
        return;
    }
    made.push(Block::new(Kind::Paragraph, 0, para.join(" ")));
    para.clear();
}

fn marked(line: &str) -> Option<Block> {
    if let Some(block) = heading(line) {
        return Some(block);
    }
    if let Some(rest) = line.strip_prefix("> ") {
        return Some(Block::new(Kind::Quote, 0, rest.trim().to_string()));
    }
    item(line)
}

fn heading(line: &str) -> Option<Block> {
    let hashes = line.chars().take_while(|sign| *sign == '#').count();
    if hashes == 0 || hashes > DEEPEST {
        return None;
    }
    let rest = line.get(hashes..)?.strip_prefix(' ')?;
    let level = u32::try_from(hashes).unwrap_or(1);
    Some(Block::new(Kind::Heading, level, rest.trim().to_string()))
}

fn item(line: &str) -> Option<Block> {
    if let Some(rest) = BULLETS.iter().find_map(|mark| line.strip_prefix(mark)) {
        return Some(Block::new(Kind::Item, 0, rest.trim().to_string()));
    }
    numbered(line)
}

fn numbered(line: &str) -> Option<Block> {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let rest = line.get(digits..)?.strip_prefix(". ")?;
    Some(Block::new(Kind::Item, 0, rest.trim().to_string()))
}
