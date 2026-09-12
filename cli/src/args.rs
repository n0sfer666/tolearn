use std::path::PathBuf;

use crate::error::CliError;

pub const USAGE: &str = "\
Использование:
  tolearn export   <каталог> <папка> [--json]
  tolearn pack     <каталог> <файл.tolearn> [--json]
  tolearn unpack   <файл.tolearn> <каталог> [--json]";

#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    pub command: Command,
    pub source: PathBuf,
    pub json: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Export { into: PathBuf },
    Pack { out: PathBuf },
    Unpack { into: PathBuf },
}

type Build = fn(PathBuf) -> Command;

pub fn parse(argv: &[String]) -> Result<Args, CliError> {
    let mut rest = argv.iter().map(String::as_str);
    let name = rest
        .next()
        .ok_or_else(|| CliError::Usage("команда не названа".to_owned()))?;
    let (source, target, build): (&str, &str, Build) = match name {
        "export" => ("каталог программы", "папку", |into| {
            Command::Export { into }
        }),
        "pack" => (
            "каталог программы",
            "файл пакета",
            |out| Command::Pack { out },
        ),
        "unpack" => ("пакет", "каталог", |into| Command::Unpack {
            into,
        }),
        other => return Err(CliError::Usage(format!("неизвестная команда `{other}`"))),
    };

    let mut free: Vec<&str> = Vec::new();
    let mut json = false;
    for word in rest {
        if let Some(flag) = word.strip_prefix("--") {
            if flag != "json" {
                return Err(CliError::Usage(format!("неизвестный флаг `--{flag}`")));
            }
            json = true;
            continue;
        }
        free.push(word);
    }

    let given = |index: usize, what: &str| {
        free.get(index)
            .map(PathBuf::from)
            .ok_or_else(|| CliError::Usage(format!("`{name}` не назвал {what}")))
    };

    Ok(Args {
        source: given(0, source)?,
        command: build(given(1, target)?),
        json,
    })
}
