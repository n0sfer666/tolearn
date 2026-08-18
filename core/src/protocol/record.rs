use super::types::Applied;
use crate::progress::{Document, DocumentError};

pub fn record(
    document: &mut Document,
    topic: &str,
    applied: &Applied,
) -> Result<(), DocumentError> {
    document.start(topic)?;
    if let Some(attempt) = applied.state.attempts.last() {
        document.push_attempt(topic, attempt)?;
    }
    document.settle(topic, &applied.state)
}
