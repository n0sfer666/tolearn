#[cfg(not(feature = "speech"))]
mod hush;
#[cfg(feature = "speech")]
mod tape;
mod tongue;

#[cfg(not(feature = "speech"))]
pub use hush::{listening, start, stop};
#[cfg(feature = "speech")]
pub use tape::{listening, start, stop};
pub use tongue::tongue;

pub fn available() -> bool {
    tolearn_speech::available()
}
