use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use super::{Export, ExportError};

pub fn write(
    export: &Export,
    target: &Path,
    fetch: impl Fn(&str) -> Result<Vec<u8>, ExportError>,
) -> Result<usize, ExportError> {
    let taken = vacant(target)?;
    let assets = export
        .assets
        .iter()
        .map(|asset| Ok((asset.to.as_str(), fetch(&asset.from)?)))
        .collect::<Result<Vec<(&str, Vec<u8>)>, ExportError>>()?;
    let staging = staging(target)?;
    let files = export
        .pages
        .iter()
        .map(|page| (page.path.as_str(), page.text.as_bytes()))
        .chain(assets.iter().map(|(to, bytes)| (*to, bytes.as_slice())));
    let written = fill(&staging, target, files)
        .and_then(|count| settle(&staging, target, taken).map(|()| count));
    if written.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    written
}

fn vacant(target: &Path) -> Result<bool, ExportError> {
    let occupied = || ExportError::Occupied {
        path: target.to_owned(),
    };
    match fs::symlink_metadata(target) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(ExportError::unwritable(target, &error)),
        Ok(meta) if !meta.is_dir() => Err(occupied()),
        Ok(_) => {
            let mut entries =
                fs::read_dir(target).map_err(|error| ExportError::unwritable(target, &error))?;
            if entries.next().is_some() {
                return Err(occupied());
            }
            Ok(true)
        }
    }
}

fn staging(target: &Path) -> Result<PathBuf, ExportError> {
    let Some(name) = target.file_name() else {
        return Err(ExportError::Unwritable {
            path: target.to_owned(),
            kind: ErrorKind::InvalidInput,
        });
    };
    let parent = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|error| ExportError::unwritable(parent, &error))?;
    let staging = parent.join(format!(
        ".{}.partial-{}",
        name.to_string_lossy(),
        std::process::id()
    ));
    fs::create_dir(&staging).map_err(|error| ExportError::unwritable(&staging, &error))?;
    Ok(staging)
}

fn fill<'a>(
    staging: &Path,
    target: &Path,
    files: impl Iterator<Item = (&'a str, &'a [u8])>,
) -> Result<usize, ExportError> {
    let mut count = 0;
    let mut folders = BTreeSet::from([staging.to_owned()]);
    for (relative, bytes) in files {
        let path = staging.join(relative);
        let failed =
            |error: std::io::Error| ExportError::unwritable(&target.join(relative), &error);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(failed)?;
            folders.extend(
                parent
                    .ancestors()
                    .take_while(|folder| folder.starts_with(staging))
                    .map(Path::to_owned),
            );
        }
        let mut file = File::create(&path).map_err(failed)?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(failed)?;
        count += 1;
    }
    folders.iter().for_each(|folder| synced(folder));
    Ok(count)
}

fn settle(staging: &Path, target: &Path, taken: bool) -> Result<(), ExportError> {
    let Err(error) = fs::rename(staging, target) else {
        if let Some(parent) = target.parent() {
            synced(parent);
        }
        return Ok(());
    };
    if !taken {
        return Err(ExportError::unwritable(target, &error));
    }
    fs::remove_dir(target).map_err(|_| ExportError::Occupied {
        path: target.to_owned(),
    })?;
    fs::rename(staging, target).map_err(|error| {
        let _ = fs::create_dir(target);
        ExportError::unwritable(target, &error)
    })
}

fn synced(folder: &Path) {
    let _ = File::open(folder).and_then(|folder| folder.sync_all());
}
