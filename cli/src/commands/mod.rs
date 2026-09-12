mod export;
mod pack;
mod unpack;

use crate::args::{Args, Command};
use crate::error::CliError;
use crate::out::Output;

pub fn run(args: &Args) -> Result<Output, CliError> {
    let root = args.source.as_path();
    match &args.command {
        Command::Export { into } => export::run(root, into),
        Command::Pack { out } => pack::run(root, out),
        Command::Unpack { into } => unpack::run(root, into),
    }
}
