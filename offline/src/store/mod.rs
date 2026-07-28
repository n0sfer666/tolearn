mod error;
mod index;
mod types;

pub use error::StoreError;
pub use types::{Fetched, Held, Stored};

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use index::Index;

#[derive(Debug)]
pub struct Store {
    root: PathBuf,
    budget: u64,
    index: Index,
}

impl Store {
    pub fn open(root: &Path, budget: u64) -> Result<Self, StoreError> {
        std::fs::create_dir_all(root.join("objects")).map_err(StoreError::Unwritable)?;
        let index = Index::open(&root.join("index.sqlite"))?;
        Ok(Self {
            root: root.to_path_buf(),
            budget,
            index,
        })
    }

    pub fn put(
        &mut self,
        url: &str,
        program: &str,
        fetched: &Fetched<'_>,
        at: i64,
    ) -> Result<Stored, StoreError> {
        let hash = digest(fetched.bytes);
        let path = self.spot(&hash);
        if !path.exists() {
            let directory = path.parent().unwrap_or(&self.root);
            std::fs::create_dir_all(directory).map_err(StoreError::Unwritable)?;
            std::fs::write(&path, fetched.bytes).map_err(StoreError::Unwritable)?;
        }

        let stale = self.index.find(url)?;
        let held = Held {
            hash: hash.clone(),
            path: path.clone(),
            kind: fetched.kind.to_string(),
            size: fetched.bytes.len() as u64,
            fetched_at: at,
            etag: fetched.etag.map(str::to_string),
            last_modified: fetched.last_modified.map(str::to_string),
        };
        self.index.remember(url, program, &held, at)?;
        if let Some(stale) = stale {
            self.drop_object(&stale.hash)?;
        }

        Ok(Stored {
            hash,
            path,
            size: held.size,
        })
    }

    pub fn get(&mut self, url: &str, at: i64) -> Result<Option<Held>, StoreError> {
        let Some(held) = self.index.find(url)? else {
            return Ok(None);
        };
        self.index.touch(&held.hash, at)?;
        Ok(Some(Held {
            path: self.spot(&held.hash),
            ..held
        }))
    }

    pub fn protect(&mut self, program: &str, protected: bool) -> Result<(), StoreError> {
        self.index.protect(program, protected)
    }

    pub fn size(&self) -> Result<u64, StoreError> {
        self.index.size()
    }

    pub fn sweep(&mut self) -> Result<Vec<String>, StoreError> {
        let mut total = self.index.size()?;
        let mut evicted = Vec::new();
        for (url, hash, size) in self.index.loose()? {
            if total <= self.budget {
                break;
            }
            self.index.forget(&url, &hash)?;
            self.drop_object(&hash)?;
            total = total.saturating_sub(size);
            evicted.push(url);
        }
        Ok(evicted)
    }

    fn spot(&self, hash: &str) -> PathBuf {
        self.root.join("objects").join(&hash[0..2]).join(hash)
    }

    fn drop_object(&self, hash: &str) -> Result<(), StoreError> {
        if self.index.holds(hash)? {
            return Ok(());
        }
        let path = self.spot(hash);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(StoreError::Unwritable(error)),
        }
    }
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
