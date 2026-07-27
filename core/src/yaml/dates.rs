pub(crate) fn is_date(value: &str) -> bool {
    let Some([year, month, day]) = split(value, '-') else {
        return false;
    };
    let (Some(year), Some(month), Some(day)) = (number(year, 4), number(month, 2), number(day, 2))
    else {
        return false;
    };
    (1..=12).contains(&month) && day >= 1 && day <= days_in(year, month)
}

pub(crate) fn is_moment(value: &str) -> bool {
    let Some((date, rest)) = value.split_once('T') else {
        return false;
    };
    let Some((time, offset)) = split_offset(rest) else {
        return false;
    };
    is_date(date) && is_time(time) && is_offset(offset)
}

fn split_offset(rest: &str) -> Option<(&str, &str)> {
    if let Some(time) = rest.strip_suffix('Z') {
        return Some((time, "Z"));
    }
    let cut = rest.rfind(['+', '-'])?;
    Some((&rest[..cut], &rest[cut..]))
}

fn is_time(value: &str) -> bool {
    let (clock, fraction) = value.split_once('.').unwrap_or((value, "1"));
    let Some([hour, minute, second]) = split(clock, ':') else {
        return false;
    };
    let (Some(hour), Some(minute), Some(second)) =
        (number(hour, 2), number(minute, 2), number(second, 2))
    else {
        return false;
    };
    hour <= 23
        && minute <= 59
        && second <= 60
        && !fraction.is_empty()
        && fraction.chars().all(|digit| digit.is_ascii_digit())
}

fn is_offset(value: &str) -> bool {
    if value == "Z" {
        return true;
    }
    let Some(rest) = value.strip_prefix(['+', '-']) else {
        return false;
    };
    let Some((hour, minute)) = rest.split_once(':') else {
        return false;
    };
    matches!((number(hour, 2), number(minute, 2)), (Some(hour), Some(minute)) if hour <= 23 && minute <= 59)
}

fn split<const N: usize>(value: &str, separator: char) -> Option<[&str; N]> {
    let mut parts = value.split(separator);
    let mut taken = [""; N];
    for slot in &mut taken {
        *slot = parts.next()?;
    }
    parts.next().is_none().then_some(taken)
}

fn number(value: &str, digits: usize) -> Option<u32> {
    if value.len() != digits || !value.chars().all(|digit| digit.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn days_in(year: u32, month: u32) -> u32 {
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
