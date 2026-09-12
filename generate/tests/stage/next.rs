use std::path::Path;

use tolearn_core::Hours;
use tolearn_core::library::Library;
use tolearn_generate::fork::{self, After};
use tolearn_generate::{Step, ledger};

use crate::forking::{ALONE, TAKEN_AT, after, begun, forked, leaf, noise, proposed, said, taken};
use crate::starting::{Canvas, Recorder, drawn, files, flat, names, paired, run, split};
use crate::web::{ARTICLE, Bench, DOCS};

#[test]
fn taking_an_alternative_lands_its_stage_in_place_of_the_next_map_row() {
    let mut bench = Bench::new("next-alternative");
    let uuid = forked(&mut bench, &noise());
    let folder = bench.dir.join("programs").join(&uuid);
    let before = files(&folder);
    let recorder = Recorder::default();

    let stage = taken(&mut bench, &after(&uuid, "tracker"), 1, &flat(), &recorder).unwrap();

    assert_eq!(stage, "noise");
    let tree = Library::at(&bench.dir).open(&uuid).unwrap();
    let ids: Vec<&str> = tree
        .program
        .map
        .stages
        .iter()
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(ids[..3], ["tracker", "noise", "notes"]);
    assert_eq!(tree.program.map.stages[1].hours, Hours { min: 2, max: 3 });
    assert_eq!(tree.stages.keys().collect::<Vec<_>>(), ["noise", "tracker"]);
    let now = files(&folder);
    for (name, bytes) in before
        .iter()
        .filter(|(name, _)| *name != Path::new("program.yaml"))
    {
        assert_eq!(now.get(name), Some(bytes), "{name:?}");
    }
    let heard = recorder.heard();
    paired(&heard);
    let steps: Vec<Step> = heard.iter().step_by(2).map(|(_, step)| *step).collect();
    assert_eq!(
        steps,
        [Step::Sources, Step::Text, Step::Diagrams, Step::Write]
    );
    let cache = bench.dir.join("cache").join(&uuid);
    assert!(!cache.join("build").exists());
    assert_eq!(names(&cache.join("fork")), Vec::<String>::new());
    let again = fork::known(&bench.dir, &after(&uuid, "tracker")).unwrap_err();
    assert_eq!(again.code(), "next.taken");
}

#[test]
fn taking_the_next_row_keeps_the_map_and_lists_each_source_once() {
    let mut bench = Bench::new("next-row");
    let uuid = forked(&mut bench, &noise());
    let map = leaf(&bench, &uuid).map;

    let stage = taken(
        &mut bench,
        &after(&uuid, "tracker"),
        0,
        &flat(),
        &Recorder::default(),
    );

    assert_eq!(stage.as_deref(), Ok("voices"));
    let program = leaf(&bench, &uuid);
    assert_eq!(program.map, map);
    let urls: Vec<&str> = program
        .sources
        .pages
        .iter()
        .map(|page| page.url.as_str())
        .collect();
    assert_eq!(urls, [ARTICLE, DOCS]);
    assert_eq!(program.sources.books.len(), 1);
}

#[test]
fn a_take_without_its_fork_or_past_its_variants_is_refused_before_the_model() {
    let mut bench = Bench::new("next-refused");
    let uuid = begun(&mut bench);
    let at = after(&uuid, "tracker");
    let model = flat();

    let unforked = taken(&mut bench, &at, 0, &model, &Recorder::default()).unwrap_err();
    proposed(&bench, &at, &said(&[&noise()])).unwrap();
    let past = taken(&mut bench, &at, 2, &model, &Recorder::default()).unwrap_err();

    assert_eq!(unforked.code(), "next.unforked");
    assert_eq!(past.code(), "next.choice");
    assert!(model.prompts().is_empty());
}

#[test]
fn the_take_logs_its_calls_under_the_stage_it_landed() {
    let mut bench = Bench::new("next-ledger");
    let uuid = forked(&mut bench, &noise());
    let path = ledger::path(&bench.dir, &uuid);
    let kept = ledger::read(&path).unwrap().len();

    taken(
        &mut bench,
        &after(&uuid, "tracker"),
        1,
        &flat(),
        &Recorder::default(),
    )
    .unwrap();

    let records = ledger::read(&path).unwrap();
    let added = &records[kept..];
    assert_eq!(
        added.last().map(|record| record.step.as_str()),
        Some("text")
    );
    assert!(
        added
            .iter()
            .all(|record| record.stage.as_deref() == Some("noise")
                && record.program.as_deref() == Some(uuid.as_str())
                && record.at == Some(TAKEN_AT)),
        "{added:?}"
    );
}

#[test]
fn a_take_inside_a_subprogram_lands_in_its_branch_and_logs_to_the_root() {
    let mut bench = Bench::new("next-branch");
    let uuid = run(
        &mut bench,
        &drawn("split.txt"),
        &split(),
        &Canvas(true),
        &Recorder::default(),
    )
    .unwrap();
    let child = leaf(&bench, &uuid).map.children[0].uuid.clone();
    let at = After {
        program: &uuid,
        node: &child,
        stage: "formats",
    };
    proposed(&bench, &at, &said(&[ALONE])).unwrap();

    let stage = taken(&mut bench, &at, 0, &flat(), &Recorder::default());

    assert_eq!(stage.as_deref(), Ok("bits"));
    let tree = Library::at(&bench.dir).open(&uuid).unwrap();
    assert!(tree.stages.is_empty());
    let landed: Vec<&String> = tree.children[&child].stages.keys().collect();
    assert_eq!(landed, ["bits", "formats"]);
    let records = ledger::read(&ledger::path(&bench.dir, &uuid)).unwrap();
    let last = records.last().unwrap();
    assert_eq!(last.program.as_deref(), Some(child.as_str()));
    assert_eq!(last.stage.as_deref(), Some("bits"));
}
