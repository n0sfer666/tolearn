mod build;
mod draft;
mod jobs;
mod live;
mod made;
mod parts;
mod registry;
mod steps;

pub use draft::{Waiting, forget as unpin, waiting};
pub use live::Live;
pub use made::{Made, Staged, Summary};
pub use registry::{forget, go, look, stop, take};
pub use steps::{ROUNDS, TRIES};

use std::path::PathBuf;
use std::sync::Arc;

use tolearn_core::generate::Request;
use tolearn_provider::Provider;

use jobs::Job;
use steps::Speaker;

pub fn start(provider: Provider, key: Option<String>, request: Request, root: PathBuf) -> String {
    let (name, job) = registry::register();
    let speaker = Speaker { provider, key };
    std::thread::spawn(move || ended(build::fresh(&speaker, &request, &job, &root), &job));
    name
}

pub fn carry(
    provider: Provider,
    key: Option<String>,
    root: PathBuf,
    today: String,
) -> Option<String> {
    let draft = draft::read(&root)?;
    let (name, job) = registry::register();
    let speaker = Speaker { provider, key };
    std::thread::spawn(move || ended(build::carry(&speaker, &job, &root, draft, &today), &job));
    Some(name)
}

fn ended(made: Result<(), Vec<String>>, job: &Arc<Job>) {
    match made {
        Ok(()) => {}
        Err(_) if job.stopped() => job.broke(Vec::new()),
        Err(refused) => job.broke(refused),
    }
}
