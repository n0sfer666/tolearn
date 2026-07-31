mod jobs;
mod renderer;
mod saver;
mod seen;
mod state;
mod watched;

pub use jobs::{Live, look, stop};
pub use renderer::install;
pub use seen::{Seen, label, seen};
pub use state::state;

use std::cell::RefCell;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use tolearn_core::topic::Material;
use tolearn_offline::fresh::Conditional;
use tolearn_offline::queue::{refresh, unload};
use tolearn_offline::store::{Held, Store};
use tolearn_offline::video;

use jobs::Job;
use saver::Bundled;
use watched::Watched;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Save,
    Refresh,
}

pub fn start(
    root: &Path,
    budget: u64,
    program: &str,
    materials: Vec<Material>,
    mode: Mode,
) -> String {
    let (name, job) = jobs::register(materials.len());
    let root = root.to_path_buf();
    let program = program.to_owned();
    std::thread::spawn(move || run(&root, budget, &program, &materials, mode, &job));
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

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|span| i64::try_from(span.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}

fn run(root: &Path, budget: u64, program: &str, materials: &[Material], mode: Mode, job: &Job) {
    let opened = match Store::open(root, budget) {
        Ok(store) => store,
        Err(error) => return job.broke(&error.to_string()),
    };
    let store = RefCell::new(opened);
    let at = now();
    let probe = match Conditional::new(saver::TIMEOUT) {
        Ok(probe) => probe,
        Err(error) => return job.broke(&error),
    };
    let watched = Watched::new(&store, &probe, at, job);
    let saver = Bundled {
        store: &store,
        program: program.to_owned(),
        at,
        limit: budget,
        tools: video::ready(&std::env::var("PATH").unwrap_or_default()).ok(),
        job,
    };

    let report = match mode {
        Mode::Save => unload(materials, &saver, &job.stop),
        Mode::Refresh => refresh(materials, &watched, &saver, &job.stop),
    };
    if !report.cancelled {
        watched.stamped(materials, &report.saved);
    }

    let mut store = store.into_inner();
    let _ = store.protect(program, true);
    let _ = store.sweep();
    job.told(&report);
}
