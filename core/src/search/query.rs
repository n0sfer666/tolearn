use super::types::{Document, Hit, Source};

const TITLE_WEIGHT: u32 = 3;
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
                .filter_map(|document| hit(&source.roadmap, document, &tokens))
        })
        .collect();
    found.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then(left.kind.cmp(&right.kind))
            .then(left.topic.cmp(&right.topic))
            .then(left.title.cmp(&right.title))
    });
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

fn hit(roadmap: &str, document: &Document, tokens: &[Vec<char>]) -> Option<Hit> {
    let title = lowered(&document.title);
    let text = lowered(&document.text);
    let mut score = 0;
    for token in tokens {
        match (at(&title, token), at(&text, token)) {
            (Some(_), _) => score += TITLE_WEIGHT,
            (None, Some(_)) => score += 1,
            (None, None) => return None,
        }
    }

    Some(Hit {
        kind: document.kind,
        roadmap: roadmap.to_owned(),
        topic: document.topic.clone(),
        title: document.title.clone(),
        snippet: snippet(&document.text, &text, tokens),
        score,
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
