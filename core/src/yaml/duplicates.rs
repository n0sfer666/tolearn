use std::collections::HashMap;

use saphyr_parser::{Event, Parser};

use super::error::ParseError;
use super::failure::ParseFailure;

pub(crate) fn check(source: &str) -> Result<(), ParseError> {
    let mut frames: Vec<Frame> = Vec::new();
    for event in Parser::new_from_str(source) {
        let (event, span) = event.map_err(ParseError::from_scan)?;
        match event {
            Event::Scalar(value, ..) => {
                if let Some(frame) = frames.last_mut()
                    && frame.take_key(&value)
                    && let Some(first) = frame.remember(&value, span.start.line())
                {
                    return Err(twice(&value, first, span.start.line(), &frame.path));
                }
                step(&mut frames);
            }
            Event::Alias(_) => {
                if let Some(frame) = frames.last_mut() {
                    frame.take_key("");
                }
                step(&mut frames);
            }
            Event::MappingStart(..) => frames.push(Frame::mapping(descend(&frames))),
            Event::SequenceStart(..) => frames.push(Frame::sequence(descend(&frames))),
            Event::MappingEnd | Event::SequenceEnd => {
                frames.pop();
                step(&mut frames);
            }
            _ => {}
        }
    }
    Ok(())
}

struct Frame {
    keys: Option<HashMap<String, usize>>,
    key: Option<String>,
    expects_key: bool,
    index: usize,
    path: String,
}

impl Frame {
    fn mapping(path: String) -> Self {
        Self {
            keys: Some(HashMap::new()),
            key: None,
            expects_key: true,
            index: 0,
            path,
        }
    }

    fn sequence(path: String) -> Self {
        Self {
            keys: None,
            key: None,
            expects_key: false,
            index: 0,
            path,
        }
    }

    fn take_key(&mut self, value: &str) -> bool {
        if self.keys.is_none() || !self.expects_key {
            return false;
        }
        self.key = Some(value.to_owned());
        true
    }

    fn remember(&mut self, key: &str, line: usize) -> Option<usize> {
        self.keys
            .as_mut()
            .and_then(|keys| keys.insert(key.to_owned(), line))
    }
}

fn step(frames: &mut [Frame]) {
    if let Some(frame) = frames.last_mut() {
        if frame.keys.is_some() {
            frame.expects_key = !frame.expects_key;
        } else {
            frame.index += 1;
        }
    }
}

fn descend(frames: &[Frame]) -> String {
    let Some(frame) = frames.last() else {
        return String::new();
    };
    match (&frame.keys, &frame.key) {
        (Some(_), Some(key)) if frame.path.is_empty() => key.clone(),
        (Some(_), Some(key)) => format!("{}.{key}", frame.path),
        (Some(_), None) => frame.path.clone(),
        (None, _) => format!("{}[{}]", frame.path, frame.index),
    }
}

fn twice(key: &str, first: usize, again: usize, path: &str) -> ParseError {
    ParseError::at_line(
        ParseFailure::DuplicateKey,
        again,
        path,
        format!("the key `{key}` is written twice, first on line {first}"),
    )
}
