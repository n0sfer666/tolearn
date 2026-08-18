use std::path::Path;

use tolearn_core::atomic;
use tolearn_core::roadmap::{Roadmap, parse as read_roadmap};
use tolearn_core::topic::{Topic, parse as read_topic};

const ROADMAP: &str = "roadmap.yaml";
const PROGRESS: &str = "progress.yaml";

#[derive(Debug)]
pub struct Draft {
    pub map: Roadmap,
    pub map_text: String,
    pub progress: String,
    pub got: Vec<Option<(Topic, String)>>,
}

#[derive(Debug, Clone)]
pub struct Waiting {
    pub id: String,
    pub title: String,
    pub total: usize,
    pub done: usize,
}

pub fn keep(root: &Path, map_text: &str, progress: &str) -> std::io::Result<()> {
    forget(root);
    std::fs::create_dir_all(root)?;
    atomic::write(&root.join(ROADMAP), map_text)?;
    atomic::write(&root.join(PROGRESS), progress)
}

pub fn note(root: &Path, file: &str, text: &str) -> std::io::Result<()> {
    let path = root.join(file);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    atomic::write(&path, text)
}

pub fn erase(root: &Path, file: &str) {
    let _ = std::fs::remove_file(root.join(file));
}

pub fn read(root: &Path) -> Option<Draft> {
    let map_text = std::fs::read_to_string(root.join(ROADMAP)).ok()?;
    let progress = std::fs::read_to_string(root.join(PROGRESS)).ok()?;
    let map = read_roadmap(&map_text).ok()?;
    let got = map
        .topics
        .iter()
        .map(|entry| kept(root, &entry.file, &entry.id))
        .collect();
    Some(Draft {
        map,
        map_text,
        progress,
        got,
    })
}

pub fn waiting(root: &Path) -> Option<Waiting> {
    let draft = read(root)?;
    Some(Waiting {
        id: draft.map.id,
        title: draft.map.title,
        total: draft.got.len(),
        done: draft.got.iter().flatten().count(),
    })
}

pub fn forget(root: &Path) {
    let _ = std::fs::remove_dir_all(root);
}

fn kept(root: &Path, file: &str, id: &str) -> Option<(Topic, String)> {
    let text = std::fs::read_to_string(root.join(file)).ok()?;
    let topic = read_topic(&text).ok()?;
    (topic.id == id).then_some((topic, text))
}
