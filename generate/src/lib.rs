mod error;
mod gate;
mod model;

pub mod sources;

pub use error::GenerateError;
pub use gate::{HOSTS, Online, REACH_TIMEOUT_SECS, online};
pub use model::Model;
