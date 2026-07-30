use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use tolearn_offline::queue::Report;

static JOBS: LazyLock<Mutex<HashMap<String, Arc<Job>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static COUNTED: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Default)]
pub struct Live {
    pub total: usize,
    pub done: usize,
    pub current: String,
    pub finished: bool,
    pub cancelled: bool,
    pub bytes: u64,
    pub saved: Vec<String>,
    pub skipped: Vec<(String, String)>,
    pub failed: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct Job {
    pub stop: AtomicBool,
    live: Mutex<Live>,
}

impl Job {
    pub fn starting(&self, url: &str) {
        self.change(|live| url.clone_into(&mut live.current));
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
                .map(|(url, why)| (url.clone(), why.name().to_owned()))
                .collect();
            live.failed = report.failed.clone();
        });
    }

    pub fn broke(&self, why: &str) {
        self.change(|live| {
            live.finished = true;
            live.failed.push((live.current.clone(), why.to_owned()));
            live.current = String::new();
        });
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

pub fn register(total: usize) -> (String, Arc<Job>) {
    let name = format!("job-{}", COUNTED.fetch_add(1, Ordering::Relaxed) + 1);
    let job = Arc::new(Job {
        stop: AtomicBool::new(false),
        live: Mutex::new(Live {
            total,
            ..Live::default()
        }),
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
