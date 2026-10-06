pub(super) fn plain(html: &str) -> String {
    let mut text = String::new();
    let mut inside = false;
    for symbol in html.chars() {
        match symbol {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => text.push(symbol),
            _ => {}
        }
    }
    decode(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        let known = tail
            .find(';')
            .and_then(|end| entity(&tail[1..end]).map(|symbol| (symbol, end)));
        match known {
            Some((symbol, end)) => {
                out.push(symbol);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ => {
            let number = name.strip_prefix('#')?;
            let code = match number.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}
