use tolearn_core::library::Library;
use tolearn_generate::ledger::{self, Kind, Record, Total};
use tolearn_generate::{online, plan};

use crate::answers::fine;
use crate::metered::{MODEL, Metered, spent};
use crate::starting::{Canvas, Recorder, drawn, flat, run, started};
use crate::support::{Up, answer, request};
use crate::web::{ARTICLE, Bench, DOCS, scripted};

pub const ISBN: &str = "978-0-262-03378-7";
pub const QUERY: &str = "NES APU pulse wave";

pub fn shape(records: &[Record]) -> Vec<(&str, Kind, Option<&str>)> {
    records
        .iter()
        .map(|record| (record.step.as_str(), record.kind, record.target.as_deref()))
        .collect()
}

#[test]
fn every_model_call_and_source_check_lands_in_the_ledger_in_pipeline_order() {
    let mut bench = Bench::new("ledger-flat");
    let model = Metered::new(
        flat(),
        vec![spent(Some(900), Some(300)), spent(Some(5_000), None)],
    );

    let uuid = run(
        &mut bench,
        &drawn("flat.txt"),
        &model,
        &Canvas(true),
        &Recorder::default(),
    )
    .unwrap();

    let records = ledger::read(&ledger::path(&bench.dir, &uuid)).unwrap();
    assert_eq!(
        shape(&records),
        [
            ("sources", Kind::Model, None),
            ("sources", Kind::Book, Some(ISBN)),
            ("sources", Kind::Page, Some(ARTICLE)),
            ("sources", Kind::Page, Some(DOCS)),
            ("sources", Kind::Image, Some(QUERY)),
            ("text", Kind::Model, None),
        ]
    );
    assert!(
        records.iter().all(|record| record.ok
            && record.at == Some(1_000)
            && record.program.as_deref() == Some(uuid.as_str())
            && record.stage.as_deref() == Some("tracker")),
        "{records:?}"
    );
    assert_eq!(
        (records[0].input, records[0].output),
        (Some(900), Some(300))
    );
    assert_eq!((records[5].input, records[5].output), (Some(5_000), None));
    assert_eq!(records[0].model.as_deref(), Some(MODEL));
    assert!(
        records[1..5]
            .iter()
            .all(|record| record.model.is_none() && record.input.is_none()),
        "{records:?}"
    );
    assert!(
        records[5].to_string().contains("нет данных"),
        "{}",
        records[5]
    );
    assert!(
        !records[0].to_string().contains("нет данных"),
        "{}",
        records[0]
    );

    let total = Total::of(&records);
    assert_eq!((total.calls, total.checks), (2, 4));
    assert_eq!((total.input, total.output), (5_900, 300));
    assert_eq!(
        total.ms,
        records.iter().map(|record| record.ms).sum::<u64>()
    );
    assert_eq!(total.unknown, 1);
    assert!(!total.complete());
    assert_eq!(Total::stage(&records, &uuid, "tracker"), total);
    assert_eq!(Total::stage(&records, "чужая", "tracker"), Total::default());
}

#[test]
fn records_carried_in_come_first_and_each_part_belongs_to_its_program() {
    let mut bench = Bench::new("ledger-split");
    let fine = fine().to_string();
    let script = scripted(&[
        "не карта",
        &answer("split.txt"),
        &answer("part.txt"),
        "sources.txt",
        &fine,
    ]);
    let model = Metered::new(script, Vec::new());
    let gate = online(&Up, &model).unwrap();
    let plan = plan::plan(&gate, &request()).unwrap();

    let uuid = started(
        &mut bench,
        &plan,
        &gate,
        &Canvas(true),
        &Recorder::default(),
    )
    .unwrap();

    let records = ledger::read(&ledger::path(&bench.dir, &uuid)).unwrap();
    let tree = Library::at(&bench.dir).open(&uuid).unwrap();
    let child = tree.program.map.children[0].uuid.as_str();
    let first = tree.children[child].program.map.stages[0].id.as_str();
    let steps: Vec<(&str, Option<usize>)> = records
        .iter()
        .map(|record| (record.step.as_str(), record.round))
        .collect();
    assert_eq!(
        steps[..4],
        [
            ("plan", None),
            ("plan", Some(1)),
            ("part", None),
            ("sources", None)
        ]
    );
    assert_eq!(steps.last(), Some(&("text", None)));
    let owners: Vec<(Option<&str>, Option<&str>)> = records
        .iter()
        .map(|record| (record.program.as_deref(), record.stage.as_deref()))
        .collect();
    assert_eq!(owners[..2], [(Some(uuid.as_str()), None); 2]);
    assert_eq!(owners[2], (Some(child), None));
    assert!(
        owners[3..]
            .iter()
            .all(|owner| *owner == (Some(child), Some(first))),
        "{owners:?}"
    );
}

#[test]
fn a_replacement_of_sources_and_a_repair_of_the_stage_carry_their_round() {
    let mut bench = Bench::new("ledger-rounds");
    let fine = fine().to_string();
    let script = scripted(&["sources-bad.txt", "sources-replaced.txt", "не этап", &fine]);
    let model = Metered::new(script, Vec::new());

    let uuid = run(
        &mut bench,
        &drawn("flat.txt"),
        &model,
        &Canvas(true),
        &Recorder::default(),
    )
    .unwrap();

    let records = ledger::read(&ledger::path(&bench.dir, &uuid)).unwrap();
    let asked: Vec<(&str, Option<usize>)> = records
        .iter()
        .filter(|record| record.kind == Kind::Model)
        .map(|record| (record.step.as_str(), record.round))
        .collect();
    assert_eq!(
        asked,
        [
            ("sources", None),
            ("sources", Some(1)),
            ("text", None),
            ("repair", Some(1))
        ]
    );
    assert!(
        records
            .iter()
            .any(|record| !record.ok && record.kind == Kind::Book),
        "{records:?}"
    );
}
