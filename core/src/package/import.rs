use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::scratch::scratch;
use super::unpack_error::UnpackError;
use super::{renew, verify};
use crate::library::Library;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    pub uuid: String,
    pub copy_of: Option<String>,
}

pub fn import(package: &Path, library: &Library) -> Result<Imported, UnpackError> {
    let root = library.root();
    let made = fs::symlink_metadata(root).is_err();
    fs::create_dir_all(root).map_err(|error| UnpackError::Unwritable {
        path: root.to_owned(),
        kind: error.kind(),
    })?;
    let imported = scratch(root, "").and_then(|scratch| {
        let imported = settle(package, library, &scratch);
        let _ = fs::remove_dir_all(&scratch);
        imported
    });
    if made {
        let _ = fs::remove_dir(root);
    }
    imported
}

fn settle(package: &Path, library: &Library, scratch: &Path) -> Result<Imported, UnpackError> {
    let tree = verify::open(package, scratch)?;
    let copy_of = if tree.uuids().is_disjoint(&taken(library)?) {
        None
    } else {
        renew::renew(&tree, scratch)?;
        Some(tree.program.uuid)
    };
    let uuid = library.install(scratch).map_err(UnpackError::Library)?;
    Ok(Imported { uuid, copy_of })
}

fn taken(library: &Library) -> Result<BTreeSet<String>, UnpackError> {
    let mut taken = BTreeSet::new();
    for entry in library.list().map_err(UnpackError::Library)? {
        if let Ok(tree) = &entry.program {
            taken.extend(tree.uuids());
        }
        taken.insert(entry.directory);
    }
    Ok(taken)
}
