use std::path::PathBuf;

use tolearn_core::notes::{Note, Stamp};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::notes::{failed, root};

pub struct Store(PathBuf);

pub fn store(context: &Context, directory: Option<&String>) -> Result<Store, IpcError> {
    Ok(Store(root(context, directory)?))
}

impl Store {
    pub fn read(&self, roadmap: &str, topic: &str) -> Result<Option<Note>, IpcError> {
        tolearn_core::notes::read(&self.0, roadmap, topic).map_err(failed)
    }

    pub fn save(
        &self,
        roadmap: &str,
        topic: &str,
        body: &str,
        expected: Option<Stamp>,
    ) -> Result<Written, IpcError> {
        match tolearn_core::notes::save(&self.0, roadmap, topic, body, expected) {
            Ok(stamp) => Ok(Written::Saved(stamp)),
            Err(tolearn_core::notes::NoteError::Conflict { theirs, .. }) => {
                Ok(Written::Conflict(theirs))
            }
            Err(error) => Err(failed(error)),
        }
    }
}

pub enum Written {
    Saved(Stamp),
    Conflict(String),
}
