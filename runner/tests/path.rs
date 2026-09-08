#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "runner gate: a panic here is the report"
)]

use std::time::Duration;

use tolearn_runner::{Limits, Outcome, search, spawn};

fn scratch(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("tolearn-path-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[cfg(unix)]
#[test]
fn the_program_inherits_the_search_it_was_found_by() {
    let directory = scratch("inherited");

    let run = spawn(
        "sh",
        &["-c".to_owned(), "printf %s \"$PATH\"".to_owned()],
        &directory,
        "",
        Limits {
            timeout: Duration::from_secs(10),
            silence: None,
            output_bytes: 64 * 1024,
        },
        None,
    )
    .unwrap();

    let _ = std::fs::remove_dir_all(&directory);
    assert!(
        matches!(run.outcome, Outcome::Finished { code: Some(0) }),
        "{run:?}"
    );
    assert_eq!(run.stdout, search());
}

#[test]
fn the_enriched_search_never_loses_what_it_inherited() {
    let inherited = std::env::var("PATH").unwrap_or_default();
    let found = search();

    for place in inherited
        .split(if cfg!(windows) { ';' } else { ':' })
        .filter(|part| !part.is_empty())
    {
        assert!(found.contains(place), "{place} ушло из {found}");
    }
}
