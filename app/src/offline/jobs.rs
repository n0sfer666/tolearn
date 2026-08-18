use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use tolearn_offline::queue::Report;

use super::marks::Marks;

static JOBS: LazyLock<Mutex<HashMap<String, Arc<Job>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static COUNTED: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Default)]
pub struct Left {
    pub url: String,
    pub title: String,
    pub why: String,
}

#[derive(Debug, Clone, Default)]
pub struct Live {
    pub total: usize,
    pub done: usize,
    pub current: String,
    pub title: String,
    pub topic: String,
    pub topic_at: usize,
    pub topics: usize,
    pub finished: bool,
    pub cancelled: bool,
    pub bytes: u64,
    pub saved: Vec<String>,
    pub skipped: Vec<Left>,
    pub failed: Vec<Left>,
}

#[derive(Debug)]
pub struct Job {
    pub stop: AtomicBool,
    marks: Marks,
    live: Mutex<Live>,
}

impl Job {
    pub fn starting(&self, url: &str) {
        let mark = self.marks.of(url);
        self.change(|live| {
            url.clone_into(&mut live.current);
            live.title = mark.title;
            live.topic = mark.topic;
            live.topic_at = mark.topic_at;
        });
    }

    pub fn stepped(&self) {
        self.change(|live| live.done += 1);
    }

    pub fn told(&self, report: &Report) {
        self.change(|live| {
            live.finished = true;
            live.cancelled = report.cancelled;
            live.bytes = report.bytes;
            live.current = String::new();
            live.saved = report.saved.clone();
            live.skipped = report
                .skipped
                .iter()
                .map(|(url, why)| self.left(url, why.name()))
                .collect();
            live.failed = report
                .failed
                .iter()
                .map(|(url, why)| self.left(url, why))
                .collect();
        });
    }

    pub fn broke(&self, why: &str) {
        self.change(|live| {
            live.finished = true;
            let left = Left {
                url: live.current.clone(),
                title: live.title.clone(),
                why: why.to_owned(),
            };
            live.failed.push(left);
            live.current = String::new();
        });
    }

    fn left(&self, url: &str, why: &str) -> Left {
        Left {
            url: url.to_owned(),
            title: self.marks.title(url),
            why: why.to_owned(),
        }
    }

    pub fn look(&self) -> Live {
        self.live
            .lock()
            .map(|live| live.clone())
            .unwrap_or_default()
    }

    fn change(&self, edit: impl FnOnce(&mut Live)) {
        if let Ok(mut live) = self.live.lock() {
            edit(&mut live);
        }
    }
}

pub fn register(total: usize, marks: Marks) -> (String, Arc<Job>) {
    let name = format!("job-{}", COUNTED.fetch_add(1, Ordering::Relaxed) + 1);
    let job = Arc::new(Job {
        stop: AtomicBool::new(false),
        live: Mutex::new(Live {
            total,
            topics: marks.topics(),
            ..Live::default()
        }),
        marks,
    });
    if let Ok(mut jobs) = JOBS.lock() {
        jobs.insert(name.clone(), Arc::clone(&job));
    }
    (name, job)
}

pub fn look(name: &str) -> Option<Live> {
    found(name).map(|job| job.look())
}

pub fn stop(name: &str) -> bool {
    let Some(job) = found(name) else {
        return false;
    };
    job.stop.store(true, Ordering::Relaxed);
    true
}

fn found(name: &str) -> Option<Arc<Job>> {
    JOBS.lock().ok()?.get(name).map(Arc::clone)
}
