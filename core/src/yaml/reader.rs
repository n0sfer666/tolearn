use std::fmt;

use saphyr::MarkedYaml;

use super::dates;
use super::error::ParseError;
use super::failure::ParseFailure;

#[derive(Debug, Clone)]
pub struct Reader<'a> {
    node: &'a MarkedYaml<'a>,
    path: String,
}

impl<'a> Reader<'a> {
    pub(crate) fn root(node: &'a MarkedYaml<'a>) -> Self {
        Self {
            node,
            path: String::new(),
        }
    }

    pub fn field(&self, name: &str) -> Result<Self, ParseError> {
        if !self.node.data.is_mapping() {
            return Err(self.fail(ParseFailure::WrongType, "expected a mapping"));
        }
        let value = self.node.data.as_mapping_get(name).ok_or_else(|| {
            self.fail(
                ParseFailure::MissingField,
                format!("field `{name}` is missing"),
            )
        })?;
        Ok(Self {
            node: value,
            path: nested(&self.path, name),
        })
    }

    pub fn optional_field(&self, name: &str) -> Result<Option<Self>, ParseError> {
        if !self.node.data.is_mapping() {
            return Err(self.fail(ParseFailure::WrongType, "expected a mapping"));
        }
        Ok(self.node.data.as_mapping_get(name).map(|value| Self {
            node: value,
            path: nested(&self.path, name),
        }))
    }

    pub fn text(&self) -> Result<String, ParseError> {
        let text = self.any_text()?;
        if text.trim().is_empty() {
            return Err(self.fail(
                ParseFailure::Empty,
                "expected a string with something in it",
            ));
        }
        Ok(text)
    }

    pub fn any_text(&self) -> Result<String, ParseError> {
        self.node
            .data
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| self.fail(ParseFailure::WrongType, "expected a string"))
    }

    pub fn date(&self) -> Result<String, ParseError> {
        let text = self.text()?;
        if !dates::is_date(&text) {
            return Err(self.fail(
                ParseFailure::BadDate,
                format!("`{text}` is no date of the form YYYY-MM-DD"),
            ));
        }
        Ok(text)
    }

    pub fn optional_date(&self) -> Result<Option<String>, ParseError> {
        if self.node.data.is_null() {
            return Ok(None);
        }
        self.date().map(Some)
    }

    pub fn moment(&self) -> Result<String, ParseError> {
        let text = self.text()?;
        if !dates::is_moment(&text) {
            return Err(self.fail(
                ParseFailure::BadDate,
                format!("`{text}` is no moment in time by RFC 3339"),
            ));
        }
        Ok(text)
    }

    pub fn optional_text(&self) -> Result<Option<String>, ParseError> {
        if self.node.data.is_null() {
            return Ok(None);
        }
        self.text().map(Some)
    }

    pub fn choice<T: Copy>(&self, what: &str, options: &[(&str, T)]) -> Result<T, ParseError> {
        let text = self.text()?;
        options
            .iter()
            .find(|(name, _)| *name == text)
            .map(|(_, value)| *value)
            .ok_or_else(|| self.unknown(format!("unknown {what} `{text}`")))
    }

    pub fn number(&self, minimum: u32) -> Result<u32, ParseError> {
        let integer = self
            .node
            .data
            .as_integer()
            .ok_or_else(|| self.fail(ParseFailure::WrongType, "expected an integer"))?;
        let value = u32::try_from(integer).map_err(|_| {
            self.fail(
                ParseFailure::OutOfRange,
                format!("the integer {integer} is outside the range of the field"),
            )
        })?;
        if value < minimum {
            return Err(self.fail(
                ParseFailure::OutOfRange,
                format!("expected an integer not below {minimum}"),
            ));
        }
        Ok(value)
    }

    pub fn flag(&self) -> Result<bool, ParseError> {
        self.node
            .data
            .as_bool()
            .ok_or_else(|| self.fail(ParseFailure::WrongType, "expected true or false"))
    }

    pub fn items(&self) -> Result<Vec<Self>, ParseError> {
        let sequence = self
            .node
            .data
            .as_sequence()
            .ok_or_else(|| self.fail(ParseFailure::WrongType, "expected a list"))?;
        Ok(sequence
            .iter()
            .enumerate()
            .map(|(index, item)| Self {
                node: item,
                path: format!("{}[{index}]", self.path),
            })
            .collect())
    }

    pub fn list<T>(
        &self,
        each: impl Fn(&Self) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        self.items()?.iter().map(each).collect()
    }

    pub fn texts(&self) -> Result<Vec<String>, ParseError> {
        self.list(Self::text)
    }

    pub fn entries(&self) -> Result<Vec<(String, Self)>, ParseError> {
        let mapping = self
            .node
            .data
            .as_mapping()
            .ok_or_else(|| self.fail(ParseFailure::WrongType, "expected a mapping"))?;
        mapping
            .iter()
            .map(|(key, value)| {
                let name = key.data.as_str().ok_or_else(|| {
                    ParseError::at(
                        ParseFailure::WrongType,
                        key.span.start,
                        &self.path,
                        "expected a string as a key",
                    )
                })?;
                Ok((
                    name.to_owned(),
                    Self {
                        node: value,
                        path: nested(&self.path, name),
                    },
                ))
            })
            .collect()
    }

    pub fn malformed(&self, message: impl fmt::Display) -> ParseError {
        self.fail(ParseFailure::WrongType, message.to_string())
    }

    pub fn unknown(&self, message: impl fmt::Display) -> ParseError {
        self.fail(ParseFailure::UnknownValue, message.to_string())
    }

    fn fail(&self, failure: ParseFailure, message: impl Into<String>) -> ParseError {
        ParseError::at(failure, self.node.span.start, &self.path, message)
    }
}

fn nested(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}
