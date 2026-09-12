use std::path::Path;

use serde::{Deserialize, Serialize};
use tolearn_core::Hours;
use tolearn_core::program::StageRow;

use crate::error::GenerateError;
use crate::sources::Cache;

use super::after::After;
use super::offered::Fork;
use super::variant::Variant;

const FORK: &str = "fork";

#[derive(Serialize, Deserialize)]
struct Kept {
    variants: Vec<KeptVariant>,
}

#[derive(Serialize, Deserialize)]
struct KeptVariant {
    id: String,
    title: String,
    hours: (u32, u32),
    why: String,
    recommended: bool,
}

pub(super) fn load(data: &Path, after: &After<'_>) -> Result<Option<Fork>, GenerateError> {
    let kept: Option<Kept> = Cache::at(data, after.node).load(FORK, after.stage)?;
    Ok(kept.map(|kept| Fork {
        variants: kept
            .variants
            .into_iter()
            .map(|variant| {
                let (min, max) = variant.hours;
                Variant {
                    row: StageRow {
                        id: variant.id,
                        title: variant.title,
                        hours: Hours { min, max },
                    },
                    why: variant.why,
                    recommended: variant.recommended,
                }
            })
            .collect(),
    }))
}

pub(super) fn save(data: &Path, after: &After<'_>, fork: &Fork) -> Result<(), GenerateError> {
    let kept = Kept {
        variants: fork
            .variants
            .iter()
            .map(|variant| KeptVariant {
                id: variant.row.id.clone(),
                title: variant.row.title.clone(),
                hours: (variant.row.hours.min, variant.row.hours.max),
                why: variant.why.clone(),
                recommended: variant.recommended,
            })
            .collect(),
    };
    Cache::at(data, after.node).save(FORK, after.stage, &kept)
}

pub(super) fn forget(data: &Path, after: &After<'_>) -> Result<(), GenerateError> {
    Cache::at(data, after.node).forget(FORK, after.stage)
}
