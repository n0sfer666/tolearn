mod args;
mod bundle;
mod commands;
mod error;
mod out;
mod today;

use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let code = match args::parse(&argv).and_then(|args| {
        let output = commands::run(&args)?;
        println!("{}", output.shown(args.json));
        Ok(output.code())
    }) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            error.code()
        }
    };
    ExitCode::from(u8::try_from(code).unwrap_or(2))
}
