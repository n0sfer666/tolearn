use tolearn_generate::ledger::Kind;
use tolearn_generate::online;

use crate::ledger::shape;
use crate::metered::Metered;
use crate::starting::{Canvas, Recorder, drawn, flat, leftovers, started};
use crate::support::Up;
use crate::web::Bench;

#[test]
fn a_failed_call_is_recorded_as_refused_and_a_failed_start_leaves_its_records_in_the_tally() {
    let mut bench = Bench::new("ledger-failed");
    let model = Metered::new(flat(), Vec::new()).broken(1);
    let gate = online(&Up, &model).unwrap();

    let error = started(
        &mut bench,
        &drawn("flat.txt"),
        &gate,
        &Canvas(true),
        &Recorder::default(),
    )
    .unwrap_err();

    assert_eq!(error.code(), "provider.unreachable");
    let records = gate.tally().records();
    assert_eq!(records.len(), 6, "{records:?}");
    assert_eq!(shape(&records[5..]), [("text", Kind::Model, None)]);
    assert!(!records[5].ok);
    assert!(records[5].to_string().ends_with("отказ"), "{}", records[5]);
    assert!(records[..5].iter().all(|record| record.ok), "{records:?}");
    assert!(leftovers(&bench).is_empty(), "{:?}", leftovers(&bench));
}
