mod checks;
mod exam;
mod merge;
mod progress;
mod scan;
mod validate;

use crate::args::{Args, Command};
use crate::error::CliError;
use crate::out::Output;

pub fn run(args: &Args) -> Result<Output, CliError> {
    let root = args.bundle.as_path();
    match &args.command {
        Command::Validate => validate::run(root),
        Command::Scan => scan::run(root),
        Command::Progress { today } => progress::run(root, today.as_deref()),
        Command::Exam { .. } => exam::run(root, &args.command),
        Command::Merge { was } => merge::run(root, was),
    }
}
