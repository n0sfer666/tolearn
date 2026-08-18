use super::types::{Document, Source};

pub const SCHEMA: &str = "tolearn/search/v1";

pub fn text(sources: &[Source]) -> String {
    let mut out = format!("schema: {SCHEMA}\n");
    if sources.is_empty() {
        out.push_str("sources: []\n");
        return out;
    }
    out.push_str("sources:\n");
    for source in sources {
        out.push_str(&format!(
            "  - path: {}\n",
            quoted(&source.path.to_string_lossy())
        ));
        out.push_str(&format!("    roadmap: {}\n", quoted(&source.roadmap)));
        out.push_str(&format!(
            "    modified: {}\n",
            quoted(&source.stamp.modified_nanos.to_string())
        ));
        out.push_str(&format!(
            "    size: {}\n",
            quoted(&source.stamp.size.to_string())
        ));
        out.push_str(&documents(&source.documents));
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
        out.push_str(&format!("        topic: {}\n", quoted(&document.topic)));
        out.push_str(&format!("        title: {}\n", quoted(&document.title)));
        out.push_str(&format!("        text: {}\n", quoted(&document.text)));
    }
    out
}

fn quoted(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}
