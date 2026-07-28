use tolearn_core::settings::{Locale, Settings, SettingsError, Theme};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::SettingsView;

pub fn stored(context: &Context) -> Result<Settings, IpcError> {
    Settings::read(&context.settings()).map_err(failed)
}

pub fn view(settings: &Settings) -> SettingsView {
    SettingsView {
        disk_budget_mb: settings.disk_budget_mb,
        notes_directory: settings
            .notes_directory
            .as_ref()
            .map(|path| path.display().to_string()),
        locale: settings.locale.label().to_owned(),
        theme: settings.theme.label().to_owned(),
    }
}

pub fn taken(view: &SettingsView) -> Result<Settings, IpcError> {
    if view.disk_budget_mb == 0 {
        return Err(refused("бюджет диска", "0"));
    }
    Ok(Settings {
        disk_budget_mb: view.disk_budget_mb,
        notes_directory: view
            .notes_directory
            .as_ref()
            .filter(|path| !path.is_empty())
            .map(Into::into),
        locale: Locale::parse(&view.locale).ok_or_else(|| refused("язык", &view.locale))?,
        theme: Theme::parse(&view.theme).ok_or_else(|| refused("тема", &view.theme))?,
    })
}

pub fn failed(error: SettingsError) -> IpcError {
    let code = match error {
        SettingsError::Unwritable(_) => "settings.unwritable",
        SettingsError::Unreadable(_) | SettingsError::Malformed(_) => "settings.unreadable",
    };
    IpcError::new(code, error.to_string())
}

fn refused(what: &str, value: &str) -> IpcError {
    IpcError::new(
        "settings.unknown-value",
        format!("`{value}` — не значение для настройки «{what}»"),
    )
}
