use super::types::{Document, Hit, Kind, Source};

const BEFORE: usize = 30;
const LENGTH: usize = 160;

pub fn hits(sources: &[Source], query: &str, limit: usize) -> Vec<Hit> {
    let tokens = tokens(query);
    if tokens.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<Hit> = sources
        .iter()
        .flat_map(|source| {
            source
                .documents
                .iter()
                .filter_map(|document| hit(&source.program, document, &tokens))
        })
        .collect();
    found.sort_by_key(|hit| hit.kind);
    found.truncate(limit);
    found
}

fn tokens(query: &str) -> Vec<Vec<char>> {
    query
        .split(|letter: char| !letter.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(lowered)
        .collect()
}

fn lowered(text: &str) -> Vec<char> {
    text.chars()
        .map(|letter| letter.to_lowercase().next().unwrap_or(letter))
        .collect()
}

fn hit(program: &str, document: &Document, tokens: &[Vec<char>]) -> Option<Hit> {
    let matched = match document.kind {
        Kind::Stage => &document.title,
        Kind::Block => &document.text,
    };
    let lower = lowered(matched);
    if !tokens.iter().all(|token| at(&lower, token).is_some()) {
        return None;
    }
    let snippet = match document.kind {
        Kind::Stage => String::new(),
        Kind::Block => snippet(&document.text, &lower, tokens),
    };
    Some(Hit {
        kind: document.kind,
        program: program.to_owned(),
        node: document.node.clone(),
        node_title: document.node_title.clone(),
        stage: document.stage.clone(),
        title: document.title.clone(),
        block: document.block.clone(),
        snippet,
    })
}

fn snippet(original: &str, lower: &[char], tokens: &[Vec<char>]) -> String {
    let from = tokens
        .iter()
        .filter_map(|token| at(lower, token))
        .min()
        .unwrap_or(0)
        .saturating_sub(BEFORE);
    let taken: String = original.chars().skip(from).take(LENGTH).collect();
    let words: Vec<&str> = taken.split_whitespace().collect();
    let mut out = words.join(" ");
    if from > 0 {
        out.insert(0, '…');
    }
    if original.chars().count() > from + LENGTH {
        out.push('…');
    }
    out
}

fn at(haystack: &[char], needle: &[char]) -> Option<usize> {
    if needle.len() > haystack.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
