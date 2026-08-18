use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

static MADE: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct Scratch {
    path: PathBuf,
}

impl Scratch {
    pub fn new() -> Result<Self, io::Error> {
        let path = std::env::temp_dir().join(format!(
            "tolearn-harness-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
