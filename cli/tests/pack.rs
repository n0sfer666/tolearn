#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

#[allow(
    dead_code,
    reason = "the pack gate takes only its part of the shared support module"
)]
mod support;

use std::path::PathBuf;

use support::{code, copy, json, scratch, stderr, tolearn};
use tolearn_core::package::pack;

fn repo(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(relative)
}

#[test]
fn pack_writes_the_package_that_core_builds() {
    let program = repo("examples/chiptune");
    let file = scratch("pack").join("out/chiptune.tolearn");

    let out = tolearn(&[
        "pack",
        program.to_str().unwrap(),
        file.to_str().unwrap(),
        "--json",
    ]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let written = std::fs::read(&file).unwrap();
    assert_eq!(written, pack(&program).unwrap());
    let report = json(&out);
    assert_eq!(report["path"], file.display().to_string());
    assert_eq!(report["bytes"], written.len());
}

#[test]
fn a_program_that_does_not_read_is_not_packed_and_no_file_appears() {
    let program = repo("fixtures/v2/invalid/program.too-deep__four-levels");
    let file = scratch("pack-refused").join("broken.tolearn");

    let out = tolearn(&["pack", program.to_str().unwrap(), file.to_str().unwrap()]);

    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("program.too-deep"),
        "{}",
        stderr(&out)
    );
    assert!(!file.exists());
}

#[test]
fn pack_without_the_package_file_is_a_usage_error() {
    let out = tolearn(&["pack", repo("examples/chiptune").to_str().unwrap()]);

    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("`pack`"), "{}", stderr(&out));
}

#[test]
fn a_package_inside_the_program_is_refused() {
    let program = scratch("pack-inside").join("chiptune");
    copy(&repo("examples/chiptune"), &program);
    let file = program.join("chiptune.tolearn");

    let out = tolearn(&["pack", program.to_str().unwrap(), file.to_str().unwrap()]);

    assert_eq!(code(&out), 2);
    assert!(
        stderr(&out).contains("каталог программы"),
        "{}",
        stderr(&out)
    );
    assert!(!file.exists());
}
