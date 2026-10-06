use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use uuid::Uuid;

use super::unpack_error::UnpackError;
use super::verify::relative;
use crate::library::{self, LibraryError, Refusal};
use crate::program::{self, LoadError, Tree};
use crate::yaml::ParseError;

pub(super) fn renew(tree: &Tree, scratch: &Path) -> Result<(), UnpackError> {
    let fresh: BTreeMap<String, String> = tree
        .uuids()
        .into_iter()
        .map(|old| (old, Uuid::new_v4().to_string()))
        .collect();
    rewrite(tree, scratch, Path::new(""), &fresh)?;
    let renewed = library::read(scratch).map_err(|refusal| relative(refusal, scratch))?;
    match renewed
        .uuids()
        .into_iter()
        .find(|uuid| fresh.contains_key(uuid))
    {
        Some(uuid) => Err(UnpackError::Library(LibraryError::Taken { uuid })),
        None => Ok(()),
    }
}

fn rewrite(
    tree: &Tree,
    scratch: &Path,
    directory: &Path,
    fresh: &BTreeMap<String, String>,
) -> Result<(), UnpackError> {
    let children = directory.join("children");
    for (name, child) in &tree.children {
        let old = children.join(name);
        rewrite(child, scratch, &old, fresh)?;
        if let Some(uuid) = fresh.get(name) {
            let new = children.join(uuid);
            fs::rename(scratch.join(&old), scratch.join(&new))
                .map_err(|error| unwritable(&new, &error))?;
        }
    }
    let map = directory.join("program.yaml");
    let path = scratch.join(&map);
    let text = fs::read_to_string(&path).map_err(|error| UnpackError::Unreadable {
        path: map.clone(),
        kind: error.kind(),
    })?;
    let text = spliced(&text, fresh).map_err(|error| {
        UnpackError::Refused(Refusal::Unloadable(LoadError::Malformed {
            path: map.clone(),
            error,
        }))
    })?;
    fs::write(&path, text).map_err(|error| unwritable(&map, &error))
}

fn spliced(text: &str, fresh: &BTreeMap<String, String>) -> Result<String, ParseError> {
    let (mark, body) = text
        .strip_prefix('\u{feff}')
        .map_or(("", text), |body| ("\u{feff}", body));
    let offsets: Vec<usize> = body
        .char_indices()
        .map(|(offset, _)| offset)
        .chain([body.len()])
        .collect();
    let mut places = program::places(body)?;
    places.sort_by_key(|(chars, _)| Reverse(chars.start));
    places.dedup_by_key(|(chars, _)| chars.start);
    let mut out = body.to_owned();
    for (chars, old) in places {
        let (Some(&start), Some(&end), Some(new)) = (
            offsets.get(chars.start),
            offsets.get(chars.end),
            fresh.get(&old),
        ) else {
            continue;
        };
        let raw = &body[start..end];
        let written = if raw.starts_with(['"', '\'']) {
            format!("\"{new}\"")
        } else {
            format!("{new}{}", &raw[raw.trim_end().len()..])
        };
        out.replace_range(start..end, &written);
    }
    Ok(format!("{mark}{out}"))
}

fn unwritable(path: &Path, error: &io::Error) -> UnpackError {
    UnpackError::Unwritable {
        path: path.to_owned(),
        kind: error.kind(),
    }
}
