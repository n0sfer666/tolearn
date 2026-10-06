use std::process::ExitCode;

use tolearn_offline::commons::{Found, find};
use tolearn_offline::page::Web;

const TIMEOUT_SECS: u64 = 20;

fn main() -> ExitCode {
    let web = match Web::new(TIMEOUT_SECS) {
        Ok(web) => web,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let mut failed = false;
    for query in ["Turing machine", "qzxwvjkplmtr zzqxvw"] {
        match find(&web, query) {
            Ok(Found::Picture(picture)) => println!(
                "«{query}»: {} — {}, {} — .{}, {} КБ",
                picture.title,
                picture.author,
                picture.license,
                picture.extension,
                picture.bytes.len().div_ceil(1024)
            ),
            Ok(Found::Refused(reason)) => println!("«{query}»: не прошла — {reason}"),
            Ok(Found::Missing) => println!("«{query}»: не найдено"),
            Err(error) => {
                failed = true;
                println!("«{query}»: {error}");
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
