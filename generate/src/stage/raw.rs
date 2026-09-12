use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::object::object;

use super::flaw::Flaw;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct Raw {
    pub(super) blocks: Vec<RawBlock>,
    pub(super) practice: RawPractice,
    #[serde(default)]
    pub(super) questions: Vec<RawQuestion>,
    #[serde(default)]
    pub(super) terms: Vec<String>,
    #[serde(default)]
    pub(super) tools: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct RawBlock {
    pub(super) kind: String,
    #[serde(default)]
    pub(super) text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) image: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) sources: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct RawPractice {
    #[serde(default)]
    pub(super) task: Vec<RawBlock>,
    #[serde(default)]
    pub(super) deliverable: String,
    #[serde(default)]
    pub(super) constraints: Vec<RawCheck>,
    #[serde(default)]
    pub(super) acceptance: Vec<RawCheck>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct RawCheck {
    #[serde(default)]
    pub(super) claim: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) check: Option<String>,
    #[serde(default)]
    pub(super) expect: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct RawQuestion {
    #[serde(default)]
    pub(super) text: String,
    #[serde(default)]
    pub(super) answer: String,
}

pub(super) fn parse<T: DeserializeOwned>(text: &str) -> Result<T, Flaw> {
    serde_json::from_str(object(text).map_err(Flaw::Unreadable)?)
        .map_err(|error| Flaw::Unreadable(error.to_string()))
}
