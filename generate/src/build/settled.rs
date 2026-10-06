use std::fs;
use std::path::Path;

use crate::error::GenerateError;
use crate::ledger;

use super::kit::Kit;

pub(crate) fn settled<T>(
    kit: &Kit<'_>,
    program: &str,
    node: &str,
    folder: &Path,
    landed: &Result<T, GenerateError>,
) {
    let _ = fs::remove_dir_all(folder);
    let tally = kit.online.tally();
    if landed.is_err() {
        tally.extend(tally.release());
    }
    tally.stamp(0, node, None);
    tally.date(kit.at);
    let _ = ledger::append(&ledger::path(kit.data, program), &tally.take());
}
