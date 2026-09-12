mod build;
mod error;
mod gate;
mod halt;
mod model;
mod object;
mod progress;
mod step;

pub mod diagram;
pub mod fork;
pub mod ledger;
pub mod plan;
pub mod sources;
pub mod stage;
pub mod start;

pub use error::GenerateError;
pub use gate::{HOSTS, Online, REACH_TIMEOUT_SECS, online};
pub use model::Model;
pub use progress::Progress;
pub use step::Step;

pub const REPAIRS: usize = 3;
