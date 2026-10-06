use std::process::ExitCode;

use tolearn_offline::book::{Book, Wanted, find};
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
    let asks = [
        wanted(Some("978-0-262-51087-5"), "", ""),
        wanted(
            None,
            "Structure and Interpretation of Computer Programs",
            "Abelson",
        ),
        wanted(None, "qzxwvjkplmtr", "zzqxvw"),
    ];
    let mut failed = false;
    for ask in &asks {
        let label = match &ask.isbn {
            Some(isbn) => format!("ISBN {isbn}"),
            None => format!("«{}» / {}", ask.title, ask.author),
        };
        match find(&web, ask) {
            Ok(Some(book)) => println!("{label}: {}", described(&book)),
            Ok(None) => println!("{label}: не найдено"),
            Err(error) => {
                failed = true;
                println!("{label}: {error}");
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn wanted(isbn: Option<&str>, title: &str, author: &str) -> Wanted {
    Wanted {
        isbn: isbn.map(str::to_owned),
        title: title.to_owned(),
        author: author.to_owned(),
    }
}

fn described(book: &Book) -> String {
    format!(
        "{} — {} — ISBN {}",
        book.title,
        book.authors.join(", "),
        book.isbn.as_deref().unwrap_or("нет")
    )
}
