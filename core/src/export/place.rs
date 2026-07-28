use std::path::{Path, PathBuf};

pub fn inside(bundle: &Path, target: &Path) -> bool {
    let (Some(bundle), Some(target)) = (settled(bundle), settled(target)) else {
        return false;
    };
    target.starts_with(&bundle)
}

fn settled(path: &Path) -> Option<PathBuf> {
    if let Ok(found) = path.canonicalize() {
        return Some(found);
    }
    let name = path.file_name()?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    match parent {
        Some(parent) => Some(parent.canonicalize().ok()?.join(name)),
        None => Some(std::env::current_dir().ok()?.join(name)),
    }
}
