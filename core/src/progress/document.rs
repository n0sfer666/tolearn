use saphyr::{LoadableYamlNode, Yaml};

use super::error::DocumentError;
use super::node;
use super::parse::parse;
use super::render::{Format, render};
use super::types::{Attempt, Progress};

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
