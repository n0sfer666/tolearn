use std::time::Instant;

use crate::ask::briefly;
use crate::error::CheckError;
use crate::types::Provider;

pub const PROMPT: &str = "Ответь одним словом: готов?";

const SAID_CHARS: usize = 400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probed {
    pub said: String,
    pub took_ms: u32,
    pub thinking: bool,
}

pub fn probe(provider: &Provider, key: Option<&str>) -> Result<Probed, CheckError> {
    let started = Instant::now();
    let told = briefly(provider, key, PROMPT)?;
    Ok(Probed {
        said: told.text.chars().take(SAID_CHARS).collect(),
        took_ms: took(started),
        thinking: told.thinking,
    })
}

pub(crate) fn took(started: Instant) -> u32 {
    u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX)
}
