use tolearn_generate::fork;
use tolearn_generate::{GenerateError, REPAIRS, Step, ledger};

use crate::forking::{TAKEN_AT, leaf, said, taken};
use crate::onward::{ONE, from, opened};
use crate::starting::{Heeding, Recorder, files, paired, split};
use crate::web::Bench;

const CASES: [(&str, Option<Step>); 4] = [
    ("part", Some(Step::Part)),
    ("text", Some(Step::Text)),
    ("write", Some(Step::Write)),
    ("garbage", None),
];

#[test]
fn a_transition_cancelled_or_unrepaired_leaves_the_programs_and_keeps_the_offer() {
    let garbage = vec!["не карта"; REPAIRS + 1];
    for (name, step) in CASES {
        let mut bench = Bench::new(&format!("onward-cancel-{name}"));
        let root = opened(&mut bench, 2, &[ONE]);
        let map = leaf(&bench, &root).map;
        let (first, second) = (&map.children[0], &map.children[1]);
        let at = from(&root, &first.uuid);
        let programs = files(&bench.dir.join("programs"));
        let path = ledger::path(&bench.dir, &root);
        let kept = ledger::read(&path).unwrap();
        let recorder = step.map_or_else(Recorder::default, Recorder::halting);
        let script = match step {
            Some(_) => split(),
            None => said(&garbage),
        };
        let model = Heeding {
            script,
            stop: &recorder.stop,
        };

        let result = taken(&mut bench, &at, 0, &model, &recorder);

        match step {
            Some(_) => assert_eq!(result, Err(GenerateError::Cancelled), "{name}"),
            None => assert!(
                matches!(result, Err(GenerateError::Unrepaired { .. })),
                "{result:?}"
            ),
        }
        assert_eq!(files(&bench.dir.join("programs")), programs, "{name}");
        let cache = bench.dir.join("cache");
        assert!(!cache.join(&root).join("build").exists(), "{name}");
        assert!(!cache.join(&second.uuid).exists(), "{name}");
        assert!(fork::known(&bench.dir, &at).unwrap().is_some(), "{name}");
        let records = ledger::read(&path).unwrap();
        assert_eq!(records[..kept.len()], kept[..], "{name}");
        let added = &records[kept.len()..];
        assert_eq!(
            added.is_empty(),
            step == Some(Step::Part),
            "{name}: {added:?}"
        );
        assert!(
            added
                .iter()
                .all(|record| record.stage.is_none() && record.at == Some(TAKEN_AT)),
            "{name}: {added:?}"
        );
        paired(&recorder.heard());
    }
}
