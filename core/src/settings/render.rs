use super::types::Settings;

pub const SCHEMA: &str = "tolearn/settings/v1";

pub fn text(settings: &Settings) -> String {
    format!(
        "schema: {SCHEMA}\n\
         disk_budget_mb: {}\n\
         locale: {}\n\
         theme: {}\n",
        settings.disk_budget_mb,
        settings.locale.label(),
        settings.theme.label(),
    )
}
