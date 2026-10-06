use std::fmt::Display;
use std::path::Path;

use tolearn_core::library::{Entry, Library, PROGRAMS};
use tolearn_core::program::Tree;

use crate::error::CliError;

pub fn empty(out: &Path) -> Result<(), CliError> {
    if listed(out)?.is_empty() {
        return Ok(());
    }
    Err(CliError::Data(format!(
        "в `{0}` уже лежит программа: продолжай её через `tolearn next {0}` или назови пустой каталог",
        out.display()
    )))
}

pub fn only(data: &Path) -> Result<Tree, CliError> {
    let mut entries = listed(data)?;
    match (entries.pop(), entries.len()) {
        (None, _) => Err(CliError::Data(format!(
            "в `{}` нет программы: начни с `tolearn new`",
            data.display()
        ))),
        (Some(entry), 0) => entry.program.map_err(|refusal| {
            CliError::Data(format!(
                "программа `{}` в `{}` не читается: {refusal}",
                entry.directory,
                data.display()
            ))
        }),
        (Some(_), more) => Err(CliError::Data(format!(
            "в `{}` программ несколько ({}): `tolearn next` ведёт одну",
            data.display(),
            more.saturating_add(1)
        ))),
    }
}

pub fn opened(data: &Path, uuid: &str) -> Result<Tree, CliError> {
    Library::at(data)
        .open(uuid)
        .map_err(|error| unsettled(data, uuid, error))
}

pub fn unsettled(data: &Path, uuid: &str, why: impl Display) -> CliError {
    CliError::Data(format!(
        "программа записана в `{}`, но итог не собран: {why}",
        data.join(PROGRAMS).join(uuid).display()
    ))
}

fn listed(data: &Path) -> Result<Vec<Entry>, CliError> {
    Library::at(data)
        .list()
        .map_err(|error| CliError::Data(format!("`{}`: {error}", data.display())))
}
