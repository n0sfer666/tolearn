use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tolearn_core::exam::{Line, Side};
use tolearn_core::progress::{Answer, Outcome};
use tolearn_core::topic::Topic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kept {
    pub side: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grade {
    pub id: String,
    pub result: String,
    pub quote: Option<String>,
    pub missed: Vec<String>,
    pub signal_extension: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Dialog {
    pub topic: String,
    pub fingerprint: String,
    pub artifact: Option<String>,
    pub failed_checks: Vec<String>,
    pub at: usize,
    pub lines: Vec<Kept>,
    pub log: Vec<Kept>,
    pub graded: Vec<Grade>,
    pub hinted: Vec<String>,
    pub followed: bool,
    pub exchanges: u32,
    pub seconds: u64,
    pub tokens: u32,
    pub verdict: Option<String>,
}

impl Dialog {
    pub fn fresh(topic: &Topic) -> Self {
        Self {
            topic: topic.id.clone(),
            fingerprint: fingerprint(topic),
            ..Self::default()
        }
    }

    pub fn stale(&self, topic: &Topic) -> bool {
        self.fingerprint != fingerprint(topic)
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

    pub fn answers(&self) -> Vec<Answer> {
        answers(&self.graded)
    }

    pub fn grade(&mut self, answer: &Answer) {
        self.graded.push(scored(answer));
    }
}

pub fn answers(graded: &[Grade]) -> Vec<Answer> {
    graded
        .iter()
        .map(|grade| Answer {
            id: grade.id.clone(),
            outcome: Outcome::read(&grade.result).unwrap_or(Outcome::Miss),
            quote: grade.quote.clone(),
            missed: grade.missed.clone(),
            signal_extension: grade.signal_extension,
        })
        .collect()
}

pub fn scored(answer: &Answer) -> Grade {
    Grade {
        id: answer.id.clone(),
        result: answer.outcome.label().to_owned(),
        quote: answer.quote.clone(),
        missed: answer.missed.clone(),
        signal_extension: answer.signal_extension,
    }
}

pub fn fingerprint(topic: &Topic) -> String {
    let mut digest = Sha256::new();
    digest.update(topic.id.as_bytes());
    digest.update(topic.title.as_bytes());
    digest.update(topic.practice.task.as_bytes());
    digest.update(topic.exam.focus.as_bytes());
    for question in &topic.questions {
        digest.update(question.id.as_bytes());
        digest.update(question.text.as_bytes());
    }
    format!("{:x}", digest.finalize())
}
