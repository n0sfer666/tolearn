use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u32,
    month: u32,
    day: u32,
}

impl Date {
    pub fn parse(value: &str) -> Option<Self> {
        let [year, month, day] = split(value, '-')?;
        let (year, month, day) = (number(year, 4)?, number(month, 2)?, number(day, 2)?);
        if !(1..=12).contains(&month) || day < 1 || day > days_in(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    pub fn plus_days(self, days: u32) -> Self {
        let mut date = self;
        for _ in 0..days {
            date = date.tomorrow();
        }
        date
    }

    fn tomorrow(self) -> Self {
        let Self { year, month, day } = self;
        match (day + 1, month) {
            (day, month) if day <= days_in(year, month) => Self { year, month, day },
            (_, 12) => Self {
                year: year + 1,
                month: 1,
                day: 1,
            },
            (_, month) => Self {
                year,
                month: month + 1,
                day: 1,
            },
        }
    }
}

impl Display for Date {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

pub(crate) fn split<const N: usize>(value: &str, separator: char) -> Option<[&str; N]> {
    let mut parts = value.split(separator);
    let mut taken = [""; N];
    for slot in &mut taken {
        *slot = parts.next()?;
    }
    parts.next().is_none().then_some(taken)
}

pub(crate) fn number(value: &str, digits: usize) -> Option<u32> {
    if value.len() != digits || !value.chars().all(|digit| digit.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

pub(crate) fn days_in(year: u32, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_leap(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
