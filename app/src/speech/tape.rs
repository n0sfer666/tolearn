use std::path::Path;
use std::sync::Mutex;

use tolearn_speech::capture::Microphone;
use tolearn_speech::listen::Listener;
use tolearn_speech::{SpeechError, model};

static TAPE: Mutex<Option<Microphone>> = Mutex::new(None);
static EAR: Mutex<Option<Listener>> = Mutex::new(None);

pub fn listening() -> bool {
    TAPE.lock().is_ok_and(|tape| tape.is_some())
}

pub fn start() -> Result<(), SpeechError> {
    let mut tape = TAPE.lock().map_err(|_| seized())?;
    if tape.is_some() {
        return Ok(());
    }
    *tape = Some(Microphone::open()?);
    Ok(())
}

pub fn stop(resources: &Path, language: &str) -> Result<String, SpeechError> {
    let taken = TAPE.lock().map_err(|_| seized())?.take();
    let Some(microphone) = taken else {
        return Err(SpeechError::Silent);
    };
    let voice = microphone.stop()?;
    let mut ear = EAR.lock().map_err(|_| seized())?;
    if ear.is_none() {
        *ear = Some(Listener::open(&model::located(resources)?)?);
    }
    match ear.as_ref() {
        Some(listener) => listener.hear(&voice, language),
        None => Err(SpeechError::Failed("модель не открылась".to_owned())),
    }
}

fn seized() -> SpeechError {
    SpeechError::Failed("запись занята другим потоком".to_owned())
}
