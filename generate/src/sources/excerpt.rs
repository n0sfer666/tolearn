pub const PAGE_CHARS: usize = 8_000;

pub fn excerpt(text: &str) -> &str {
    match text.char_indices().nth(PAGE_CHARS) {
        Some((end, _)) => &text[..end],
        None => text,
    }
}
