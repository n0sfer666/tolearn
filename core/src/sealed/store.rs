use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::notes::frontmatter::{parse, render};
use crate::notes::{Note, Stamp};

use super::crypt::{open, seal};
use super::error::SealError;
use super::keys::Keys;

pub const SUFFIX: &str = "age";
pub const IDENTITY: &str = "identity.age";
const BLOBS: &str = "blobs";

#[derive(Debug, Clone)]
pub struct Sealed {
    root: PathBuf,
    keys: Keys,
}

impl Sealed {
    pub fn new(root: &Path, keys: Keys) -> Self {
        Self {
            root: root.to_path_buf(),
            keys,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn index(&self) -> Result<Vec<Note>, SealError> {
        let mut found = Vec::new();
        for path in self.files()? {
            let text = String::from_utf8(open(&read(&path)?, self.keys.identity())?)
                .map_err(|_| SealError::WrongKey)?;
            let Some(bound) = parse(&text) else {
                continue;
            };
            found.push(Note {
                roadmap: bound.roadmap,
                topic: bound.topic,
                stamp: stamp(&path)?,
                path,
                body: bound.body,
            });
        }
        found.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(found)
    }

    pub fn read(&self, roadmap: &str, topic: &str) -> Result<Option<Note>, SealError> {
        Ok(self
            .index()?
            .into_iter()
            .find(|note| note.roadmap == roadmap && note.topic == topic))
    }

    pub fn save(
        &self,
        roadmap: &str,
        topic: &str,
        body: &str,
        expected: Option<Stamp>,
    ) -> Result<Stamp, SealError> {
        let existing = self.read(roadmap, topic)?;
        if let (Some(note), Some(stamp)) = (&existing, expected)
            && note.stamp != stamp
        {
            return Err(SealError::Busy {
                path: note.path.display().to_string(),
            });
        }
        let path = match existing {
            Some(note) => note.path,
            None => self.root.join(format!("{}.{SUFFIX}", name())),
        };
        self.put(&path, render(roadmap, topic, body).as_bytes())?;
        stamp(&path)
    }

    pub fn blob(&self, name: &str) -> Result<Option<String>, SealError> {
        let path = self.root.join(BLOBS).join(format!("{name}.{SUFFIX}"));
        if !path.exists() {
            return Ok(None);
        }
        let plain = open(&read(&path)?, self.keys.identity())?;
        String::from_utf8(plain)
            .map(Some)
            .map_err(|_| SealError::WrongKey)
    }

    pub fn keep(&self, name: &str, text: &str) -> Result<(), SealError> {
        let path = self.root.join(BLOBS).join(format!("{name}.{SUFFIX}"));
        self.put(&path, text.as_bytes())
    }

    pub fn put(&self, path: &Path, plain: &[u8]) -> Result<(), SealError> {
        let cipher = seal(plain, self.keys.identity())?;
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory).map_err(|error| unwritable(path, &error))?;
        }
        crate::atomic::bytes(path, &cipher).map_err(|error| unwritable(path, &error))
    }

    fn files(&self) -> Result<Vec<PathBuf>, SealError> {
        let listing = match std::fs::read_dir(&self.root) {
            Ok(listing) => listing,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(unreadable(&self.root, &error)),
        };
        let mut found = Vec::new();
        for entry in listing {
            let path = entry
                .map_err(|error| unreadable(&self.root, &error))?
                .path();
            if path.is_file()
                && path.extension().is_some_and(|kind| kind == SUFFIX)
                && path.file_name().is_some_and(|name| name != IDENTITY)
            {
                found.push(path);
            }
        }
        Ok(found)
    }
}

pub fn name() -> String {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).unwrap_or_default();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read(path: &Path) -> Result<Vec<u8>, SealError> {
    std::fs::read(path).map_err(|error| unreadable(path, &error))
}

fn stamp(path: &Path) -> Result<Stamp, SealError> {
    let data = std::fs::metadata(path).map_err(|error| unreadable(path, &error))?;
    let modified = data
        .modified()
        .map_err(|error| unreadable(path, &error))?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    Ok(Stamp {
        modified_nanos: modified.as_nanos(),
        size: data.len(),
    })
}

fn unreadable(path: &Path, error: &std::io::Error) -> SealError {
    SealError::Unreadable {
        path: path.display().to_string(),
        reason: error.to_string(),
    }
}

fn unwritable(path: &Path, error: &std::io::Error) -> SealError {
    SealError::Unwritable {
        path: path.display().to_string(),
        reason: error.to_string(),
    }
}
