use std::path::PathBuf;

pub const DEFAULT_BUDGET_MB: u32 = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub disk_budget_mb: u32,
    pub notes_directory: Option<PathBuf>,
    pub locale: Locale,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            disk_budget_mb: DEFAULT_BUDGET_MB,
            notes_directory: None,
            locale: Locale::Ru,
            theme: Theme::System,
        }
    }
}

impl Settings {
    pub fn budget_bytes(&self) -> u64 {
        u64::from(self.disk_budget_mb) * 1024 * 1024
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Ru,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    System,
    Light,
    Dark,
}

pub(super) const LOCALE: [(&str, Locale); 2] = [("ru", Locale::Ru), ("en", Locale::En)];

pub(super) const THEME: [(&str, Theme); 3] = [
    ("system", Theme::System),
    ("light", Theme::Light),
    ("dark", Theme::Dark),
];

impl Locale {
    pub fn label(self) -> &'static str {
        LOCALE
            .iter()
            .find(|(_, locale)| *locale == self)
            .map_or("ru", |(label, _)| label)
    }

    pub fn parse(label: &str) -> Option<Self> {
        LOCALE
            .iter()
            .find(|(known, _)| *known == label)
            .map(|(_, locale)| *locale)
    }
}

impl Theme {
    pub fn label(self) -> &'static str {
        THEME
            .iter()
            .find(|(_, theme)| *theme == self)
            .map_or("system", |(label, _)| label)
    }

    pub fn parse(label: &str) -> Option<Self> {
        THEME
            .iter()
            .find(|(known, _)| *known == label)
            .map(|(_, theme)| *theme)
    }
}
