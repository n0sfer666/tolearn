#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "package gate: a panic here is the report"
)]

mod support;

use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};
use support::programs::{copy_tree, scratch};
use tolearn_core::package::{MANIFEST, PackError, SCHEMA, pack};
use zip::ZipArchive;

const CHIPTUNE: &str = "examples/chiptune";
const NES_DEV: &str = "fixtures/v2/valid/nes-dev";
const MIDDLE: &str = "children/b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56";
const LEAF: &str = "children/e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";

fn entries(package: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut zip = ZipArchive::new(Cursor::new(package)).unwrap();
    (0..zip.len())
        .map(|index| {
            let mut entry = zip.by_index(index).unwrap();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();
            (entry.name().to_owned(), data)
        })
        .collect()
}

fn names(package: &[u8]) -> Vec<String> {
    entries(package).into_iter().map(|(name, _)| name).collect()
}

fn unpacked(package: &[u8]) -> (serde_json::Value, BTreeMap<String, Vec<u8>>) {
    let mut files: BTreeMap<String, Vec<u8>> = entries(package).into_iter().collect();
    let manifest = serde_json::from_slice(&files.remove(MANIFEST).unwrap()).unwrap();
    (manifest, files)
}

fn sha256(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn stamp(directory: &Path, time: SystemTime) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            stamp(&path, time);
        } else {
            let file = fs::File::options().write(true).open(&path).unwrap();
            file.set_modified(time).unwrap();
        }
    }
}

#[test]
fn two_packs_of_one_program_are_one_archive_whatever_the_file_times() {
    let source = support::root().join(CHIPTUNE);
    let elsewhere = scratch("pack-times").join("chiptune");
    copy_tree(&source, &elsewhere);
    stamp(
        &elsewhere,
        SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000_000),
    );

    let first = pack(&source).unwrap();
    let second = pack(&source).unwrap();
    let moved = pack(&elsewhere).unwrap();

    assert_eq!(sha256(&first), sha256(&second));
    assert_eq!(sha256(&first), sha256(&moved));
}

#[test]
fn entries_go_in_name_order_with_one_fixed_time_and_mode() {
    let package = pack(&support::root().join(NES_DEV)).unwrap();
    let mut zip = ZipArchive::new(Cursor::new(&package)).unwrap();
    let mut names = Vec::new();

    for index in 0..zip.len() {
        let entry = zip.by_index(index).unwrap();
        assert_eq!(
            entry.last_modified(),
            Some(zip::DateTime::default()),
            "{}",
            entry.name()
        );
        assert_eq!(
            entry.unix_mode().map(|mode| mode & 0o777),
            Some(0o644),
            "{}",
            entry.name()
        );
        names.push(entry.name().to_owned());
    }

    assert_eq!(names.pop().as_deref(), Some(MANIFEST));
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn the_manifest_holds_the_schema_and_the_sha256_of_every_other_entry() {
    let source = support::root().join(NES_DEV);

    let (manifest, files) = unpacked(&pack(&source).unwrap());

    assert_eq!(SCHEMA, "tolearn/package/1");
    assert_eq!(manifest["schema"], SCHEMA);
    assert_eq!(manifest.as_object().unwrap().len(), 2);
    let listed: BTreeMap<String, String> = manifest["files"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, sum)| (name.clone(), sum.as_str().unwrap().to_owned()))
        .collect();
    let packed: BTreeMap<String, String> = files
        .iter()
        .map(|(name, data)| (name.clone(), sha256(data)))
        .collect();
    assert_eq!(listed, packed);
    for (name, data) in &files {
        assert_eq!(data, &fs::read(source.join(name)).unwrap(), "{name}");
    }
    assert!(files.contains_key(&format!("{MIDDLE}/{LEAF}/assets/pixel.png")));
}

#[test]
fn only_the_content_of_the_program_is_packed() {
    let source = scratch("pack-content").join("chiptune");
    copy_tree(&support::root().join(CHIPTUNE), &source);
    for (path, text) in [
        ("state/progress.yaml", "stages: {}\n"),
        ("sources/cache/page.html", "<html></html>"),
        ("notes.txt", "черновик"),
        ("stages/orphan.yaml", "schema: tolearn/stage/1\n"),
        ("assets/.DS_Store", "finder"),
    ] {
        let path = source.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    let package = pack(&source).unwrap();

    assert_eq!(
        names(&package),
        [
            "assets/pulse-wave.png",
            "assets/voices.svg",
            "program.yaml",
            "stages/voices.yaml",
            MANIFEST,
        ]
    );
    assert_eq!(package, pack(&support::root().join(CHIPTUNE)).unwrap());
}

#[test]
fn a_program_that_does_not_read_is_not_packed() {
    for (fixture, code) in [
        (
            "fixtures/v2/invalid/program.too-deep__four-levels",
            "library.invalid",
        ),
        (
            "fixtures/v2/invalid/yaml.wrong-type__stage-title-a-list",
            "yaml.wrong-type",
        ),
    ] {
        let error = pack(&support::root().join(fixture)).unwrap_err();
        assert!(matches!(error, PackError::Refused(_)), "{fixture}");
        assert_eq!(error.code(), code, "{fixture}");
    }
}

#[test]
fn a_subprogram_packs_as_a_program_of_its_own() {
    let nested = support::root().join(NES_DEV).join(MIDDLE);
    let alone = scratch("pack-child").join("middle");
    copy_tree(&nested, &alone);

    let package = pack(&nested).unwrap();

    let (_, files) = unpacked(&package);
    assert_eq!(
        files["program.yaml"],
        fs::read(nested.join("program.yaml")).unwrap()
    );
    assert!(files.contains_key(&format!("{LEAF}/assets/pixel.png")));
    assert!(files.keys().all(|name| !name.starts_with(MIDDLE)));
    assert_eq!(package, pack(&alone).unwrap());
}

#[cfg(unix)]
#[test]
fn an_odd_asset_name_is_escaped_in_the_manifest() {
    let source = scratch("pack-odd-name").join("chiptune");
    copy_tree(&support::root().join(CHIPTUNE), &source);
    let odd = "assets/звук \"8\\bit\"\t.png";
    fs::write(source.join(odd), "png").unwrap();

    let (manifest, files) = unpacked(&pack(&source).unwrap());

    assert_eq!(manifest["files"][odd], sha256(b"png"));
    assert!(files.contains_key(odd));
}

#[cfg(unix)]
#[test]
fn a_file_that_will_not_read_stops_the_pack_and_is_named() {
    let source = scratch("pack-unreadable").join("chiptune");
    copy_tree(&support::root().join(CHIPTUNE), &source);
    let picture = source.join("assets/pulse-wave.png");

    support::programs::chmod(&picture, 0o000);
    let packed = pack(&source);
    support::programs::chmod(&picture, 0o644);

    let error = packed.unwrap_err();
    assert_eq!(error.code(), "package.unreadable");
    assert_eq!(
        error,
        PackError::Unreadable {
            path: picture,
            kind: std::io::ErrorKind::PermissionDenied,
        }
    );
}
