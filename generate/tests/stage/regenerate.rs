use serde_json::json;
use tolearn_core::library::Library;
use tolearn_generate::fork::After;
use tolearn_generate::{Step, ledger};

use crate::answers::{CALLOUT, fine};
use crate::forking::{TAKEN_AT, after, begun, leaf, regenerated, said, steps};
use crate::starting::{Canvas, Recorder, drawn, flat, run, split};
use crate::web::{ARTICLE, Bench, DOCS};

const STRANGER: &str = "00000000-0000-4000-8000-000000000000";

fn ids(bench: &Bench, uuid: &str) -> Vec<String> {
    Library::at(&bench.dir).open(uuid).unwrap().stages["tracker"]
        .every_block()
        .map(|block| block.id.clone())
        .collect()
}

#[test]
fn the_same_answer_keeps_the_block_ids_and_skips_the_sources() {
    let mut bench = Bench::new("regenerate-same");
    let uuid = begun(&mut bench);
    let before = ids(&bench, &uuid);
    let path = ledger::path(&bench.dir, &uuid);
    let kept = ledger::read(&path).unwrap().len();
    let recorder = Recorder::default();
    let model = said(&[&fine().to_string()]);

    regenerated(&mut bench, &after(&uuid, "tracker"), &model, &recorder).unwrap();

    assert_eq!(ids(&bench, &uuid), before);
    assert_eq!(steps(&recorder), [Step::Text, Step::Diagrams, Step::Write]);
    let prompts = model.prompts();
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains(CALLOUT) && prompts[0].contains("иначе"));
    let records = ledger::read(&path).unwrap();
    let added = &records[kept..];
    assert!(!added.is_empty());
    assert!(
        added.iter().all(|record| record.step == "text"
            && record.stage.as_deref() == Some("tracker")
            && record.program.as_deref() == Some(uuid.as_str())
            && record.at == Some(TAKEN_AT)),
        "{added:?}"
    );
    let program = leaf(&bench, &uuid);
    let urls: Vec<&str> = program
        .sources
        .pages
        .iter()
        .map(|page| page.url.as_str())
        .collect();
    assert_eq!(urls, [ARTICLE, DOCS]);
    assert!(!bench.dir.join("cache").join(&uuid).join("build").exists());
}

#[test]
fn a_changed_paragraph_changes_only_its_own_id() {
    let mut bench = Bench::new("regenerate-changed");
    let uuid = begun(&mut bench);
    let before = ids(&bench, &uuid);
    let mut answer = fine();
    answer["blocks"][1]["text"] = json!("У чипа пять голосов, и каждый звучит по-своему.");

    regenerated(
        &mut bench,
        &after(&uuid, "tracker"),
        &said(&[&answer.to_string()]),
        &Recorder::default(),
    )
    .unwrap();

    let now = ids(&bench, &uuid);
    assert_eq!(now.len(), before.len());
    let changed: Vec<usize> = (0..now.len())
        .filter(|at| now[*at] != before[*at])
        .collect();
    assert_eq!(changed, [1]);
}

#[test]
fn a_subprogram_stage_is_regenerated_in_its_branch_and_logged_to_the_root() {
    let mut bench = Bench::new("regenerate-branch");
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
    let before = Library::at(&bench.dir).open(&uuid).unwrap();
    assert!(
        bench
            .dir
            .join("cache")
            .join(&child)
            .join("gathered")
            .is_dir()
    );
    let recorder = Recorder::default();

    regenerated(&mut bench, &at, &said(&[&fine().to_string()]), &recorder).unwrap();

    assert_eq!(steps(&recorder), [Step::Text, Step::Diagrams, Step::Write]);
    let tree = Library::at(&bench.dir).open(&uuid).unwrap();
    assert!(tree.stages.is_empty());
    assert_eq!(tree.children[&child].stages, before.children[&child].stages);
    let records = ledger::read(&ledger::path(&bench.dir, &uuid)).unwrap();
    let last = records.last().unwrap();
    assert_eq!(last.program.as_deref(), Some(child.as_str()));
    assert_eq!(last.stage.as_deref(), Some("formats"));
}

#[test]
fn a_missing_node_row_stage_or_program_is_refused_before_the_model() {
    let mut bench = Bench::new("regenerate-refused");
    let uuid = begun(&mut bench);
    let model = flat();
    let stranger = After {
        program: &uuid,
        node: STRANGER,
        stage: "tracker",
    };

    let codes: Vec<&str> = [
        stranger,
        after(&uuid, "nope"),
        after(&uuid, "voices"),
        after(STRANGER, "tracker"),
    ]
    .iter()
    .map(|at| {
        regenerated(&mut bench, at, &model, &Recorder::default())
            .unwrap_err()
            .code()
    })
    .collect();

    assert_eq!(
        codes,
        [
            "regenerate.node",
            "regenerate.stage",
            "regenerate.ungenerated",
            "library.absent"
        ]
    );
    assert!(model.prompts().is_empty());
}
