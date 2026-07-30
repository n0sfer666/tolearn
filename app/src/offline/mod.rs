mod jobs;
mod renderer;
mod saver;
mod seen;

pub use jobs::{Live, look, stop};
pub use renderer::install;
pub use seen::{Seen, seen};

use std::cell::RefCell;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use tolearn_core::topic::Material;
use tolearn_offline::queue::unload;
use tolearn_offline::store::{Held, Store};
use tolearn_offline::video;

use jobs::Job;
use saver::Bundled;

pub fn start(root: &Path, budget: u64, program: &str, materials: Vec<Material>) -> String {
    let (name, job) = jobs::register(materials.len());
    let root = root.to_path_buf();
    let program = program.to_owned();
    std::thread::spawn(move || run(&root, budget, &program, &materials, &job));
    name
}

pub fn opened(root: &Path, budget: u64, url: &str) -> Option<Held> {
    seen(root, budget).opened(url, now())
}

pub fn failed(name: &str) -> Vec<String> {
    look(name)
        .map(|live| live.failed.into_iter().map(|(url, _)| url).collect())
        .unwrap_or_default()
}

fn run(root: &Path, budget: u64, program: &str, materials: &[Material], job: &Job) {
    let store = match Store::open(root, budget) {
        Ok(store) => store,
        Err(error) => return job.broke(&error.to_string()),
    };
    let saver = Bundled {
        store: RefCell::new(store),
        program: program.to_owned(),
        at: now(),
        limit: budget,
        tools: video::ready(&std::env::var("PATH").unwrap_or_default()).ok(),
        job,
    };

    let report = unload(materials, &saver, &job.stop);
    let mut store = saver.store.into_inner();
    let _ = store.protect(program, true);
    let _ = store.sweep();
    job.told(&report);
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|span| i64::try_from(span.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}
