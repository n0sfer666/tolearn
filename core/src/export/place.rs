use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn inside(root: &Path, target: &Path) -> bool {
    let (Some(root), Some(target)) = (settled(root), settled(target)) else {
        return true;
    };
    target.starts_with(&root)
}

pub fn claimed(target: &Path) -> bool {
    settled(target).is_none_or(|target| {
        target
            .ancestors()
            .any(|folder| folder.join("program.yaml").exists())
    })
}

fn settled(path: &Path) -> Option<PathBuf> {
    let absolute = std::path::absolute(path).ok()?;
    let mut settled = PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::ParentDir => {
                settled.pop();
            }
            Component::CurDir => {}
            other => settled.push(other),
        }
        if fs::symlink_metadata(&settled).is_ok() {
            settled = settled.canonicalize().ok()?;
        }
    }
    Some(settled)
}
