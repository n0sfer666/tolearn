pub fn last(text: &str) -> Option<&str> {
    fenced(text).or_else(|| braced(text))
}

fn fenced(text: &str) -> Option<&str> {
    let mut found = None;
    let mut opened: Option<usize> = None;
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let start = at;
        at += line.len();
        let head = line.trim();
        match opened {
            Some(from) if head.starts_with("```") => {
                found = Some(text[from..start].trim_matches('\n'));
                opened = None;
            }
            Some(_) => {}
            None if head.starts_with("```json") => opened = Some(at),
            None => {}
        }
    }
    found
}

fn braced(text: &str) -> Option<&str> {
    let mut found = None;
    let mut opened = None;
    let mut depth = 0_usize;
    let mut quoted = false;
    let mut escaped = false;
    for (at, symbol) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match symbol {
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '{' if !quoted => {
                if depth == 0 {
                    opened = Some(at);
                }
                depth += 1;
            }
            '}' if !quoted && depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    found = opened.map(|from| &text[from..=at]);
                }
            }
            _ => {}
        }
    }
    found
}
