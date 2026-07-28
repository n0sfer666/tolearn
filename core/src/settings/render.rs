use super::types::Settings;

pub const SCHEMA: &str = "tolearn/settings/v1";

pub fn text(settings: &Settings) -> String {
    let directory = settings
        .notes_directory
        .as_ref()
        .map_or_else(|| "null".to_owned(), |path| quoted(&path.to_string_lossy()));
    format!(
        "schema: {SCHEMA}\n\
         disk_budget_mb: {}\n\
         notes_directory: {directory}\n\
         locale: {}\n\
         theme: {}\n",
        settings.disk_budget_mb,
        settings.locale.label(),
        settings.theme.label(),
    )
}

fn quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
