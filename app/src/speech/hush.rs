use std::path::Path;

use tolearn_speech::SpeechError;

pub fn listening() -> bool {
    false
}

pub fn start() -> Result<(), SpeechError> {
    Err(SpeechError::Off)
}

pub fn stop(_resources: &Path, _language: &str) -> Result<String, SpeechError> {
    Err(SpeechError::Off)
}
