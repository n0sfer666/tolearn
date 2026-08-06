use std::path::Path;

use tolearn_core::atomic;
use tolearn_core::roadmap::Roadmap;

const EXAMINER: &str = include_str!("../../assets/examiner.md");
const SHOWN: usize = 3;

#[derive(Debug, Clone)]
pub struct Made {
    pub id: String,
    pub title: String,
    pub roadmap: String,
    pub progress: String,
    pub topics: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub id: String,
    pub title: String,
    pub topics: usize,
    pub hours_min: u32,
    pub hours_max: u32,
    pub stages: Vec<Staged>,
}

#[derive(Debug, Clone, Default)]
pub struct Staged {
    pub n: u32,
    pub title: String,
    pub topics: usize,
    pub first: Vec<String>,
}

impl Made {
    pub fn write(&self, root: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(root)?;
        atomic::write(&root.join("roadmap.yaml"), &self.roadmap)?;
        atomic::write(&root.join("progress.yaml"), &self.progress)?;
        atomic::write(&root.join("examiner.md"), EXAMINER)?;
        for (file, text) in &self.topics {
            let path = root.join(file);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            atomic::write(&path, text)?;
        }
        Ok(())
    }
}

pub fn summary(map: &Roadmap) -> Summary {
    Summary {
        id: map.id.clone(),
        title: map.title.clone(),
        topics: map.topics.len(),
        hours_min: map.topics.iter().map(|entry| entry.est_hours.min).sum(),
        hours_max: map.topics.iter().map(|entry| entry.est_hours.max).sum(),
        stages: map
            .stages
            .iter()
            .map(|stage| staged(map, stage.n))
            .collect(),
    }
}

fn staged(map: &Roadmap, n: u32) -> Staged {
    let inside: Vec<&str> = map
        .topics
        .iter()
        .filter(|entry| entry.stage == n)
        .map(|entry| entry.title.as_str())
        .collect();
    Staged {
        n,
        title: map
            .stages
            .iter()
            .find(|stage| stage.n == n)
            .map(|stage| stage.title.clone())
            .unwrap_or_default(),
        topics: inside.len(),
        first: inside
            .iter()
            .take(SHOWN)
            .map(|title| (*title).to_owned())
            .collect(),
    }
}
