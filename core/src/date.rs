pub(crate) fn is_date(value: &str) -> bool {
    let Some([year, month, day]) = split(value, '-') else {
        return false;
    };
    let (Some(year), Some(month), Some(day)) = (number(year, 4), number(month, 2), number(day, 2))
    else {
        return false;
    };
    (1..=12).contains(&month) && (1..=days_in(year, month)).contains(&day)
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
