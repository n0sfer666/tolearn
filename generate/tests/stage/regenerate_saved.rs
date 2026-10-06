use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use tolearn_core::library::Library;
use tolearn_generate::Step;

use crate::answers::fine;
use crate::forking::{after, begun, regenerated, said, steps};
use crate::starting::{Recorder, flat, names};
use crate::web::Bench;

const GATHERED: [Step; 4] = [Step::Sources, Step::Text, Step::Diagrams, Step::Write];
const KEPT: [Step; 3] = [Step::Text, Step::Diagrams, Step::Write];

fn cached(bench: &Bench, uuid: &str, kind: &str) -> PathBuf {
    bench.dir.join("cache").join(uuid).join(kind)
}

fn set(bench: &Bench, uuid: &str) -> PathBuf {
    let folder = cached(bench, uuid, "gathered");
    let listed = names(&folder);
    assert_eq!(listed.len(), 1, "{listed:?}");
    folder.join(&listed[0])
}

fn regathered(bench: &mut Bench, uuid: &str) -> Vec<Step> {
    let recorder = Recorder::default();
    regenerated(bench, &after(uuid, "tracker"), &flat(), &recorder).unwrap();
    steps(&recorder)
}

#[test]
fn without_the_saved_set_the_sources_are_gathered_and_saved_again() {
    let mut bench = Bench::new("regenerate-unsaved");
    let uuid = begun(&mut bench);
    fs::remove_dir_all(cached(&bench, &uuid, "gathered")).unwrap();

    assert_eq!(regathered(&mut bench, &uuid), GATHERED);
    assert!(set(&bench, &uuid).is_file());
}

#[test]
fn a_corrupt_set_is_gathered_again() {
    let mut bench = Bench::new("regenerate-corrupt");
    let uuid = begun(&mut bench);
    fs::write(set(&bench, &uuid), "{").unwrap();

    assert_eq!(regathered(&mut bench, &uuid), GATHERED);
}

#[test]
fn a_set_saved_for_another_map_row_is_gathered_again() {
    let mut bench = Bench::new("regenerate-stale");
    let uuid = begun(&mut bench);
    let path = set(&bench, &uuid);
    let mut kept: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    kept["row"] = Value::from("tracker\nДругой этап с тем же id\n1–2");
    fs::write(&path, kept.to_string()).unwrap();

    assert_eq!(regathered(&mut bench, &uuid), GATHERED);
}

#[test]
fn a_missing_picture_makes_the_set_a_miss() {
    let mut bench = Bench::new("regenerate-pictureless");
    let uuid = begun(&mut bench);
    fs::remove_dir_all(cached(&bench, &uuid, "pictures")).unwrap();

    assert_eq!(regathered(&mut bench, &uuid), GATHERED);
}

#[test]
fn the_pictures_lie_beside_the_set_and_come_back_into_the_stage() {
    let mut bench = Bench::new("regenerate-pictures");
    let uuid = begun(&mut bench);
    let kept: Value = serde_json::from_slice(&fs::read(set(&bench, &uuid)).unwrap()).unwrap();
    let images = kept["images"].as_array().unwrap();
    assert_eq!(images.len(), 1);
    assert!(images[0].get("bytes").is_none());
    let pictures = cached(&bench, &uuid, "pictures");
    let listed = names(&pictures);
    assert_eq!(listed.len(), 1);
    assert_eq!(
        fs::read(pictures.join(&listed[0])).unwrap(),
        b"\x89PNG thumbnail"
    );
    let recorder = Recorder::default();

    regenerated(
        &mut bench,
        &after(&uuid, "tracker"),
        &said(&[&fine().to_string()]),
        &recorder,
    )
    .unwrap();

    assert_eq!(steps(&recorder), KEPT);
    let asset = format!("assets/{}", images[0]["file"].as_str().unwrap());
    let tree = Library::at(&bench.dir).open(&uuid).unwrap();
    assert!(
        tree.stages["tracker"]
            .every_block()
            .any(|block| block.asset.as_deref() == Some(asset.as_str()))
    );
}
