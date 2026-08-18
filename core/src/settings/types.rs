use std::path::PathBuf;

pub const DEFAULT_BUDGET_MB: u32 = 2048;
pub const DEFAULT_HISTORY_DEPTH: u32 = 5;
pub const DEFAULT_HISTORY_SHARE: u32 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub disk_budget_mb: u32,
    pub notes_directory: Option<PathBuf>,
    pub locale: Locale,
    pub theme: Theme,
    pub history_depth: u32,
    pub history_share_percent: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            disk_budget_mb: DEFAULT_BUDGET_MB,
            notes_directory: None,
            locale: Locale::Ru,
            theme: Theme::System,
            history_depth: DEFAULT_HISTORY_DEPTH,
            history_share_percent: DEFAULT_HISTORY_SHARE,
        }
    }
}

impl Settings {
    pub fn budget_bytes(&self) -> u64 {
        u64::from(self.disk_budget_mb) * 1024 * 1024
    }

    pub fn history_bytes(&self) -> u64 {
        self.budget_bytes() * u64::from(self.history_share_percent) / 100
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
