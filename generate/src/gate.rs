use tolearn_offline::reach::Reach;
use tolearn_provider::{CheckError, Said};

use crate::error::GenerateError;
use crate::model::Model;

pub const HOSTS: [&str; 2] = ["openlibrary.org", "commons.wikimedia.org"];

pub const REACH_TIMEOUT_SECS: u64 = 10;

#[derive(Debug)]
pub struct Online<'a> {
    model: &'a dyn Model,
}

impl Online<'_> {
    pub fn ask(&self, prompt: &str) -> Result<Said, GenerateError> {
        self.model.ask(prompt).map_err(|error| match error {
            CheckError::Cancelled => GenerateError::Cancelled,
            error => GenerateError::Provider(error),
        })
    }
}

pub fn online<'a>(reach: &dyn Reach, model: &'a dyn Model) -> Result<Online<'a>, GenerateError> {
    for host in HOSTS {
        reach
            .reach(&format!("https://{host}/"))
            .map_err(|reason| GenerateError::Offline {
                host: host.to_owned(),
                reason,
            })?;
    }
    Ok(Online { model })
}
