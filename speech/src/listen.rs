use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;
use std::ptr;
use std::sync::Once;

use crate::error::SpeechError;
use crate::ffi;
use crate::pcm::Pcm;

const THREADS: usize = 4;

static HUSH: Once = Once::new();

#[derive(Debug)]
pub struct Listener {
    handle: *mut c_void,
}

unsafe impl Send for Listener {}

impl Listener {
    pub fn open(model: &Path) -> Result<Self, SpeechError> {
        HUSH.call_once(|| unsafe { ffi::tolearn_speech_hush() });

        let path = CString::new(model.as_os_str().as_encoded_bytes()).map_err(|_| {
            SpeechError::Rejected {
                path: model.to_path_buf(),
                reason: "в пути есть нулевой байт".to_owned(),
            }
        })?;
        let handle = unsafe { ffi::tolearn_speech_open(path.as_ptr(), 1) };
        if handle.is_null() {
            return Err(SpeechError::Rejected {
                path: model.to_path_buf(),
                reason: "whisper.cpp не принял файл весов".to_owned(),
            });
        }
        Ok(Self { handle })
    }

    pub fn hear(&self, voice: &Pcm, language: &str) -> Result<String, SpeechError> {
        if voice.is_empty() {
            return Err(SpeechError::Silent);
        }
        let samples = i32::try_from(voice.samples().len())
            .map_err(|_| SpeechError::Failed("запись длиннее, чем whisper.cpp умеет".to_owned()))?;
        let tongue = CString::new(language)
            .map_err(|_| SpeechError::Failed("язык задан с нулевым байтом".to_owned()))?;
        let asked = if language.is_empty() {
            ptr::null()
        } else {
            tongue.as_ptr()
        };

        let mut heard: *mut c_char = ptr::null_mut();
        let failed = unsafe {
            ffi::tolearn_speech_hear(
                self.handle,
                voice.samples().as_ptr(),
                samples,
                asked,
                threads(),
                &raw mut heard,
            )
        };
        if failed != 0 {
            return Err(SpeechError::Failed(format!(
                "whisper.cpp вернул код {failed}"
            )));
        }
        if heard.is_null() {
            return Err(SpeechError::Failed(
                "whisper.cpp не вернул текста".to_owned(),
            ));
        }
        let text = unsafe { CStr::from_ptr(heard) }
            .to_string_lossy()
            .trim()
            .to_owned();
        unsafe { ffi::tolearn_speech_forget(heard) };
        Ok(text)
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        unsafe { ffi::tolearn_speech_close(self.handle) };
    }
}

fn threads() -> i32 {
    let cores = std::thread::available_parallelism().map_or(THREADS, std::num::NonZero::get);
    i32::try_from(cores.min(THREADS)).unwrap_or(1)
}
