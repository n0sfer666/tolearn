mod disk;
mod error;
mod index;
mod migrate;
mod types;

pub use error::StoreError;
pub use types::{Checked, Fetched, Held, Stored};

use std::path::{Path, PathBuf};

use crate::digest::digest;

use index::{Found, Index};

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

        let size = fetched.bytes.len() as u64;
        let held = Held {
            hash: hash.clone(),
            path: path.clone(),
            kind: fetched.kind.to_string(),
            size,
            fetched_at: at,
            etag: fetched.etag.map(str::to_string),
            last_modified: fetched.last_modified.map(str::to_string),
            body_hash: None,
            checked_at: None,
        };
        self.write(url, program, held, None, at)?;

        Ok(Stored { hash, path, size })
    }

    pub fn corner(&self, url: &str) -> Result<PathBuf, StoreError> {
        let path = self.nook(&kept(url));
        std::fs::create_dir_all(&path).map_err(StoreError::Unwritable)?;
        Ok(path)
    }

    pub fn keep(
        &mut self,
        url: &str,
        program: &str,
        kind: &str,
        at: i64,
    ) -> Result<Stored, StoreError> {
        let hash = kept(url);
        let path = self.nook(&hash);
        let size = disk::weigh(&path)?;
        let held = Held {
            hash: hash.clone(),
            path: path.clone(),
            kind: kind.to_string(),
            size,
            fetched_at: at,
            etag: None,
            last_modified: None,
            body_hash: None,
            checked_at: None,
        };
        self.write(url, program, held, Some(path.clone()), at)?;

        Ok(Stored { hash, path, size })
    }

    pub fn get(&mut self, url: &str, at: i64) -> Result<Option<Held>, StoreError> {
        let Some(found) = self.index.find(url)? else {
            return Ok(None);
        };
        self.index.touch(&found.held.hash, at)?;
        Ok(Some(self.at_hand(found)))
    }

    pub fn held(&self, url: &str) -> Result<Option<Held>, StoreError> {
        Ok(self.index.find(url)?.map(|found| self.at_hand(found)))
    }

    pub fn stamp(&mut self, url: &str, checked: &Checked) -> Result<(), StoreError> {
        self.index.stamp(url, checked)
    }

    pub fn checked(&mut self, url: &str, at: i64) -> Result<(), StoreError> {
        self.index.checked(url, at)
    }

    pub fn protect(&mut self, program: &str, protected: bool) -> Result<(), StoreError> {
        self.index.protect(program, protected)
    }

    pub fn size(&self) -> Result<u64, StoreError> {
        self.index.size()
    }

    pub fn spare(&self) -> Result<u64, StoreError> {
        self.index.spare()
    }

    pub fn sweep(&mut self) -> Result<Vec<String>, StoreError> {
        let mut total = self.index.size()?;
        let mut evicted = Vec::new();
        for (url, found) in self.index.loose()? {
            if total <= self.budget {
                break;
            }
            let size = found.held.size;
            self.index.forget(&url, &found.held.hash)?;
            self.drop_object(&found)?;
            total = total.saturating_sub(size);
            evicted.push(url);
        }
        Ok(evicted)
    }

    fn write(
        &mut self,
        url: &str,
        program: &str,
        held: Held,
        outside: Option<PathBuf>,
        at: i64,
    ) -> Result<(), StoreError> {
        let stale = self.index.find(url)?;
        self.index
            .remember(url, program, &Found { held, outside }, at)?;
        if let Some(stale) = stale {
            self.drop_object(&stale)?;
        }
        Ok(())
    }

    fn at_hand(&self, found: Found) -> Held {
        let path = found.outside.unwrap_or_else(|| self.spot(&found.held.hash));
        Held { path, ..found.held }
    }

    fn spot(&self, hash: &str) -> PathBuf {
        self.root.join("objects").join(&hash[0..2]).join(hash)
    }

    fn nook(&self, hash: &str) -> PathBuf {
        self.root.join("artifacts").join(&hash[0..2]).join(hash)
    }

    fn drop_object(&self, found: &Found) -> Result<(), StoreError> {
        if self.index.holds(&found.held.hash)? {
            return Ok(());
        }
        match found.outside.as_ref() {
            Some(path) => disk::erase(path),
            None => disk::erase(&self.spot(&found.held.hash)),
        }
    }
}

fn kept(url: &str) -> String {
    digest(format!("kept:{url}").as_bytes())
}
