use std::path::{Path, PathBuf};

use crate::error::SpeechError;

pub const ENV: &str = "TOLEARN_WHISPER_MODEL";
pub const FILE: &str = "ggml-small-q5_1.bin";

pub fn located(resources: &Path) -> Result<PathBuf, SpeechError> {
    if let Some(asked) = std::env::var_os(ENV) {
        let asked = PathBuf::from(asked);
        return if asked.is_file() {
            Ok(asked)
        } else {
            Err(SpeechError::Unreadable {
                path: asked,
                reason: format!("{ENV} указывает не на файл"),
            })
        };
    }
    let shipped = resources.join(FILE);
    if shipped.is_file() {
        Ok(shipped)
    } else {
        Err(SpeechError::NoModel)
    }
}
