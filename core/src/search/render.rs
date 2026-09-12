use super::types::{Document, Seen, Source};

pub const SCHEMA: &str = "tolearn/search/v2";
pub const BUILD: &str = env!("CARGO_PKG_VERSION");

pub fn text(sources: &[Source]) -> String {
    let mut out = format!("schema: {SCHEMA}\nbuild: {}\n", quoted(BUILD));
    if sources.is_empty() {
        out.push_str("sources: []\n");
        return out;
    }
    out.push_str("sources:\n");
    for source in sources {
        out.push_str(&format!("  - program: {}\n", quoted(&source.program)));
        out.push_str(&files(&source.files));
        out.push_str(&documents(&source.documents));
    }
    out
}

fn files(files: &[Seen]) -> String {
    if files.is_empty() {
        return "    files: []\n".to_owned();
    }
    let mut out = "    files:\n".to_owned();
    for file in files {
        out.push_str(&format!("      - name: {}\n", quoted(&file.name)));
        let modified = file.stamp.modified_nanos.to_string();
        out.push_str(&format!("        modified: {}\n", quoted(&modified)));
        let size = file.stamp.size.to_string();
        out.push_str(&format!("        size: {}\n", quoted(&size)));
    }
    out
}

fn documents(documents: &[Document]) -> String {
    if documents.is_empty() {
        return "    documents: []\n".to_owned();
    }
    let mut out = "    documents:\n".to_owned();
    for document in documents {
        out.push_str(&format!("      - kind: {}\n", document.kind.label()));
        for (key, value) in [
            ("node", &document.node),
            ("node_title", &document.node_title),
            ("stage", &document.stage),
            ("title", &document.title),
            ("block", &document.block),
            ("text", &document.text),
        ] {
            out.push_str(&format!("        {key}: {}\n", quoted(value)));
        }
    }
    out
}

fn quoted(value: &str) -> String {
    let mut out = String::from('"');
    for letter in value.chars() {
        match letter {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            letter if letter.is_control() => {
                out.push_str(&format!("\\u{:04X}", u32::from(letter)));
            }
            letter => out.push(letter),
        }
    }
    out.push('"');
    out
}
