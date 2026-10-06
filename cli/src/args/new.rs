use std::path::PathBuf;

use super::Command;
use super::words::Words;
use crate::error::CliError;

#[derive(Debug, PartialEq, Eq)]
pub struct New {
    pub request: String,
    pub level: String,
    pub out: PathBuf,
    pub provider: Option<PathBuf>,
}

pub fn read(words: &Words<'_>) -> Result<Command, CliError> {
    Ok(Command::New(New {
        request: words.only("запрос")?.trim().to_owned(),
        level: words.needed("level", "уровень")?.trim().to_owned(),
        out: PathBuf::from(words.needed("out", "каталог")?),
        provider: words.value("provider").map(PathBuf::from),
    }))
}
