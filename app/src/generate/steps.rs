use tolearn_core::generate::repair;
use tolearn_provider::{Provider, ask};

use super::jobs::Job;

pub const ROUNDS: u32 = 3;

pub struct Speaker {
    pub provider: Provider,
    pub key: Option<String>,
}

pub fn taken<T>(
    speaker: &Speaker,
    job: &Job,
    asked: &str,
    judge: impl Fn(&str) -> Result<T, Vec<String>>,
) -> Result<T, Vec<String>> {
    let mut asking = asked.to_owned();
    let mut complaints = Vec::new();
    for round in 1..=ROUNDS {
        job.attempting(round);
        job.asking();
        let said = ask(&speaker.provider, speaker.key.as_deref(), &asking)
            .inspect_err(|_| job.spent(None))
            .map_err(|error| vec![error.to_string()])?;
        job.spent(said.tokens);
        if job.stopped() {
            return Err(Vec::new());
        }
        match judge(&said.text) {
            Ok(made) => return Ok(made),
            Err(found) => {
                asking = repair(asked, &said.text, &found);
                complaints = found;
            }
        }
    }
    Err(complaints)
}
