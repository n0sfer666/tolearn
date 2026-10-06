use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::CliError;

const JSON: &str = "json";

#[derive(Debug)]
pub struct Words<'a> {
    name: &'a str,
    free: Vec<&'a str>,
    given: BTreeMap<&'a str, &'a str>,
    json: bool,
}

impl<'a> Words<'a> {
    pub fn read(name: &'a str, rest: &'a [String], flags: &[&str]) -> Result<Self, CliError> {
        let mut words = Self {
            name,
            free: Vec::new(),
            given: BTreeMap::new(),
            json: false,
        };
        let mut rest = rest.iter().map(String::as_str);
        while let Some(word) = rest.next() {
            let Some(flag) = word.strip_prefix("--") else {
                words.free.push(word);
                continue;
            };
            if !flags.contains(&flag) {
                return Err(usage(format!("неизвестный флаг `--{flag}`")));
            }
            if flag == JSON {
                words.json = true;
                continue;
            }
            let value = rest
                .next()
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| usage(format!("флаг `--{flag}` без значения")))?;
            if words.given.insert(flag, value).is_some() {
                return Err(usage(format!("флаг `--{flag}` назван дважды")));
            }
        }
        Ok(words)
    }

    pub fn json(&self) -> bool {
        self.json
    }

    pub fn path(&self, index: usize, what: &str) -> Result<PathBuf, CliError> {
        self.free
            .get(index)
            .map(PathBuf::from)
            .ok_or_else(|| self.unnamed(what))
    }

    pub fn only(&self, what: &str) -> Result<&'a str, CliError> {
        match self.free.as_slice() {
            [word] if !word.trim().is_empty() => Ok(word),
            [] | [_] => Err(self.unnamed(what)),
            _ => Err(usage(format!(
                "`{}` ждёт один {what}: слова с пробелами пиши в кавычках",
                self.name
            ))),
        }
    }

    pub fn needed(&self, flag: &str, what: &str) -> Result<&'a str, CliError> {
        self.value(flag)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| self.unnamed(&format!("{what}: `--{flag}`")))
    }

    pub fn value(&self, flag: &str) -> Option<&'a str> {
        self.given.get(flag).copied()
    }

    fn unnamed(&self, what: &str) -> CliError {
        usage(format!("`{}` не назвал {what}", self.name))
    }
}

pub fn usage(text: String) -> CliError {
    CliError::Usage(text)
}
