use super::types::Program;

pub const SCHEMA: &str = "tolearn/registry/v1";

pub fn text(programs: &[Program]) -> String {
    let mut out = format!("schema: {SCHEMA}\n");
    if programs.is_empty() {
        out.push_str("programs: []\n");
        return out;
    }
    out.push_str("programs:\n");
    for program in programs {
        out.push_str(&format!("  - id: {}\n", quoted(&program.id)));
        out.push_str(&format!("    title: {}\n", quoted(&program.title)));
        out.push_str(&format!(
            "    path: {}\n",
            quoted(&program.path.to_string_lossy())
        ));
        let opened = program
            .opened_at
            .as_deref()
            .map_or_else(|| "null".to_owned(), |moment: &str| quoted(moment));
        out.push_str(&format!("    opened_at: {opened}\n"));
    }
    out
}

fn quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
