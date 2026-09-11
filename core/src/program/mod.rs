mod check;
mod error;
mod load;
mod parse;
mod places;
mod tree;
mod types;
mod validate;
mod violation;

pub use check::check;
pub use error::LoadError;
pub use load::load;
pub use parse::parse;
pub use tree::Tree;
pub use types::{Book, ChildRow, Generation, Map, Page, Program, Sources, StageRow, Volatility};
pub use validate::validate;
pub use violation::Violation;

pub(crate) use places::places;

pub const MAX_DEPTH: usize = 3;
