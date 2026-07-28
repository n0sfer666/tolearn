use saphyr::{LoadableYamlNode, Yaml};

use super::enums::Status;
use super::error::DocumentError;
use super::mark::{self, Mark};
use super::node;
use super::parse::parse;
use super::render::{Format, render};
use super::types::{Attempt, Progress, TopicState};

#[derive(Debug, Clone)]
pub struct Document {
    text: String,
    format: Format,
    progress: Progress,
}

impl Document {
    pub fn read(source: &str, format: Format) -> Result<Self, DocumentError> {
        let text = with_root(source, |root| render(root, format))?;
        let progress = parse(&text)?;
        Ok(Self {
            text,
            format,
            progress,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    pub fn push_attempt(&mut self, topic: &str, attempt: &Attempt) -> Result<(), DocumentError> {
        let node = node::attempt(attempt);
        let text = with_root(&self.text, |root| {
            append(root, topic, node)?;
            render(root, self.format)
        })?;
        self.progress = parse(&text)?;
        self.text = text;
        Ok(())
    }

    pub fn mark(&mut self, topic: &str, mark: &Mark) -> Result<(), DocumentError> {
        let node = node::attempt(&mark::attempt(mark));
        let text = with_root(&self.text, |root| {
            append(root, topic, node)?;
            set(root, topic, "status", Some(mark.status.label()))?;
            set(root, topic, "passed_at", mark.passed_at.as_deref())?;
            set(
                root,
                topic,
                "next_review_at",
                mark.next_review_at.as_deref(),
            )?;
            render(root, self.format)
        })?;
        self.progress = parse(&text)?;
        self.text = text;
        Ok(())
    }

    pub fn settle(&mut self, topic: &str, state: &TopicState) -> Result<(), DocumentError> {
        self.edit(|root, format| {
            set(root, topic, "status", Some(state.status.label()))?;
            set(root, topic, "passed_at", state.passed_at.as_deref())?;
            set(
                root,
                topic,
                "next_review_at",
                state.next_review_at.as_deref(),
            )?;
            put(root, topic, "gaps", node::texts(&state.gaps))?;
            render(root, format)
        })
    }

    pub fn reschedule(&mut self, topic: &str, date: &str) -> Result<(), DocumentError> {
        self.edit(|root, format| {
            set(root, topic, "next_review_at", Some(date))?;
            render(root, format)
        })
    }

    pub fn restate(&mut self, topic: &str, status: Status) -> Result<(), DocumentError> {
        self.edit(|root, format| {
            set(root, topic, "status", Some(status.label()))?;
            render(root, format)
        })
    }

    pub fn start(&mut self, topic: &str) -> Result<(), DocumentError> {
        if self.progress.state(topic).is_some() {
            return Ok(());
        }
        self.edit(|root, format| {
            add(root, topic)?;
            render(root, format)
        })
    }

    fn edit(
        &mut self,
        act: impl FnOnce(&mut Yaml<'_>, Format) -> Result<String, DocumentError>,
    ) -> Result<(), DocumentError> {
        let format = self.format;
        let text = with_root(&self.text, |root| act(root, format))?;
        self.progress = parse(&text)?;
        self.text = text;
        Ok(())
    }
}

fn add(root: &mut Yaml<'_>, topic: &str) -> Result<(), DocumentError> {
    let topics = root
        .as_mapping_get_mut("topics")
        .ok_or_else(|| DocumentError::Malformed("it holds no topics".to_owned()))?;
    match topics {
        Yaml::Mapping(map) => {
            map.insert(node::text(topic), node::fresh());
            Ok(())
        }
        other => Err(DocumentError::Malformed(format!(
            "the topics are {other:?}, not a mapping"
        ))),
    }
}

fn set(
    root: &mut Yaml<'_>,
    topic: &str,
    key: &str,
    value: Option<&str>,
) -> Result<(), DocumentError> {
    put(root, topic, key, node::maybe_text(value))
}

fn put(
    root: &mut Yaml<'_>,
    topic: &str,
    key: &str,
    written: Yaml<'static>,
) -> Result<(), DocumentError> {
    let state = root
        .as_mapping_get_mut("topics")
        .and_then(|topics| topics.as_mapping_get_mut(topic))
        .ok_or_else(|| DocumentError::UnknownTopic(topic.to_owned()))?;
    match state.as_mapping_get_mut(key) {
        Some(slot) => *slot = written,
        None => match state {
            Yaml::Mapping(map) => {
                map.insert(node::text(key), written);
            }
            other => {
                return Err(DocumentError::Malformed(format!(
                    "`{topic}` is {other:?}, not a mapping"
                )));
            }
        },
    }
    Ok(())
}

fn append(root: &mut Yaml<'_>, topic: &str, attempt: Yaml<'static>) -> Result<(), DocumentError> {
    let attempts = root
        .as_mapping_get_mut("topics")
        .and_then(|topics| topics.as_mapping_get_mut(topic))
        .ok_or_else(|| DocumentError::UnknownTopic(topic.to_owned()))?
        .as_mapping_get_mut("attempts")
        .ok_or_else(|| DocumentError::Malformed(format!("`{topic}` has no attempts")))?;
    match attempts {
        Yaml::Sequence(items) => {
            items.push(attempt);
            Ok(())
        }
        other => Err(DocumentError::Malformed(format!(
            "the attempts of `{topic}` are {other:?}, not a list"
        ))),
    }
}

fn with_root<T>(
    source: &str,
    act: impl FnOnce(&mut Yaml<'_>) -> Result<T, DocumentError>,
) -> Result<T, DocumentError> {
    let mut documents =
        Yaml::load_from_str(source).map_err(|error| DocumentError::Malformed(error.to_string()))?;
    let root = documents
        .first_mut()
        .ok_or_else(|| DocumentError::Malformed("it holds no document".to_owned()))?;
    act(root)
}
