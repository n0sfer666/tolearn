use std::io;
use std::path::Path;

pub use tolearn_core::places::{Places, places};

const CONFIG: [&str; 3] = ["settings.yaml", "provider.yaml", "registry.yaml"];
const DATA: [&str; 3] = ["offline", "unpacked", "history"];
const INDEX: &str = "search-";

pub fn migrate(old: &Path, places: &Places) -> io::Result<()> {
    if !old.is_dir() {
        return Ok(());
    }
    for name in CONFIG {
        moved(&old.join(name), &places.config.join(name))?;
    }
    for name in DATA {
        moved(&old.join(name), &places.data.join(name))?;
    }
    indexes(old, &places.data)
}

fn indexes(old: &Path, data: &Path) -> io::Result<()> {
    for entry in std::fs::read_dir(old)? {
        let path = entry?.path();
        if !index(&path) {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        moved(&path, &data.join(name))?;
    }
    Ok(())
}

fn index(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.starts_with(INDEX) && name.ends_with(".yaml")
}

fn moved(from: &Path, to: &Path) -> io::Result<()> {
    if !from.exists() || to.exists() {
        return Ok(());
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(from, to)
}
