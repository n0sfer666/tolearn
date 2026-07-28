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
         theme: {}\n\
         history_depth: {}\n\
         history_share_percent: {}\n",
        settings.disk_budget_mb,
        settings.locale.label(),
        settings.theme.label(),
        settings.history_depth,
        settings.history_share_percent,
    )
}

fn quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
