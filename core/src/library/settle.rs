use std::fs;
use std::path::Path;

use crate::yaml::is_uuid;

pub(super) fn settle(root: &Path) {
    let Ok(listing) = fs::read_dir(root) else {
        return;
    };
    for item in listing.flatten() {
        let name = item.file_name().to_string_lossy().into_owned();
        let Some(uuid) = displaced(&name) else {
            continue;
        };
        let target = root.join(uuid);
        if fs::symlink_metadata(&target).is_ok() {
            let _ = fs::remove_dir_all(item.path());
        } else {
            let _ = fs::rename(item.path(), &target);
        }
    }
}

fn displaced(name: &str) -> Option<&str> {
    let (uuid, _) = name
        .strip_prefix('.')?
        .strip_suffix(".old")?
        .split_once('.')?;
    is_uuid(uuid).then_some(uuid)
}
