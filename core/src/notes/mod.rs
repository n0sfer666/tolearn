mod error;
mod frontmatter;
mod index;
mod store;
mod types;

pub use error::NoteError;
pub use index::index;
pub use store::{read, save};
pub use types::{Note, Stamp};
