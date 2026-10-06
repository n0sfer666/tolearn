use super::types::{LOCALE, Settings, THEME};
use crate::yaml::{ParseError, read};

pub fn settings(source: &str) -> Result<Settings, ParseError> {
    read(source, |node| {
        Ok(Settings {
            disk_budget_mb: node.field("disk_budget_mb")?.number(1)?,
            locale: node.field("locale")?.choice("locale", &LOCALE)?,
            theme: node.field("theme")?.choice("theme", &THEME)?,
        })
    })
}
