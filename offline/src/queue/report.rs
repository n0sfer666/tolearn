use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    Paywall,
    LoginRequired,
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub saved: Vec<String>,
    pub skipped: Vec<(String, Skip)>,
    pub failed: Vec<(String, String)>,
    pub bytes: u64,
    pub cancelled: bool,
}

impl Skip {
    pub fn name(self) -> &'static str {
        match self {
            Self::Paywall => "paywall",
            Self::LoginRequired => "login_required",
        }
    }
}

impl Report {
    pub fn failed_urls(&self) -> Vec<&str> {
        self.failed.iter().map(|(url, _)| url.as_str()).collect()
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Скачано     {}", self.saved.len())?;
        writeln!(
            f,
            "Пропущено   {}   {}",
            self.skipped.len(),
            tally(self.skipped.iter().map(|(_, why)| why.name()))
        )?;
        write!(
            f,
            "Провалено   {}   {}",
            self.failed.len(),
            tally(self.failed.iter().map(|(_, why)| why.as_str()))
        )
    }
}

fn tally<'a>(reasons: impl Iterator<Item = &'a str>) -> String {
    let mut counted: BTreeMap<&str, usize> = BTreeMap::new();
    for reason in reasons {
        *counted.entry(reason).or_default() += 1;
    }
    counted
        .into_iter()
        .map(|(reason, times)| format!("{reason} ({times})"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn одинаковые_причины_складываются() {
        assert_eq!(
            tally(["таймаут", "404", "таймаут"].into_iter()),
            "404 (1), таймаут (2)"
        );
    }

    #[test]
    fn без_причин_разбивка_пустая() {
        assert_eq!(tally([].into_iter()), "");
    }
}
