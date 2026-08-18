mod error;
mod weigh;

pub use error::RepoError;

use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use gix::progress::Discard;
use gix::remote::fetch::Shallow;

#[derive(Debug, Clone)]
pub struct Cloned {
    pub path: PathBuf,
    pub bytes: u64,
}

pub fn clone(url: &str, into: &Path, limit: u64) -> Result<Cloned, RepoError> {
    let stop = AtomicBool::new(false);
    let done = AtomicBool::new(false);
    let taken = std::thread::scope(|scope| {
        let watchman = scope.spawn(|| weigh::watch(into, limit, &stop, &done));
        let outcome = fetch(url, into, &stop);
        done.store(true, Ordering::Relaxed);
        let _ = watchman.join();
        outcome
    });

    let bytes = weigh::size(into);
    if bytes > limit {
        wipe(into);
        return Err(RepoError::TooBig {
            limit,
            taken: bytes,
        });
    }
    match taken {
        Err(reason) => {
            wipe(into);
            Err(RepoError::Clone(reason))
        }
        Ok(()) => Ok(Cloned {
            path: into.to_path_buf(),
            bytes,
        }),
    }
}

fn fetch(url: &str, into: &Path, stop: &AtomicBool) -> Result<(), String> {
    let depth = NonZeroU32::new(1).ok_or_else(|| "глубина ноль".to_string())?;
    let mut prepared = gix::prepare_clone(url, into)
        .map_err(|error| error.to_string())?
        .with_shallow(Shallow::DepthAtRemote(depth));
    let (mut checkout, _) = prepared
        .fetch_then_checkout(Discard, stop)
        .map_err(|error| error.to_string())?;
    checkout
        .main_worktree(Discard, stop)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn wipe(into: &Path) {
    let _ = std::fs::remove_dir_all(into);
}
