use std::time::{SystemTime, UNIX_EPOCH};

use tolearn_core::Date;

pub fn today() -> Option<Date> {
    civil(i64::try_from(seconds() / 86_400).ok()?)
}

pub fn moment() -> Option<String> {
    let seconds = seconds();
    let day = today()?;
    let rest = seconds % 86_400;
    Some(format!(
        "{day}T{:02}:{:02}:{:02}Z",
        rest / 3_600,
        (rest % 3_600) / 60,
        rest % 60
    ))
}

fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or_default()
}

fn civil(days: i64) -> Option<Date> {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    Date::parse(&format!("{year:04}-{month:02}-{day:02}"))
}
