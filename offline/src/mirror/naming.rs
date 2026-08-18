use url::Url;

use crate::digest::digest;

const KEEP: usize = 100;
const MARK: usize = 12;

pub(super) fn slugged(text: &str, spare: &str) -> String {
    let mapped: String = text
        .chars()
        .map(|letter| {
            if letter.is_alphanumeric() {
                letter
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = mapped.trim_matches('-');
    if trimmed.is_empty() {
        spare.to_string()
    } else {
        trimmed.to_string()
    }
}

pub(super) fn fitted(slug: &str, url: &Url) -> String {
    if slug.len() <= KEEP {
        return slug.to_string();
    }
    let edge = (0..=KEEP)
        .rev()
        .find(|at| slug.is_char_boundary(*at))
        .unwrap_or(0);
    let head = slug[..edge].trim_end_matches('-');
    format!("{head}-{}", &digest(url.as_str().as_bytes())[..MARK])
}
