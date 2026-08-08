use serde::{Deserialize, Serialize};
use tolearn_core::exam::{Line, Side};
use tolearn_core::progress::Answer;
use tolearn_core::sweep::Picked;
use tolearn_core::topic::Topic;

use crate::exam::{Grade, Kept, answers, fingerprint, scored};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Leg {
    pub topic: String,
    pub title: String,
    pub fingerprint: String,
    pub at: usize,
    pub total: usize,
    pub graded: Vec<Grade>,
    pub hinted: Vec<String>,
    pub verdict: Option<String>,
}

impl Leg {
    pub fn left(&self) -> bool {
        self.at < self.total
    }

    pub fn answers(&self) -> Vec<Answer> {
        answers(&self.graded)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Run {
    pub roadmap: String,
    pub legs: Vec<Leg>,
    pub at: usize,
    pub lines: Vec<Kept>,
    pub log: Vec<Kept>,
    pub followed: bool,
    pub seconds: u64,
    pub tokens: u32,
    pub done: bool,
}

impl Run {
    pub fn fresh(roadmap: &str, picked: &[Picked], topics: &[Topic]) -> Self {
        Self {
            roadmap: roadmap.to_owned(),
            legs: picked.iter().filter_map(|pick| leg(pick, topics)).collect(),
            ..Self::default()
        }
    }

    pub fn stale(&self, topics: &[Topic]) -> bool {
        self.legs.iter().any(|leg| {
            topics
                .iter()
                .find(|topic| topic.id == leg.topic)
                .is_none_or(|topic| fingerprint(topic) != leg.fingerprint)
        })
    }

    pub fn tell(&mut self, side: Side, text: &str) {
        let kept = Kept {
            side: side.label().to_owned(),
            text: text.trim().to_owned(),
        };
        self.lines.push(kept.clone());
        self.log.push(kept);
    }

    pub fn spent(&mut self, seconds: u64, tokens: Option<u32>) {
        self.seconds += seconds;
        self.tokens += tokens.unwrap_or_default();
    }

    pub fn turned(&mut self) {
        self.lines.clear();
        self.followed = false;
    }

    pub fn said(&self) -> Vec<Line> {
        self.lines
            .iter()
            .map(|kept| Line {
                side: Side::read(&kept.side).unwrap_or(Side::Student),
                text: kept.text.clone(),
            })
            .collect()
    }

    pub fn grade(&mut self, at: usize, answer: &Answer) {
        let Some(leg) = self.legs.get_mut(at) else {
            return;
        };
        leg.graded.push(scored(answer));
        leg.at += 1;
        self.at = at + 1;
    }

    pub fn asked(&self) -> usize {
        self.legs.iter().map(|leg| leg.graded.len()).sum()
    }

    pub fn total(&self) -> usize {
        self.legs.iter().map(|leg| leg.total).sum()
    }
}

fn leg(pick: &Picked, topics: &[Topic]) -> Option<Leg> {
    let topic = topics.iter().find(|topic| topic.id == pick.topic)?;
    Some(Leg {
        topic: topic.id.clone(),
        title: topic.title.clone(),
        fingerprint: fingerprint(topic),
        total: topic.questions.len(),
        ..Leg::default()
    })
}
