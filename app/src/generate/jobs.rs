use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use super::made::{Made, Summary};

static JOBS: LazyLock<Mutex<HashMap<String, Arc<Job>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static COUNTED: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Default)]
pub struct Live {
    pub step: String,
    pub total: usize,
    pub done: usize,
    pub attempt: u32,
    pub current: String,
    pub waiting: bool,
    pub finished: bool,
    pub cancelled: bool,
    pub refused: Vec<String>,
    pub seconds: u64,
    pub tokens: Option<u32>,
    pub summary: Option<Summary>,
}

#[derive(Debug, Default)]
pub struct Job {
    stop: AtomicBool,
    go: AtomicBool,
    live: Mutex<Live>,
    made: Mutex<Option<Made>>,
}

impl Job {
    pub fn stepping(&self, step: &str, current: &str) {
        self.change(|live| {
            step.clone_into(&mut live.step);
            current.clone_into(&mut live.current);
            live.attempt = 1;
        });
    }

    pub fn attempting(&self, attempt: u32) {
        self.change(|live| live.attempt = attempt);
    }

    pub fn counted(&self, total: usize) {
        self.change(|live| live.total = total);
    }

    pub fn stepped(&self) {
        self.change(|live| live.done += 1);
    }

    pub fn spent(&self, seconds: u64, tokens: Option<u32>) {
        self.change(|live| {
            live.seconds += seconds;
            if let Some(counted) = tokens {
                live.tokens = Some(live.tokens.unwrap_or_default() + counted);
            }
        });
    }

    pub fn told(&self, made: Made, summary: Summary) {
        if let Ok(mut kept) = self.made.lock() {
            *kept = Some(made);
        }
        self.change(|live| {
            live.step = "done".to_owned();
            live.current = String::new();
            live.finished = true;
            live.summary = Some(summary);
        });
    }

    pub fn broke(&self, refused: Vec<String>) {
        self.change(|live| {
            live.finished = true;
            live.cancelled = refused.is_empty();
            live.refused = refused;
            live.current = String::new();
        });
    }

    pub fn waits(&self) {
        self.change(|live| live.waiting = true);
    }

    pub fn goes(&self) {
        self.go.store(true, Ordering::Relaxed);
        self.change(|live| live.waiting = false);
    }

    pub fn allowed(&self) -> bool {
        self.go.load(Ordering::Relaxed)
    }

    pub fn stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
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

pub fn register() -> (String, Arc<Job>) {
    let name = format!("gen-{}", COUNTED.fetch_add(1, Ordering::Relaxed) + 1);
    let job = Arc::new(Job::default());
    if let Ok(mut jobs) = JOBS.lock() {
        jobs.insert(name.clone(), Arc::clone(&job));
    }
    (name, job)
}

pub fn look(name: &str) -> Option<Live> {
    found(name).map(|job| job.look())
}

pub fn go(name: &str) -> bool {
    found(name).is_some_and(|job| {
        job.goes();
        true
    })
}

pub fn stop(name: &str) -> bool {
    found(name).is_some_and(|job| {
        job.stop.store(true, Ordering::Relaxed);
        job.goes();
        true
    })
}

pub fn take(name: &str) -> Option<Made> {
    found(name)?.made.lock().ok()?.clone()
}

pub fn forget(name: &str) {
    if let Ok(mut jobs) = JOBS.lock() {
        jobs.remove(name);
    }
}

fn found(name: &str) -> Option<Arc<Job>> {
    JOBS.lock().ok()?.get(name).map(Arc::clone)
}
