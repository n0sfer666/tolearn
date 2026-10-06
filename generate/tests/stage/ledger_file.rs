use tolearn_generate::ledger;
use tolearn_generate::{Step, online};

use crate::metered::{Metered, spent};
use crate::support::Up;
use crate::web::{Bench, scripted};

#[test]
fn the_ledger_appends_and_reads_back_with_missing_counts_as_null() {
    let bench = Bench::new("ledger-file");
    let path = ledger::path(&bench.dir, "program");
    assert!(ledger::read(&path).unwrap().is_empty());
    let model = Metered::new(scripted(&["ответ"]), vec![spent(None, Some(3))]);
    let gate = online(&Up, &model).unwrap();

    gate.ask(Step::Text, "первый").unwrap();
    ledger::append(&path, &gate.tally().take()).unwrap();
    gate.ask(Step::Repair(1), "второй").unwrap();
    ledger::append(&path, &gate.tally().take()).unwrap();

    let records = ledger::read(&path).unwrap();
    let steps: Vec<(&str, Option<usize>)> = records
        .iter()
        .map(|record| (record.step.as_str(), record.round))
        .collect();
    assert_eq!(steps, [("text", None), ("repair", Some(1))]);
    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(text.lines().count(), 2);
    assert!(text.contains(r#""input":null"#), "{text}");
    assert!(text.contains(r#""output":3"#), "{text}");
}
