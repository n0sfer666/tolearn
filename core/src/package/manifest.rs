use std::fmt::Write;

use super::SCHEMA;

pub(super) fn render(sums: &[(String, String)]) -> String {
    let files: Vec<String> = sums
        .iter()
        .map(|(name, sum)| format!("    {}: {}", quoted(name), quoted(sum)))
        .collect();
    format!(
        "{{\n  \"schema\": {},\n  \"files\": {{\n{}\n  }}\n}}\n",
        quoted(SCHEMA),
        files.join(",\n")
    )
}

fn quoted(text: &str) -> String {
    let mut out = String::from("\"");
    for symbol in text.chars() {
        match symbol {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            symbol if symbol < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(symbol));
            }
            symbol => out.push(symbol),
        }
    }
    out.push('"');
    out
}
