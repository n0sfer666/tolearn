use std::collections::BTreeSet;

use tolearn_core::Hours;
use tolearn_core::program::Map;
use tolearn_core::yaml::is_slug;

use crate::plan::{MAX_HOURS, stage_fits};

use super::MAX_ALTERNATIVES;
use super::flaw::Flaw;
use super::offered::Fork;

pub(super) fn check(fork: &Fork, map: &Map) -> Vec<Flaw> {
    let mut flaws = Vec::new();
    let Some((next, alternatives)) = fork.variants.split_first() else {
        return flaws;
    };
    blank(
        &next.why,
        &format!("почему дальше этап «{}»", next.row.id),
        &mut flaws,
    );
    if alternatives.len() > MAX_ALTERNATIVES {
        flaws.push(Flaw::TooMany(alternatives.len()));
    }
    let mut seen: BTreeSet<&str> = map.stages.iter().map(|row| row.id.as_str()).collect();
    let total = map.hours();
    for variant in alternatives {
        let row = &variant.row;
        blank(
            &row.title,
            &format!("название этапа «{}»", row.id),
            &mut flaws,
        );
        blank(
            &variant.why,
            &format!("почему этап «{}»", row.id),
            &mut flaws,
        );
        if !is_slug(&row.id) {
            flaws.push(Flaw::StageId(row.id.clone()));
        } else if !seen.insert(row.id.as_str()) {
            flaws.push(Flaw::DuplicateId(row.id.clone()));
        }
        if !stage_fits(row.hours) {
            flaws.push(Flaw::StageHours {
                id: row.id.clone(),
                hours: row.hours,
            });
        }
        let hours = Hours {
            min: swapped(total.min, next.row.hours.min, row.hours.min),
            max: swapped(total.max, next.row.hours.max, row.hours.max),
        };
        if hours.max > MAX_HOURS {
            flaws.push(Flaw::LeafHours {
                id: row.id.clone(),
                hours,
            });
        }
    }
    let recommended = fork
        .variants
        .iter()
        .filter(|variant| variant.recommended)
        .count();
    if fork.variants.len() > 1 && recommended != 1 {
        flaws.push(Flaw::Recommended(recommended));
    }
    flaws
}

fn swapped(total: u32, out: u32, into: u32) -> u32 {
    total.saturating_sub(out).saturating_add(into)
}

fn blank(text: &str, what: &str, flaws: &mut Vec<Flaw>) {
    if text.trim().is_empty() {
        flaws.push(Flaw::Blank(what.to_owned()));
    }
}
