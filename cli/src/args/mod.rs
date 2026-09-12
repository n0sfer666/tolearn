mod new;
mod next;
mod words;

use std::path::PathBuf;

use crate::error::CliError;
use words::{Words, usage};

pub use new::New;
pub use next::Next;

pub const USAGE: &str = "\
Использование:
  tolearn export   <каталог> <папка> [--json]
  tolearn pack     <каталог> <файл.tolearn> [--json]
  tolearn unpack   <файл.tolearn> <каталог> [--json]
  tolearn new      \"<запрос>\" --level <уровень> --out <каталог> [--provider <файл>] [--json]
  tolearn next     <каталог> [--choice <номер>] [--provider <файл>] [--json]";

#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    pub command: Command,
    pub json: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Export { source: PathBuf, into: PathBuf },
    Pack { source: PathBuf, out: PathBuf },
    Unpack { source: PathBuf, into: PathBuf },
    New(New),
    Next(Next),
}

type Build = fn(&Words<'_>) -> Result<Command, CliError>;

const COMMANDS: [(&str, &[&str], Build); 5] = [
    ("export", &["json"], export),
    ("pack", &["json"], pack),
    ("unpack", &["json"], unpack),
    ("new", &["json", "level", "out", "provider"], new::read),
    ("next", &["json", "choice", "provider"], next::read),
];

pub fn parse(argv: &[String]) -> Result<Args, CliError> {
    let (name, rest) = argv
        .split_first()
        .ok_or_else(|| usage("команда не названа".to_owned()))?;
    let (_, flags, build) = COMMANDS
        .iter()
        .find(|(known, _, _)| known == name)
        .ok_or_else(|| usage(format!("неизвестная команда `{name}`")))?;
    let words = Words::read(name, rest, flags)?;
    Ok(Args {
        command: build(&words)?,
        json: words.json(),
    })
}

fn export(words: &Words<'_>) -> Result<Command, CliError> {
    Ok(Command::Export {
        source: words.path(0, "каталог программы")?,
        into: words.path(1, "папку")?,
    })
}

fn pack(words: &Words<'_>) -> Result<Command, CliError> {
    Ok(Command::Pack {
        source: words.path(0, "каталог программы")?,
        out: words.path(1, "файл пакета")?,
    })
}

fn unpack(words: &Words<'_>) -> Result<Command, CliError> {
    Ok(Command::Unpack {
        source: words.path(0, "пакет")?,
        into: words.path(1, "каталог")?,
    })
}
