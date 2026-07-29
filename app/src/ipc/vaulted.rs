use std::path::PathBuf;

use tolearn_core::notes::{Note, Stamp};
use tolearn_core::sealed::{Keys, SealError, Sealed, sealed};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::notes::{failed, root};

pub enum Store {
    Plain(PathBuf),
    Locked(Sealed),
}

pub fn store(context: &Context, directory: Option<&String>) -> Result<Store, IpcError> {
    let root = root(context, directory)?;
    if directory.is_none() && sealed(&root) {
        return Ok(Store::Locked(Sealed::new(&root, keys(context)?)));
    }
    Ok(Store::Plain(root))
}

pub fn keys(context: &Context) -> Result<Keys, IpcError> {
    let secret = context
        .keys()
        .key()
        .map_err(|error| IpcError::new("notes.vault", error.to_string()))?
        .ok_or_else(|| {
            IpcError::new(
                "notes.locked",
                "ключ устройства не найден: хранилище открывается парольной фразой".to_owned(),
            )
        })?;
    Keys::parse(&secret).map_err(broke)
}

impl Store {
    pub fn index(&self) -> Result<Vec<Note>, IpcError> {
        match self {
            Self::Plain(root) => tolearn_core::notes::index(root).map_err(failed),
            Self::Locked(store) => store.index().map_err(broke),
        }
    }

    pub fn read(&self, roadmap: &str, topic: &str) -> Result<Option<Note>, IpcError> {
        match self {
            Self::Plain(root) => tolearn_core::notes::read(root, roadmap, topic).map_err(failed),
            Self::Locked(store) => store.read(roadmap, topic).map_err(broke),
        }
    }

    pub fn locked(&self) -> bool {
        matches!(self, Self::Locked(_))
    }

    pub fn sealed(&self) -> Option<&Sealed> {
        match self {
            Self::Plain(_) => None,
            Self::Locked(store) => Some(store),
        }
    }

    pub fn root(&self) -> &std::path::Path {
        match self {
            Self::Plain(root) => root,
            Self::Locked(store) => store.root(),
        }
    }

    pub fn save(
        &self,
        roadmap: &str,
        topic: &str,
        body: &str,
        expected: Option<Stamp>,
    ) -> Result<Written, IpcError> {
        match self {
            Self::Plain(root) => {
                match tolearn_core::notes::save(root, roadmap, topic, body, expected) {
                    Ok(stamp) => Ok(Written::Saved(stamp)),
                    Err(tolearn_core::notes::NoteError::Conflict { theirs, .. }) => {
                        Ok(Written::Conflict(theirs))
                    }
                    Err(error) => Err(failed(error)),
                }
            }
            Self::Locked(store) => match store.save(roadmap, topic, body, expected) {
                Ok(stamp) => Ok(Written::Saved(stamp)),
                Err(SealError::Busy { .. }) => Ok(Written::Conflict(
                    store
                        .read(roadmap, topic)
                        .map_err(broke)?
                        .map(|note| note.body)
                        .unwrap_or_default(),
                )),
                Err(error) => Err(broke(error)),
            },
        }
    }
}

pub enum Written {
    Saved(Stamp),
    Conflict(String),
}

pub fn broke(error: SealError) -> IpcError {
    let code = match error {
        SealError::Unreadable { .. } => "notes.unreadable",
        SealError::Unwritable { .. } => "notes.unwritable",
        SealError::Locked { .. } => "notes.locked",
        SealError::WrongKey => "notes.wrong-key",
        SealError::MalformedKey => "notes.malformed-key",
        SealError::Busy { .. } => "notes.busy",
    };
    IpcError::new(code, error.to_string())
}
