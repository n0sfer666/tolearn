mod jobs;
mod live;
mod made;
mod parts;
mod registry;
mod steps;

pub use live::Live;
pub use made::{Made, Staged, Summary};
pub use registry::{forget, go, look, stop, take};
pub use steps::{ROUNDS, TRIES};

use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use tolearn_core::generate::{Request, roadmap, topic};
use tolearn_core::roadmap::Roadmap;
use tolearn_core::topic::Topic;
use tolearn_provider::Provider;

use jobs::Job;
use steps::{Speaker, taken};

const PATIENCE: Duration = Duration::from_secs(600);
const PULSE: Duration = Duration::from_millis(100);

pub fn start(provider: Provider, key: Option<String>, request: Request) -> String {
    let (name, job) = registry::register();
    let speaker = Speaker { provider, key };
    std::thread::spawn(move || run(&speaker, &request, &job));
    name
}

fn run(speaker: &Speaker, request: &Request, job: &Arc<Job>) {
    match build(speaker, request, job) {
        Ok(()) => {}
        Err(_) if job.stopped() => job.broke(Vec::new()),
        Err(refused) => job.broke(refused),
    }
}

fn build(speaker: &Speaker, request: &Request, job: &Arc<Job>) -> Result<(), Vec<String>> {
    job.stepping("skeleton", "");
    let (map, map_text, progress) = taken(speaker, job, &roadmap(request), parts::skeleton)?;
    job.counted(map.topics.len());

    job.stepping("confirm", "");
    if !confirmed(job) {
        return Err(Vec::new());
    }

    let mut got = map.topics.iter().map(|_| None).collect::<Vec<_>>();
    loop {
        let missed = gather(speaker, job, &map, &mut got)?;
        if missed.is_empty() {
            break;
        }
        if !mended(job, missed) {
            return Err(Vec::new());
        }
    }

    let mut topics = Vec::new();
    let mut files = Vec::new();
    for (entry, made) in map.topics.iter().zip(got) {
        if let Some((made, text)) = made {
            files.push((entry.file.clone(), text));
            topics.push(made);
        }
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

fn gather(
    speaker: &Speaker,
    job: &Arc<Job>,
    map: &Roadmap,
    got: &mut [Option<(Topic, String)>],
) -> Result<Vec<String>, Vec<String>> {
    let mut missed = Vec::new();
    for (at, entry) in map.topics.iter().enumerate() {
        if got[at].is_some() {
            continue;
        }
        if job.stopped() {
            return Err(Vec::new());
        }
        job.stepping("topic", &entry.title);
        let asked = topic(map, entry);
        match taken(speaker, job, &asked, |answer| {
            parts::one(map, entry, answer)
        }) {
            Ok(pair) => {
                got[at] = Some(pair);
                job.stepped();
            }
            Err(_) if job.stopped() => return Err(Vec::new()),
            Err(why) => missed.push(format!("{}: {}", entry.title, why.join("; "))),
        }
    }
    Ok(missed)
}

fn confirmed(job: &Job) -> bool {
    job.waits();
    let until = Instant::now() + PATIENCE;
    while !job.allowed() && Instant::now() < until {
        sleep(PULSE);
    }
    job.allowed() && !job.stopped()
}

fn mended(job: &Job, missed: Vec<String>) -> bool {
    job.stepping("missed", "");
    job.missing(missed);
    let until = Instant::now() + PATIENCE;
    while !job.asked() && Instant::now() < until {
        sleep(PULSE);
    }
    if !job.asked() || job.stopped() {
        return false;
    }
    job.mending();
    true
}
