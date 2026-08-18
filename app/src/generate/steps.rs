use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use tolearn_core::generate::repair;
use tolearn_provider::{Provider, Said, Watch, watched};

use super::jobs::Job;
use crate::journal::Journal;

pub const ROUNDS: u32 = 3;
pub const TRIES: u32 = 3;

const PAUSE: Duration = Duration::from_secs(2);
const KIND: &str = "Генерация";

pub struct Speaker {
    pub provider: Provider,
    pub key: Option<String>,
    pub journal: Journal,
}

pub fn taken<T>(
    speaker: &Speaker,
    job: &Arc<Job>,
    asked: &str,
    judge: impl Fn(&str) -> Result<T, Vec<String>>,
) -> Result<T, Vec<String>> {
    let mut asking = asked.to_owned();
    let mut complaints = Vec::new();
    let watch = listener(job);
    for round in 1..=ROUNDS {
        job.attempting(round);
        let said = heard(speaker, job, &asking, &watch)?;
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

fn heard(
    speaker: &Speaker,
    job: &Arc<Job>,
    asking: &str,
    watch: &Watch,
) -> Result<Said, Vec<String>> {
    let mut left = TRIES;
    loop {
        job.asking();
        let answer = watched(
            &speaker.provider,
            speaker.key.as_deref(),
            asking,
            Arc::clone(watch),
        );
        left -= 1;
        let error = match answer {
            Ok(said) => {
                speaker.journal.said(KIND, asking, &said.text);
                job.spent(said.tokens);
                job.retrying(0);
                return Ok(said);
            }
            Err(error) => error,
        };
        speaker.journal.refused(KIND, asking, &error.to_string());
        job.spent(None);
        if left == 0 || job.stopped() || !error.transient() {
            return Err(vec![error.to_string()]);
        }
        job.retrying(TRIES - left + 1);
        sleep(PAUSE);
    }
}

fn listener(job: &Arc<Job>) -> Watch {
    let job = Arc::clone(job);
    Arc::new(move |piece: &str| job.heard(piece))
}
