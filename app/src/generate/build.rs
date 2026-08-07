use std::path::Path;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use tolearn_core::generate::{Request, roadmap, topic};
use tolearn_core::roadmap::Roadmap;
use tolearn_core::topic::Topic;

use super::draft::{self, Draft};
use super::jobs::Job;
use super::made::{self, Made};
use super::parts;
use super::steps::{ROUNDS, Speaker, taken};

const PULSE: Duration = Duration::from_millis(100);

pub fn fresh(
    speaker: &Speaker,
    request: &Request,
    job: &Arc<Job>,
    root: &Path,
) -> Result<(), Vec<String>> {
    job.stepping("skeleton", "");
    let (map, map_text, progress) = taken(speaker, job, &roadmap(request), parts::skeleton)?;
    let _ = draft::keep(root, &map_text, &progress);
    job.counted(map.topics.len(), 0);

    job.stepping("confirm", "");
    if !confirmed(job) {
        return Err(Vec::new());
    }
    let got = map.topics.iter().map(|_| None).collect();
    whole(
        speaker,
        job,
        root,
        Draft {
            map,
            map_text,
            progress,
            got,
        },
    )
}

pub fn carry(
    speaker: &Speaker,
    job: &Arc<Job>,
    root: &Path,
    draft: Draft,
) -> Result<(), Vec<String>> {
    counted(job, &draft);
    whole(speaker, job, root, draft)
}

fn whole(
    speaker: &Speaker,
    job: &Arc<Job>,
    root: &Path,
    mut draft: Draft,
) -> Result<(), Vec<String>> {
    let mut rounds = 0;
    loop {
        let missed = gather(speaker, job, root, &draft.map, &mut draft.got)?;
        if !missed.is_empty() {
            if !mended(job, missed) {
                return Err(Vec::new());
            }
            continue;
        }

        let (topics, files) = collected(&draft);
        let refused = match parts::whole(&draft.map_text, &topics) {
            Ok((map, marked)) => {
                let summary = made::summary(&map);
                job.told(
                    Made {
                        id: map.id,
                        title: map.title,
                        roadmap: marked,
                        progress: draft.progress,
                        topics: files,
                    },
                    summary,
                );
                return Ok(());
            }
            Err(refused) => refused,
        };

        let blamed = blamed(&draft, &refused.topics);
        if blamed.is_empty() {
            return Err(refused.why);
        }
        for at in blamed {
            if let Some(entry) = draft.map.topics.get(at) {
                draft::erase(root, &entry.file);
            }
            draft.got[at] = None;
        }
        counted(job, &draft);

        rounds += 1;
        if rounds > ROUNDS {
            if !mended(job, refused.why) {
                return Err(Vec::new());
            }
            rounds = 0;
        }
    }
}

fn collected(draft: &Draft) -> (Vec<Topic>, Vec<(String, String)>) {
    let mut topics = Vec::new();
    let mut files = Vec::new();
    for (entry, made) in draft.map.topics.iter().zip(&draft.got) {
        if let Some((made, text)) = made {
            files.push((entry.file.clone(), text.clone()));
            topics.push(made.clone());
        }
    }
    (topics, files)
}

fn blamed(draft: &Draft, named: &[String]) -> Vec<usize> {
    draft
        .map
        .topics
        .iter()
        .enumerate()
        .filter(|&(at, entry)| draft.got[at].is_some() && named.contains(&entry.id))
        .map(|(at, _)| at)
        .collect()
}

fn counted(job: &Job, draft: &Draft) {
    job.counted(draft.got.len(), draft.got.iter().flatten().count());
}

fn gather(
    speaker: &Speaker,
    job: &Arc<Job>,
    root: &Path,
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
        let done: Vec<Topic> = got.iter().flatten().map(|(made, _)| made.clone()).collect();
        match taken(speaker, job, &asked, |answer| {
            parts::one(map, entry, &done, answer)
        }) {
            Ok((made, text)) => {
                let _ = draft::note(root, &entry.file, &text);
                got[at] = Some((made, text));
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
    while !job.allowed() {
        sleep(PULSE);
    }
    !job.stopped()
}

fn mended(job: &Job, missed: Vec<String>) -> bool {
    job.stepping("missed", "");
    job.missing(missed);
    while !job.asked() {
        sleep(PULSE);
    }
    if job.stopped() {
        return false;
    }
    job.mending();
    true
}
