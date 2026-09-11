use super::types::Settings;

pub const SCHEMA: &str = "tolearn/settings/v1";

pub fn text(settings: &Settings) -> String {
    format!(
        "schema: {SCHEMA}\n\
         disk_budget_mb: {}\n\
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
