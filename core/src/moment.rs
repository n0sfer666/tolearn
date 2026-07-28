use crate::date::{days_in, number, split};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moment {
    seconds: i64,
}

impl Moment {
    pub fn parse(value: &str) -> Option<Self> {
        let (stamp, offset) = zoned(value)?;
        let (day, time) = stamp.split_once(['T', 't', ' '])?;
        let [year, month, date] = split(day, '-')?;
        let civil = days(number(year, 4)?, number(month, 2)?, number(date, 2)?)?;
        Some(Self {
            seconds: civil * 86_400 + clock(time)? - offset,
        })
    }

    pub fn seconds_since(self, earlier: Self) -> i64 {
        self.seconds - earlier.seconds
    }
}

fn zoned(value: &str) -> Option<(&str, i64)> {
    if let Some(stamp) = value.strip_suffix(['Z', 'z']) {
        return Some((stamp, 0));
    }
    let head = value.get(..value.len().checked_sub(6)?)?;
    let tail = value.get(value.len().checked_sub(6)?..)?;
    let sign = match tail.chars().next()? {
        '+' => 1,
        '-' => -1,
        _ => return None,
    };
    let [hours, minutes] = split(tail.get(1..)?, ':')?;
    let shift = i64::from(number(hours, 2)?) * 3600 + i64::from(number(minutes, 2)?) * 60;
    if shift >= 86_400 {
        return None;
    }
    Some((head, sign * shift))
}

fn clock(time: &str) -> Option<i64> {
    let plain = time.split_once('.').map_or(time, |(head, _)| head);
    let [hours, minutes, seconds] = split(plain, ':')?;
    let (hours, minutes, seconds) = (number(hours, 2)?, number(minutes, 2)?, number(seconds, 2)?);
    if hours > 23 || minutes > 59 || seconds > 60 {
        return None;
    }
    Some(i64::from(hours) * 3600 + i64::from(minutes) * 60 + i64::from(seconds))
}

fn days(year: u32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || day < 1 || day > days_in(year, month) {
        return None;
    }
    let year = i64::from(year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year =
        (153 * (i64::from(month) + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let of_era_days = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    Some(era * 146_097 + of_era_days - 719_468)
}
