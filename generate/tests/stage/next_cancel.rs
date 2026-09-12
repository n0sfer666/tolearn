use tolearn_generate::fork;
use tolearn_generate::{GenerateError, Step, ledger};

use crate::forking::{TAKEN_AT, after, forked, noise, taken};
use crate::starting::{Heeding, Recorder, files, flat, paired};
use crate::web::Bench;

#[test]
fn a_take_cancelled_before_it_lands_leaves_the_program_keeps_the_fork_and_logs_no_stage() {
    for step in [Step::Text, Step::Write] {
        let mut bench = Bench::new(&format!("next-cancel-{step:?}"));
        let uuid = forked(&mut bench, &noise());
        let at = after(&uuid, "tracker");
        let programs = files(&bench.dir.join("programs"));
        let path = ledger::path(&bench.dir, &uuid);
        let kept = ledger::read(&path).unwrap();
        let recorder = Recorder::halting(step);
        let model = Heeding {
            script: flat(),
            stop: &recorder.stop,
        };

        let result = taken(&mut bench, &at, 1, &model, &recorder);

        assert_eq!(result, Err(GenerateError::Cancelled), "{step:?}");
        assert_eq!(files(&bench.dir.join("programs")), programs, "{step:?}");
        assert!(!bench.dir.join("cache").join(&uuid).join("build").exists());
        assert!(fork::known(&bench.dir, &at).unwrap().is_some(), "{step:?}");
        let records = ledger::read(&path).unwrap();
        assert_eq!(records[..kept.len()], kept[..]);
        let added = &records[kept.len()..];
        assert!(!added.is_empty(), "{step:?}");
        assert!(
            added.iter().all(|record| record.stage.is_none()
                && record.program.as_deref() == Some(uuid.as_str())
                && record.at == Some(TAKEN_AT)),
            "{step:?}: {added:?}"
        );
        paired(&recorder.heard());
    }
}
