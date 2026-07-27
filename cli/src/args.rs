use std::path::PathBuf;

use crate::error::CliError;

pub const USAGE: &str = "\
Использование:
  tolearn validate <бандл> [--json]
  tolearn scan     <бандл> [--json]
  tolearn progress <бандл> [--json] [--today ГГГГ-ММ-ДД]
  tolearn exam     <бандл> <тема> [--verdict <файл>] [--template <файл>] [--run-checks] [--json]
  tolearn merge    <бандл> --was <каталог> [--json]

Check-команды бандла не выполняются никогда, кроме явного `--run-checks`.";

#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    pub command: Command,
    pub bundle: PathBuf,
    pub json: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Validate,
    Scan,
    Progress {
        today: Option<String>,
    },
    Exam {
        topic: String,
        verdict: Option<PathBuf>,
        template: Option<PathBuf>,
        run_checks: bool,
    },
    Merge {
        was: PathBuf,
    },
}

pub fn parse(argv: &[String]) -> Result<Args, CliError> {
    let mut rest = argv.iter().map(String::as_str);
    let name = rest
        .next()
        .ok_or_else(|| CliError::Usage("команда не названа".to_owned()))?;
    let mut free: Vec<&str> = Vec::new();
    let mut flags: Vec<(&str, Option<String>)> = Vec::new();
    let mut rest = rest.peekable();
    while let Some(word) = rest.next() {
        if let Some(flag) = word.strip_prefix("--") {
            let value = match flag {
                "json" | "run-checks" => None,
                "verdict" | "template" | "was" | "today" => Some(taken(flag, rest.next())?),
                _ => return Err(CliError::Usage(format!("неизвестный флаг `--{flag}`"))),
            };
            flags.push((flag, value));
            continue;
        }
        free.push(word);
    }

    let json = flags.iter().any(|(flag, _)| *flag == "json");
    let bundle =
        PathBuf::from(free.first().copied().ok_or_else(|| {
            CliError::Usage(format!("команде `{name}` не назван каталог бандла"))
        })?);
    let value = |wanted: &str| {
        flags
            .iter()
            .find(|(flag, _)| *flag == wanted)
            .and_then(|(_, value)| value.clone())
    };

    let command = match name {
        "validate" => Command::Validate,
        "scan" => Command::Scan,
        "progress" => Command::Progress {
            today: value("today"),
        },
        "exam" => Command::Exam {
            topic: free
                .get(1)
                .map(|topic| (*topic).to_owned())
                .ok_or_else(|| CliError::Usage("`exam` не назвал тему".to_owned()))?,
            verdict: value("verdict").map(PathBuf::from),
            template: value("template").map(PathBuf::from),
            run_checks: flags.iter().any(|(flag, _)| *flag == "run-checks"),
        },
        "merge" => Command::Merge {
            was: value("was")
                .map(PathBuf::from)
                .ok_or_else(|| CliError::Usage("`merge` не назвал `--was`".to_owned()))?,
        },
        other => return Err(CliError::Usage(format!("неизвестная команда `{other}`"))),
    };

    Ok(Args {
        command,
        bundle,
        json,
    })
}

fn taken(flag: &str, value: Option<&str>) -> Result<String, CliError> {
    value
        .map(ToOwned::to_owned)
        .ok_or_else(|| CliError::Usage(format!("у `--{flag}` нет значения")))
}
