mod build;
mod error;
mod gate;
mod halt;
mod located;
mod model;
mod object;
mod progress;
mod step;

pub mod diagram;
pub mod exam;
pub mod fork;
pub mod ledger;
pub mod plan;
pub mod regenerate;
pub mod sources;
pub mod stage;
pub mod start;

pub use error::GenerateError;
pub use gate::{HOSTS, Online, REACH_TIMEOUT_SECS, local, online};
pub use model::Model;
pub use progress::{Progress, stepped};
pub use step::Step;

pub const REPAIRS: usize = 3;
