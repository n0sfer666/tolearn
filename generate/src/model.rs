use std::fmt;

use tolearn_provider::{CheckError, Said};

pub trait Model: fmt::Debug {
    fn ask(&self, prompt: &str) -> Result<Said, CheckError>;
}
