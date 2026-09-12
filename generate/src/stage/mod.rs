mod answer;
mod cited;
mod draft;
mod flaw;
mod gather;
mod gathered;
mod place;
mod prompt;
mod proposal;
mod text;

pub use draft::{Draft, Drafted};
pub use flaw::Flaw;
pub use gather::gather;
pub use gathered::{Chaptered, Dropped, Gathered, Visited};
pub use place::Place;
pub use text::text;

pub const MAX_BOOKS: usize = 2;
pub const MAX_PAGES: usize = 3;
pub const MAX_IMAGES: usize = 2;
pub const SOURCES_PROMPT_CHARS: usize = 2_000;
pub const TEXT_PROMPT_CHARS: usize = 30_000;
