use std::fmt::Display;
use std::path::Path;

use serde_json::json;
use tolearn_core::export::{claimed, render, within, write};
use tolearn_core::program::load;

use crate::error::CliError;
use crate::out::Output;

pub fn run(program: &Path, into: &Path) -> Result<Output, CliError> {
    if claimed(into) {
        return Err(CliError::Usage(format!(
            "`{}` лежит в каталоге программы: туда пишут только генерация и импорт",
            into.display()
        )));
    }
    let tree = load(program).map_err(|error| refused(error.code(), error))?;
    let files = write(&render(&tree), into, within(program))
        .map_err(|error| refused(error.code(), error))?;
    Ok(Output::new(
        format!("выгружено в {} (файлов: {files})", into.display()),
        json!({ "path": into.display().to_string(), "files": files }),
    ))
}

fn refused(code: &str, error: impl Display) -> CliError {
    CliError::Export(format!("{code} — {error}"))
}
