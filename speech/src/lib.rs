mod error;
mod pcm;

#[cfg(feature = "speech")]
mod ffi;

#[cfg(feature = "speech")]
mod resample;

#[cfg(feature = "speech")]
pub mod capture;

#[cfg(feature = "speech")]
pub mod listen;

pub mod model;
pub mod wav;
pub mod wer;
pub mod words;

pub use error::SpeechError;
pub use pcm::{Pcm, RATE};

pub fn available() -> bool {
    cfg!(feature = "speech")
}
