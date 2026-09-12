use std::path::PathBuf;

use super::Command;
use super::words::{Words, usage};
use crate::error::CliError;

#[derive(Debug, PartialEq, Eq)]
pub struct Next {
    pub data: PathBuf,
    pub choice: Option<usize>,
    pub provider: Option<PathBuf>,
}

pub fn read(words: &Words<'_>) -> Result<Command, CliError> {
    let choice = words
        .value("choice")
        .map(|raw| {
            raw.parse::<usize>()
                .ok()
                .filter(|choice| *choice >= 1)
                .ok_or_else(|| usage("`--choice` ждёт номер варианта: 1, 2, …".to_owned()))
        })
        .transpose()?;
    Ok(Command::Next(Next {
        data: PathBuf::from(words.only("каталог")?),
        choice,
        provider: words.value("provider").map(PathBuf::from),
    }))
}
