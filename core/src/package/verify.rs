use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::unpack_error::UnpackError;
use super::{MANIFEST, hex, manifest, svg};
use crate::archive::{self, Limits};
use crate::library::{self, Refusal};
use crate::program::{LoadError, Tree};

pub(super) fn open(package: &Path, scratch: &Path) -> Result<Tree, UnpackError> {
    archive::unzip(package, scratch, &Limits::DEFAULT).map_err(|error| UnpackError::Archive {
        package: package.to_owned(),
        error,
    })?;
    let listed = manifest::read(&listing(scratch)?)?;
    let mut found = BTreeSet::new();
    gather(scratch, Path::new(""), &mut found)?;
    found.remove(MANIFEST);
    sealed(scratch, &listed, &found)?;
    let tree = library::read(scratch).map_err(|refusal| relative(refusal, scratch))?;
    let files: BTreeSet<String> = tree.files().into_iter().collect();
    if let Some(file) = listed.keys().find(|file| !files.contains(*file)) {
        return Err(UnpackError::Unused { file: file.clone() });
    }
    for file in files
        .iter()
        .filter(|file| file.to_ascii_lowercase().ends_with(".svg"))
    {
        svg::check(&read(scratch, file)?).map_err(|reason| UnpackError::UnsafeSvg {
            file: file.clone(),
            reason,
        })?;
    }
    Ok(tree)
}

pub(super) fn relative(refusal: Refusal, scratch: &Path) -> UnpackError {
    let inside = |path: &Path| path.strip_prefix(scratch).unwrap_or(path).to_path_buf();
    UnpackError::Refused(match refusal {
        Refusal::Unloadable(LoadError::Unreadable { path, kind }) => {
            Refusal::Unloadable(LoadError::Unreadable {
                path: inside(&path),
                kind,
            })
        }
        Refusal::Unloadable(LoadError::Malformed { path, error }) => {
            Refusal::Unloadable(LoadError::Malformed {
                path: inside(&path),
                error,
            })
        }
        other => other,
    })
}

fn listing(scratch: &Path) -> Result<Vec<u8>, UnpackError> {
    fs::read(scratch.join(MANIFEST)).map_err(|error| match error.kind() {
        ErrorKind::NotFound => UnpackError::NoManifest,
        kind => UnpackError::BadManifest {
            reason: kind.to_string(),
        },
    })
}

fn gather(
    scratch: &Path,
    relative: &Path,
    found: &mut BTreeSet<String>,
) -> Result<(), UnpackError> {
    let unreadable = |error: io::Error| UnpackError::Unreadable {
        path: relative.to_owned(),
        kind: error.kind(),
    };
    for entry in fs::read_dir(scratch.join(relative)).map_err(unreadable)? {
        let entry = entry.map_err(unreadable)?;
        let name = relative.join(entry.file_name());
        if entry.file_type().map_err(unreadable)?.is_dir() {
            gather(scratch, &name, found)?;
        } else {
            found.insert(name.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn sealed(
    scratch: &Path,
    listed: &BTreeMap<String, String>,
    found: &BTreeSet<String>,
) -> Result<(), UnpackError> {
    if let Some(file) = found.iter().find(|file| !listed.contains_key(*file)) {
        return Err(UnpackError::Unlisted { file: file.clone() });
    }
    for (file, sum) in listed {
        if !found.contains(file) {
            return Err(UnpackError::Missing { file: file.clone() });
        }
        if hex(&read(scratch, file)?) != *sum {
            return Err(UnpackError::Checksum { file: file.clone() });
        }
    }
    Ok(())
}

fn read(scratch: &Path, file: &str) -> Result<Vec<u8>, UnpackError> {
    fs::read(scratch.join(file)).map_err(|error| UnpackError::Unreadable {
        path: PathBuf::from(file),
        kind: error.kind(),
    })
}
