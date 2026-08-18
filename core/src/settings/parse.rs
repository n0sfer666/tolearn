use std::path::PathBuf;

use super::types::{DEFAULT_HISTORY_DEPTH, DEFAULT_HISTORY_SHARE, LOCALE, Settings, THEME};
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
            history_depth: match node.optional_field("history_depth")? {
                Some(field) => field.number(0)?,
                None => DEFAULT_HISTORY_DEPTH,
            },
            history_share_percent: match node.optional_field("history_share_percent")? {
                Some(field) => field.bounded(0, 100)?,
                None => DEFAULT_HISTORY_SHARE,
            },
        })
    })
}
