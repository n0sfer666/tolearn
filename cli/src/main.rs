use std::process::ExitCode;

use tolearn_cli::{World, run};

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match run(&argv, World::real) {
        Ok(shown) => {
            println!("{shown}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(u8::try_from(error.code()).unwrap_or(2))
        }
    }
}
