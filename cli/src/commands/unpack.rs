use std::path::Path;

use serde_json::json;
use tolearn_core::package::unpack;

use crate::error::CliError;
use crate::out::Output;

pub fn run(file: &Path, into: &Path) -> Result<Output, CliError> {
    let tree = unpack(file, into)
        .map_err(|error| CliError::Import(format!("{} — {error}", error.code())))?;
    Ok(Output::new(
        format!("распаковано в {}: «{}»", into.display(), tree.program.title),
        json!({ "path": into.display().to_string(), "uuid": tree.program.uuid }),
    ))
}
