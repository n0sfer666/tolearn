pub fn titled(text: &str) -> String {
    let text = text.trim_end_matches(['.', ',', ';', ':', '!', '。', '，', '；', '：', '！']);
    let bare = text.trim_end_matches('#');
    if bare.len() < text.len() && (bare.is_empty() || bare.ends_with([' ', '\t'])) {
        return format!("{bare}\\{}", &text[bare.len()..]);
    }
    text.to_owned()
}

pub fn opened(text: &str) -> String {
    if headed(text) || marked(text) || ruled(text) {
        return format!("\\{text}");
    }
    match numbered(text) {
        Some((digits, _)) => format!("{}\\{}", &text[..digits], &text[digits..]),
        None => text.to_owned(),
    }
}

pub fn headed(text: &str) -> bool {
    let rest = text.trim_start_matches('#');
    (1..=6).contains(&(text.len() - rest.len()))
        && (rest.is_empty() || rest.starts_with([' ', '\t']))
}

pub fn ruled(line: &str) -> bool {
    let marks: Vec<char> = line
        .chars()
        .filter(|letter| !letter.is_whitespace())
        .collect();
    marks.len() >= 3
        && ['-', '*', '_']
            .iter()
            .any(|mark| marks.iter().all(|letter| letter == mark))
}

pub fn numbered(text: &str) -> Option<(usize, u64)> {
    let rest = text.trim_start_matches(|letter: char| letter.is_ascii_digit());
    let digits = text.len() - rest.len();
    let after = rest.strip_prefix(['.', ')'])?;
    if !(1..=9).contains(&digits) || !(after.is_empty() || after.starts_with([' ', '\t'])) {
        return None;
    }
    Some((digits, text[..digits].parse().ok()?))
}

pub fn info(lang: Option<&str>) -> &str {
    lang.filter(|lang| {
        let mut letters = lang.chars();
        letters
            .next()
            .is_some_and(|first| first.is_ascii_lowercase())
            && letters.all(|letter| {
                letter.is_ascii_lowercase() || letter.is_ascii_digit() || "+#-".contains(letter)
            })
    })
    .unwrap_or("text")
}

pub fn local(path: &str) -> String {
    let mut out = String::new();
    for letter in path.chars() {
        if letter.is_control() || " ()<>[]\\`%#?\"^{}|".contains(letter) {
            let mut bytes = [0; 4];
            for byte in letter.encode_utf8(&mut bytes).bytes() {
                out.push_str(&format!("%{byte:02X}"));
            }
        } else {
            out.push(letter);
        }
    }
    out
}

fn marked(text: &str) -> bool {
    let bulleted = text.starts_with(['-', '+', '*'])
        && text[1..]
            .chars()
            .next()
            .is_none_or(|next| next == ' ' || next == '\t');
    bulleted || text.starts_with('>') || text.starts_with("```") || text.starts_with("~~~")
}
