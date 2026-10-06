mod branch;
mod check;
mod error;
mod load;
mod parse;
mod places;
mod position;
mod tree;
mod types;
mod validate;
mod violation;
mod write;

pub use branch::Branch;
pub use check::check;
pub use error::LoadError;
pub use load::load;
pub use parse::parse;
pub use position::{Position, position};
pub use tree::Tree;
pub use types::{Book, ChildRow, Generation, Map, Page, Program, Sources, StageRow, Volatility};
pub use validate::validate;
pub use violation::Violation;
pub use write::write;

pub(crate) use places::places;

pub const MAX_DEPTH: usize = 3;
