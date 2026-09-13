use std::path::Path;

use tolearn_generate::ledger;

use crate::error::CliError;
use crate::out::Output;
use crate::session::{Summary, only};

pub fn run(data: &Path) -> Result<Output, CliError> {
    let tree = only(data)?;
    let path = ledger::path(data, &tree.program.uuid);
    let records = ledger::read(&path)?;
    Ok(Summary::of(&records).shown(&path, &tree.program.uuid))
}
