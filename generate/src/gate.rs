use std::time::Instant;

use tolearn_offline::reach::Reach;
use tolearn_provider::{CheckError, Said};

use crate::error::GenerateError;
use crate::ledger::{self, Record, Tally};
use crate::model::Model;
use crate::step::Step;

pub const HOSTS: [&str; 2] = ["openlibrary.org", "commons.wikimedia.org"];

pub const REACH_TIMEOUT_SECS: u64 = 10;

#[derive(Debug)]
pub struct Online<'a> {
    model: &'a dyn Model,
    tally: Tally,
}

impl Online<'_> {
    pub fn ask(&self, step: Step, prompt: &str) -> Result<Said, GenerateError> {
        let round = match step {
            Step::Repair(round) => Some(round),
            _ => None,
        };
        self.call(step, round, prompt)
    }

    pub fn again(&self, step: Step, round: usize, prompt: &str) -> Result<Said, GenerateError> {
        self.call(step, Some(round), prompt)
    }

    fn call(&self, step: Step, round: Option<usize>, prompt: &str) -> Result<Said, GenerateError> {
        let began = Instant::now();
        let said = self.model.ask(prompt);
        let spent = ledger::since(began);
        self.tally
            .push(Record::asked(step, round, spent, said.as_ref().ok()));
        said.map_err(|error| match error {
            CheckError::Cancelled => GenerateError::Cancelled,
            error => GenerateError::Provider(error),
        })
    }

    pub fn tally(&self) -> &Tally {
        &self.tally
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
    Ok(Online {
        model,
        tally: Tally::default(),
    })
}
