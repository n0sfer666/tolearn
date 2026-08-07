use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use super::jobs::Job;
use super::live::Live;
use super::made::Made;

static JOBS: LazyLock<Mutex<HashMap<String, Arc<Job>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static COUNTED: AtomicUsize = AtomicUsize::new(0);

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
        job.stopping();
        true
    })
}

pub fn take(name: &str) -> Option<Made> {
    found(name)?.made()
}

pub fn forget(name: &str) {
    if let Ok(mut jobs) = JOBS.lock() {
        jobs.remove(name);
    }
}

fn found(name: &str) -> Option<Arc<Job>> {
    JOBS.lock().ok()?.get(name).map(Arc::clone)
}
