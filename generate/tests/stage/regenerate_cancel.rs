use tolearn_generate::{GenerateError, REPAIRS, Step, ledger};

use crate::answers::fine;
use crate::forking::{TAKEN_AT, after, begun, regenerated, said};
use crate::starting::{Heeding, Recorder, files, paired};
use crate::web::Bench;

fn unstaged(bench: &Bench, uuid: &str, kept: usize) {
    let records = ledger::read(&ledger::path(&bench.dir, uuid)).unwrap();
    let added = &records[kept..];
    assert!(!added.is_empty());
    assert!(
        added.iter().all(|record| record.stage.is_none()
            && record.program.as_deref() == Some(uuid)
            && record.at == Some(TAKEN_AT)),
        "{added:?}"
    );
}

#[test]
fn a_regeneration_cancelled_before_it_lands_keeps_the_stage_and_logs_no_stage() {
    for step in [Step::Text, Step::Write] {
        let mut bench = Bench::new(&format!("regenerate-cancel-{step:?}"));
        let uuid = begun(&mut bench);
        let programs = files(&bench.dir.join("programs"));
        let kept = ledger::read(&ledger::path(&bench.dir, &uuid)).unwrap();
        let recorder = Recorder::halting(step);
        let model = Heeding {
            script: said(&[&fine().to_string()]),
            stop: &recorder.stop,
        };

        let result = regenerated(&mut bench, &after(&uuid, "tracker"), &model, &recorder);

        assert_eq!(result, Err(GenerateError::Cancelled), "{step:?}");
        assert_eq!(files(&bench.dir.join("programs")), programs, "{step:?}");
        assert!(!bench.dir.join("cache").join(&uuid).join("build").exists());
        let records = ledger::read(&ledger::path(&bench.dir, &uuid)).unwrap();
        assert_eq!(records[..kept.len()], kept[..]);
        unstaged(&bench, &uuid, kept.len());
        paired(&recorder.heard());
    }
}

#[test]
fn an_unrepaired_regeneration_keeps_the_stage() {
    let mut bench = Bench::new("regenerate-unrepaired");
    let uuid = begun(&mut bench);
    let programs = files(&bench.dir.join("programs"));
    let kept = ledger::read(&ledger::path(&bench.dir, &uuid))
        .unwrap()
        .len();
    let model = said(&["{}"]);

    let error = regenerated(
        &mut bench,
        &after(&uuid, "tracker"),
        &model,
        &Recorder::default(),
    )
    .unwrap_err();

    assert_eq!(error.code(), "generate.unrepaired");
    assert_eq!(model.prompts().len(), REPAIRS + 1);
    assert_eq!(files(&bench.dir.join("programs")), programs);
    unstaged(&bench, &uuid, kept);
}
