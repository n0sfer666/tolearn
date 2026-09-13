mod export;
mod ledger;
mod new;
mod next;
mod offered;
mod pack;
mod unpack;

use crate::args::{Args, Command};
use crate::error::CliError;
use crate::out::Output;
use crate::world::World;

pub fn run(
    args: &Args,
    world: impl FnOnce() -> Result<World, CliError>,
) -> Result<Output, CliError> {
    match &args.command {
        Command::Export { source, into } => export::run(source, into),
        Command::Pack { source, out } => pack::run(source, out),
        Command::Unpack { source, into } => unpack::run(source, into),
        Command::New(new) => new::run(new, world()?),
        Command::Next(next) => next::run(next, world()?),
        Command::Ledger { source } => ledger::run(source),
    }
}
