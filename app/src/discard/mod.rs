mod bin;

pub use bin::Trash;

use std::path::{Path, PathBuf};

use tolearn_core::library::PROGRAMS;
use tolearn_core::state::STATE;
use tolearn_generate::sources::CACHE;

use crate::journal::ROOM;

pub trait Bin: std::fmt::Debug + Send + Sync {
    fn discard(&self, path: &Path) -> Result<(), String>;
}

pub fn plain(uuid: &str) -> bool {
    !uuid.is_empty()
        && uuid
            .chars()
            .all(|sign| sign.is_ascii_hexdigit() || sign == '-')
}

pub fn discard(
    bin: &dyn Bin,
    data: &Path,
    root: &str,
    programs: &[String],
) -> Result<(), Vec<String>> {
    let content = PathBuf::from(PROGRAMS).join(root);
    let rooms = belongings(programs);
    if let Err(unmoved) = moved(bin, data, &content) {
        return Err(stayed(data, unmoved, &rooms));
    }
    let mut left = Vec::new();
    for room in rooms {
        if let Err(unmoved) = moved(bin, data, &room) {
            left.push(unmoved);
        }
    }
    if left.is_empty() { Ok(()) } else { Err(left) }
}

fn belongings(programs: &[String]) -> Vec<PathBuf> {
    let mut rooms = Vec::with_capacity(programs.len() * 2 + 1);
    for uuid in programs {
        rooms.push(PathBuf::from(STATE).join(uuid));
        rooms.push(PathBuf::from(CACHE).join(uuid));
    }
    rooms.push(PathBuf::from(ROOM));
    rooms
}

fn stayed(data: &Path, unmoved: String, rooms: &[PathBuf]) -> Vec<String> {
    let mut left = vec![unmoved];
    left.extend(
        rooms
            .iter()
            .filter(|room| data.join(room).exists())
            .map(|room| room.display().to_string()),
    );
    left
}

fn moved(bin: &dyn Bin, data: &Path, room: &Path) -> Result<(), String> {
    let path = data.join(room);
    if !path.exists() {
        return Ok(());
    }
    bin.discard(&path)
        .map_err(|reason| format!("{} ({reason})", room.display()))
}

#[cfg(test)]
mod tests {
    use super::plain;

    #[test]
    fn нечитаемый_uuid_в_путь_корзины_не_попадает() {
        assert!(plain("7a1d4e90-2c3b-4f58-8d6e-1b9a0c5e7f23"));
        assert!(!plain(""));
        assert!(!plain("../../etc"));
        assert!(!plain("nes dev"));
    }
}
