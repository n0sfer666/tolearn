mod args;
mod commands;
mod error;
mod out;
mod session;
mod world;

pub use error::CliError;
pub use world::World;

pub fn run(
    argv: &[String],
    world: impl FnOnce() -> Result<World, CliError>,
) -> Result<String, CliError> {
    let args = args::parse(argv)?;
    let output = commands::run(&args, world)?;
    Ok(output.shown(args.json))
}
