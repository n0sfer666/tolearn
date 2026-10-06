use serde::Deserialize;
use tolearn_core::Hours;
use tolearn_core::program::StageRow;

use crate::object::object;

use super::flaw::Flaw;
use super::offered::Fork;
use super::variant::Variant;

#[derive(Deserialize)]
struct Raw {
    next: RawNext,
    #[serde(default)]
    alternatives: Vec<RawAlternative>,
}

#[derive(Deserialize)]
struct RawNext {
    why: String,
    #[serde(default)]
    recommended: bool,
}

#[derive(Deserialize)]
struct RawAlternative {
    id: String,
    title: String,
    hours: (u32, u32),
    why: String,
    #[serde(default)]
    recommended: bool,
}

pub(super) fn read(text: &str, next: &StageRow) -> Result<Fork, Flaw> {
    let raw: Raw = serde_json::from_str(object(text).map_err(Flaw::Unreadable)?)
        .map_err(|error| Flaw::Unreadable(error.to_string()))?;
    let alone = raw.alternatives.is_empty();
    let mut variants = vec![Variant {
        row: next.clone(),
        why: raw.next.why,
        recommended: raw.next.recommended || alone,
    }];
    variants.extend(raw.alternatives.into_iter().map(|row| {
        let (min, max) = row.hours;
        Variant {
            row: StageRow {
                id: row.id,
                title: row.title,
                hours: Hours { min, max },
            },
            why: row.why,
            recommended: row.recommended,
        }
    }));
    Ok(Fork { variants })
}
