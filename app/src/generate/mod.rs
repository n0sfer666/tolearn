mod jobs;
mod made;
mod parts;
mod steps;

pub use jobs::{Live, forget, go, look, stop, take};
pub use made::{Made, Staged, Summary};
pub use steps::ROUNDS;

use std::thread::sleep;
use std::time::{Duration, Instant};

use tolearn_core::generate::{Request, roadmap, topic};
use tolearn_provider::Provider;

use jobs::Job;
use steps::{Speaker, taken};

const PATIENCE: Duration = Duration::from_secs(600);
const PULSE: Duration = Duration::from_millis(100);

pub fn start(provider: Provider, key: Option<String>, request: Request) -> String {
    let (name, job) = jobs::register();
    let speaker = Speaker { provider, key };
    std::thread::spawn(move || run(&speaker, &request, &job));
    name
}

fn run(speaker: &Speaker, request: &Request, job: &Job) {
    match build(speaker, request, job) {
        Ok(()) => {}
        Err(_) if job.stopped() => job.broke(Vec::new()),
        Err(refused) => job.broke(refused),
    }
}

fn build(speaker: &Speaker, request: &Request, job: &Job) -> Result<(), Vec<String>> {
    job.stepping("skeleton", "");
    let (map, map_text, progress) = taken(speaker, job, &roadmap(request), parts::skeleton)?;
    job.counted(map.topics.len());

    job.stepping("confirm", "");
    if !confirmed(job) {
        return Err(Vec::new());
    }

    let mut topics = Vec::new();
    let mut files = Vec::new();
    for entry in &map.topics {
        if job.stopped() {
            return Err(Vec::new());
        }
        job.stepping("topic", &entry.title);
        let asked = topic(&map, entry);
        let (made, text) = taken(speaker, job, &asked, |answer| {
            parts::one(&map, entry, answer)
        })?;
        files.push((entry.file.clone(), text));
        topics.push(made);
        job.stepped();
    }

    let (whole, marked) = parts::whole(&map_text, &topics)?;
    let summary = made::summary(&whole);
    job.told(
        Made {
            id: whole.id,
            title: whole.title,
            roadmap: marked,
            progress,
            topics: files,
        },
        summary,
    );
    Ok(())
}

fn confirmed(job: &Job) -> bool {
    job.waits();
    let until = Instant::now() + PATIENCE;
    while !job.allowed() && Instant::now() < until {
        sleep(PULSE);
    }
    job.allowed() && !job.stopped()
}
