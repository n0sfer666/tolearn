use std::iter::Peekable;
use std::str::Chars;

const BELL: char = '\u{7}';
const ESCAPE: char = '\u{1b}';

pub fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text.chars().peekable();
    while let Some(letter) = rest.next() {
        match letter {
            ESCAPE => escaped(&mut rest),
            _ => out.push(letter),
        }
    }
    out
}

fn escaped(rest: &mut Peekable<Chars<'_>>) {
    match rest.next() {
        Some('[') => skip(rest, |letter| ('\u{40}'..='\u{7e}').contains(&letter)),
        Some(']') => title(rest),
        _ => {}
    }
}

fn title(rest: &mut Peekable<Chars<'_>>) {
    skip(rest, |letter| letter == BELL || letter == ESCAPE);
    if rest.peek() == Some(&'\\') {
        rest.next();
    }
}

fn skip(rest: &mut Peekable<Chars<'_>>, ends: impl Fn(char) -> bool) {
    for letter in rest.by_ref() {
        if ends(letter) {
            break;
        }
    }
}
