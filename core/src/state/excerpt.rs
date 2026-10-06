pub const EXCERPT_CHARS: usize = 80;

pub fn excerpt(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(EXCERPT_CHARS)
        .collect()
}
