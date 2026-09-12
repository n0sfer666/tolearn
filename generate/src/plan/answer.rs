use serde::Deserialize;
use tolearn_core::Hours;
use tolearn_core::program::{StageRow, Volatility};

use super::flaw::Flaw;
use super::types::{Part, Plan};

#[derive(Deserialize)]
struct Raw {
    title: String,
    slug: String,
    goal: String,
    volatility: Changes,
    #[serde(default)]
    stages: Vec<RawStage>,
    #[serde(default)]
    children: Vec<RawPart>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Changes {
    Stable,
    Evolving,
    Volatile,
}

#[derive(Deserialize)]
struct RawStage {
    id: String,
    title: String,
    hours: (u32, u32),
}

#[derive(Deserialize)]
struct RawPart {
    title: String,
    goal: String,
    hours: (u32, u32),
}

pub(super) fn read(text: &str) -> Result<Plan, Flaw> {
    let object = match (text.find('{'), text.rfind('}')) {
        (Some(start), Some(end)) if start < end => &text[start..=end],
        _ => return Err(Flaw::Unreadable("в ответе нет объекта JSON".to_owned())),
    };
    let raw: Raw =
        serde_json::from_str(object).map_err(|error| Flaw::Unreadable(error.to_string()))?;
    Ok(Plan {
        title: raw.title,
        slug: raw.slug,
        goal: raw.goal,
        volatility: match raw.volatility {
            Changes::Stable => Volatility::Stable,
            Changes::Evolving => Volatility::Evolving,
            Changes::Volatile => Volatility::Volatile,
        },
        stages: raw
            .stages
            .into_iter()
            .map(|row| StageRow {
                id: row.id,
                title: row.title,
                hours: hours(row.hours),
            })
            .collect(),
        children: raw
            .children
            .into_iter()
            .map(|row| Part {
                title: row.title,
                goal: row.goal,
                hours: hours(row.hours),
            })
            .collect(),
    })
}

fn hours((min, max): (u32, u32)) -> Hours {
    Hours { min, max }
}
