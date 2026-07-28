use std::path::PathBuf;

use super::types::{LOCALE, Settings, THEME};
use crate::yaml::{ParseError, read};

pub fn settings(source: &str) -> Result<Settings, ParseError> {
    read(source, |node| {
        Ok(Settings {
            disk_budget_mb: node.field("disk_budget_mb")?.number(1)?,
            notes_directory: node
                .field("notes_directory")?
                .optional_text()?
                .map(PathBuf::from),
            locale: node.field("locale")?.choice("locale", &LOCALE)?,
            theme: node.field("theme")?.choice("theme", &THEME)?,
        })
    })
}
