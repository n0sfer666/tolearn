pub const WINDOW: i64 = 6 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mark {
    pub saved: bool,
    pub checkable: bool,
    pub checked_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Missing,
    Unchecked,
    Stale,
    Fresh(i64),
}

pub fn freshness(marks: &[Mark], now: i64, window: i64) -> Freshness {
    if marks.is_empty() || marks.iter().any(|mark| !mark.saved) {
        return Freshness::Missing;
    }
    let checked: Vec<Option<i64>> = marks
        .iter()
        .filter(|mark| mark.checkable)
        .map(|mark| mark.checked_at)
        .collect();
    if checked.is_empty() {
        return Freshness::Unchecked;
    }
    let Some(oldest) = checked.iter().copied().min().flatten() else {
        return Freshness::Stale;
    };
    match now - oldest < window {
        true => Freshness::Fresh(oldest),
        false => Freshness::Stale,
    }
}

impl Freshness {
    pub fn name(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Unchecked => "unchecked",
            Self::Stale => "stale",
            Self::Fresh(_) => "fresh",
        }
    }

    pub fn at(self) -> Option<i64> {
        match self {
            Self::Fresh(at) => Some(at),
            _ => None,
        }
    }
}
