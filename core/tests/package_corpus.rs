#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "package corpus gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use support::packages::{CHIPTUNE, DIAGRAM, NES_DEV, corpus};
use support::programs::scratch;
use support::sealing::{content, sealed, with};
use tolearn_core::library::Library;
use tolearn_core::package::{import, unpack};

const REFUSALS: [&str; 13] = [
    "archive.unknown-format",
    "archive.escaping-path",
    "archive.link",
    "package.no-manifest",
    "package.bad-manifest",
    "package.version",
    "package.unlisted",
    "package.missing",
    "package.checksum",
    "package.unused",
    "package.unsafe-svg",
    "library.invalid",
    "yaml.wrong-type",
];

const LEAF_SVG: &str = "children/b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56/children/e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47/assets/pipeline.svg";

const SAFE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<!-- схема -->
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 10 10">
  <defs><circle id="dot" r="1"/></defs>
  <use href="#dot"/>
  <use xlink:href=" #dot"/>
  <text font-family="sans-serif" opacity="0.5">a &amp; b &#x2014; c</text>
</svg>
"##;

fn hidden(directory: &Path) -> Vec<String> {
    fs::read_dir(directory).map_or_else(
        |_| Vec::new(),
        |listing| {
            listing
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .filter(|name| name.starts_with('.'))
                .collect()
        },
    )
}

#[test]
fn every_broken_package_is_refused_with_its_code_and_names_its_file() {
    let room = scratch("package-corpus");
    for broken in corpus() {
        let file = room.join(format!("{}.tolearn", broken.name));
        fs::write(&file, &broken.bytes).unwrap();
        let data = room.join(broken.name);
        fs::create_dir_all(&data).unwrap();
        let programs = data.join("programs");

        let imported = import(&file, &Library::at(&data)).unwrap_err();
        let into = data.join("unpacked");
        let unpacked = unpack(&file, &into).unwrap_err();

        assert_eq!(imported.code(), broken.code, "{}: {imported}", broken.name);
        assert!(
            imported.to_string().contains(broken.names),
            "{} should name `{}`: {imported}",
            broken.name,
            broken.names
        );
        assert!(
            !imported.to_string().contains(".unpack"),
            "{}: {imported}",
            broken.name
        );
        assert_eq!(unpacked, imported, "{}", broken.name);
        assert!(!programs.exists(), "{}", broken.name);
        assert!(!into.exists(), "{}", broken.name);
        assert!(hidden(&data).is_empty(), "{}", broken.name);
    }
}

#[test]
fn the_corpus_holds_a_broken_package_for_every_refusal() {
    let codes: BTreeSet<&str> = corpus().iter().map(|broken| broken.code).collect();
    assert_eq!(codes, REFUSALS.into_iter().collect());
    let names: BTreeSet<&str> = corpus().iter().map(|broken| broken.name).collect();
    assert_eq!(names.len(), corpus().len());
}

#[test]
fn an_svg_deep_in_a_subprogram_is_checked_and_named_where_it_lies() {
    let room = scratch("package-leaf-svg");
    let file = room.join("nes.tolearn");
    let script = r#"<svg xmlns="http://www.w3.org/2000/svg"><script/></svg>"#;
    fs::write(
        &file,
        sealed(with(content(NES_DEV), LEAF_SVG, script.as_bytes())),
    )
    .unwrap();

    let error = import(&file, &Library::at(&room.join("data"))).unwrap_err();

    assert_eq!(error.code(), "package.unsafe-svg");
    assert!(error.to_string().contains(LEAF_SVG), "{error}");
}

#[test]
fn an_svg_that_points_only_inside_itself_is_let_in() {
    let room = scratch("package-safe-svg");
    let file = room.join("chiptune.tolearn");
    fs::write(
        &file,
        sealed(with(content(CHIPTUNE), DIAGRAM, SAFE.as_bytes())),
    )
    .unwrap();

    let imported = import(&file, &Library::at(&room.join("data"))).unwrap();

    assert_eq!(imported.copy_of, None);
    assert_eq!(
        fs::read(
            room.join("data/programs")
                .join(&imported.uuid)
                .join(DIAGRAM)
        )
        .unwrap(),
        SAFE.as_bytes()
    );
}
