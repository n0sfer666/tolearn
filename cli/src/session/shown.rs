use std::time::Duration;

use tolearn_core::Hours;
use tolearn_generate::Step;

use super::spent::Spent;

const THOUSANDS: char = '\u{a0}';

pub fn named(step: Step) -> String {
    match step {
        Step::Plan => "карта".to_owned(),
        Step::Revise => "правка карты".to_owned(),
        Step::Fork => "развилка".to_owned(),
        Step::Part => "часть".to_owned(),
        Step::Sources => "источники".to_owned(),
        Step::Text => "текст".to_owned(),
        Step::Repair(round) => format!("починка {round}"),
        Step::Diagrams => "схемы".to_owned(),
        Step::Write => "запись".to_owned(),
    }
}

pub fn spent(elapsed: Duration, spent: Spent) -> Vec<String> {
    let mut parts = vec![format!("{:.1} с", elapsed.as_secs_f64())];
    if spent.calls > spent.unknown {
        parts.push(format!(
            "{} → {} ток.",
            grouped(spent.input),
            grouped(spent.output)
        ));
    }
    if spent.unknown > 0 {
        parts.push(format!("без данных о токенах: {}", spent.unknown));
    }
    parts
}

pub fn hours(hours: Hours) -> String {
    match hours.min == hours.max {
        true => format!("{} ч", hours.min),
        false => format!("{}–{} ч", hours.min, hours.max),
    }
}

pub fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut shown = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            shown.push(THOUSANDS);
        }
        shown.push(digit);
    }
    shown
}
