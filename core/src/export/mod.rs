mod anchors;
mod ask;
mod head;
mod lines;
mod note;
mod parts;
mod place;
mod section;

use std::collections::BTreeMap;

use crate::notes::Note;
use crate::roadmap::Roadmap;
use crate::status::Statuses;
use crate::topic::Topic;

use anchors::Anchors;
use lines::Doc;
use section::Sight;

pub use place::inside;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub anchor: String,
    pub title: String,
}

#[derive(Debug)]
pub struct Placed {
    pub id: String,
    pub title: String,
    pub anchor: String,
}

#[derive(Debug)]
pub struct Chapter {
    pub title: String,
    pub anchor: String,
    pub topics: Vec<Placed>,
}

pub fn markdown(
    roadmap: &Roadmap,
    topics: &[Topic],
    statuses: &Statuses,
    notes: &[Note],
) -> String {
    let chapters = layout(roadmap);
    let links = links(&chapters);
    let mut doc = Doc::default();
    head::head(&mut doc, roadmap, &chapters);
    for chapter in &chapters {
        doc.heading(2, &chapter.title);
        for placed in &chapter.topics {
            let Some(entry) = roadmap.topics.iter().find(|entry| entry.id == placed.id) else {
                continue;
            };
            let sight = Sight {
                links: &links,
                status: statuses.get(&placed.id),
                notes,
            };
            let document = topics.iter().find(|topic| topic.id == placed.id);
            section::section(&mut doc, entry, document, &sight);
        }
    }
    doc.text()
}

fn layout(roadmap: &Roadmap) -> Vec<Chapter> {
    let mut anchors = Anchors::default();
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut placed: Vec<String> = Vec::new();
    for stage in &roadmap.stages {
        let title = format!("Этап {} — {}", stage.n, stage.title);
        let anchor = anchors.take(&title);
        let topics = held(roadmap, &mut anchors, |entry| entry.stage == stage.n);
        placed.extend(topics.iter().map(|topic| topic.id.clone()));
        chapters.push(Chapter {
            title,
            anchor,
            topics,
        });
    }
    let left = held(roadmap, &mut anchors, |entry| !placed.contains(&entry.id));
    if !left.is_empty() {
        let title = "Вне этапов".to_owned();
        chapters.push(Chapter {
            anchor: anchors.take(&title),
            title,
            topics: left,
        });
    }
    chapters
}

fn held(
    roadmap: &Roadmap,
    anchors: &mut Anchors,
    fits: impl Fn(&crate::roadmap::TopicEntry) -> bool,
) -> Vec<Placed> {
    roadmap
        .topics
        .iter()
        .filter(|entry| fits(entry))
        .map(|entry| Placed {
            id: entry.id.clone(),
            title: entry.title.clone(),
            anchor: anchors.take(&entry.title),
        })
        .collect()
}

fn links(chapters: &[Chapter]) -> BTreeMap<String, Link> {
    chapters
        .iter()
        .flat_map(|chapter| chapter.topics.iter())
        .map(|placed| {
            (
                placed.id.clone(),
                Link {
                    anchor: placed.anchor.clone(),
                    title: placed.title.clone(),
                },
            )
        })
        .collect()
}
