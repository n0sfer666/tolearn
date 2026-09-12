mod error;
mod gate;
mod model;
mod object;

pub mod diagram;
pub mod plan;
pub mod sources;
pub mod stage;

pub use error::GenerateError;
pub use gate::{HOSTS, Online, REACH_TIMEOUT_SECS, online};
pub use model::Model;

pub const REPAIRS: usize = 3;
