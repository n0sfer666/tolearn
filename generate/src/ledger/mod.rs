mod kind;
mod record;
mod tally;
mod total;

pub use kind::Kind;
pub use record::Record;
pub use tally::Tally;
pub use total::Total;

use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::error::GenerateError;
use crate::sources::CACHE;

pub const LEDGER: &str = "ledger.jsonl";

pub fn path(data: &Path, program: &str) -> PathBuf {
    data.join(CACHE).join(program).join(LEDGER)
}

pub fn append(path: &Path, records: &[Record]) -> Result<(), GenerateError> {
    let unwritten =
        |error: &dyn Display| GenerateError::Unwritten(format!("{}: {error}", path.display()));
    let mut lines = String::new();
    for record in records {
        lines.push_str(&serde_json::to_string(record).map_err(|error| unwritten(&error))?);
        lines.push('\n');
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| unwritten(&error))?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(lines.as_bytes()))
        .map_err(|error| unwritten(&error))
}

pub fn read(path: &Path) -> Result<Vec<Record>, GenerateError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(GenerateError::Cache(format!("{}: {error}", path.display()))),
    };
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(number, line)| {
            serde_json::from_str(line).map_err(|error| {
                GenerateError::Cache(format!("{}:{}: {error}", path.display(), number + 1))
            })
        })
        .collect()
}

pub(crate) fn since(began: Instant) -> u64 {
    u64::try_from(began.elapsed().as_millis()).unwrap_or(u64::MAX)
}
