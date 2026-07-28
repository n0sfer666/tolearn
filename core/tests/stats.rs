#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "stats gate: a panic here is the report"
)]

mod support;

use tolearn_core::progress::{Progress, Verdict, parse as progress};
use tolearn_core::stats::{Stats, stats};
use tolearn_core::topic::Topic;

use support::{bundles, read};

fn recorded(file: &str) -> Progress {
    progress(&read(&format!("fixtures/valid/progress/{file}.yaml"))).unwrap()
}

fn topics() -> Vec<Topic> {
    bundles::reference().1
}

fn sampled() -> Stats {
    stats(&topics(), &recorded("attempts-sample"))
}

#[test]
fn на_малой_выборке_вывод_не_делается() {
    let scarce = stats(&topics(), &recorded("attempts-history"));

    assert_eq!(scarce.attempts, 4);
    assert!(!scarce.enough, "{scarce:?}");
    assert!(scarce.kinds.is_empty(), "{scarce:?}");
    assert!(scarce.actions.is_empty(), "{scarce:?}");
    assert_eq!(scarce.hinted, 0);
    assert_eq!(scarce.streak.longest, 0);
}

#[test]
fn калибровка_показана_и_на_малой_выборке() {
    let scarce = stats(&topics(), &recorded("attempts-history"));

    assert!(!scarce.calibration.is_empty(), "{scarce:?}");
    assert!(
        scarce
            .calibration
            .iter()
            .all(|line| !line.trim().is_empty()),
        "{:?}",
        scarce.calibration
    );
}

#[test]
fn доля_подсказок_считается_от_числа_попыток() {
    let taken = sampled();

    assert_eq!(taken.attempts, 7);
    assert!(taken.enough);
    assert_eq!(taken.hinted, 3);
    assert!(
        (taken.hinted_share - 3.0 / 7.0).abs() < 1e-9,
        "{}",
        taken.hinted_share
    );
}

#[test]
fn ответы_разложены_по_типам_вопросов() {
    let taken = sampled();

    let kinds: Vec<&str> = taken.kinds.iter().map(|item| item.kind.as_str()).collect();
    assert!(kinds.contains(&"diagnose"), "{kinds:?}");
    assert!(kinds.contains(&"tradeoff"), "{kinds:?}");
    let diagnose = taken
        .kinds
        .iter()
        .find(|item| item.kind == "diagnose")
        .unwrap();
    assert_eq!(diagnose.ok, 1);
    assert_eq!(diagnose.partial, 1);
    assert_eq!(diagnose.miss, 1);
}

#[test]
fn разложение_не_считает_ответы_дважды() {
    let taken = sampled();

    let counted: u32 = taken
        .kinds
        .iter()
        .map(|item| item.ok + item.partial + item.miss)
        .sum();
    let answers: usize = topics()
        .iter()
        .filter_map(|topic| recorded("attempts-sample").state(&topic.id).cloned())
        .flat_map(|state| state.attempts)
        .map(|attempt| attempt.per_question.len())
        .sum();
    assert_eq!(usize::try_from(counted).unwrap(), answers);
}

#[test]
fn следующие_шаги_идут_от_самого_частого() {
    let taken = sampled();

    let first = taken.actions.first().unwrap();
    assert_eq!(first.action, "retry_failed");
    assert_eq!(first.count, 4);
    let counts: Vec<u32> = taken.actions.iter().map(|item| item.count).collect();
    let mut sorted = counts.clone();
    sorted.sort_unstable_by(|left, right| right.cmp(left));
    assert_eq!(counts, sorted, "{counts:?}");
}

#[test]
fn серия_провалов_названа_вместе_с_темой() {
    let taken = sampled();

    assert_eq!(taken.streak.longest, 3);
    assert_eq!(taken.streak.topic, "structured-output");
}

#[test]
fn зачёт_обрывает_серию_провалов() {
    let taken = sampled();
    let local = recorded("attempts-sample");
    let verdicts: Vec<&str> = local
        .state("local-runtime")
        .unwrap()
        .attempts
        .iter()
        .map(|attempt| attempt.verdict.label())
        .collect();

    assert_eq!(verdicts, ["fail", "fail", "pass"]);
    assert_eq!(taken.streak.topic, "structured-output");
    assert_eq!(taken.streak.longest, 3);
}

#[test]
fn провал_после_зачёта_начинает_серию_заново() {
    let mut local = recorded("attempts-sample");
    let state = local
        .topics
        .iter_mut()
        .find(|(id, _)| id == "local-runtime")
        .map(|(_, state)| state)
        .unwrap();
    let relapse = state.attempts.first().unwrap().clone();
    state.attempts.push(relapse);
    let verdicts: Vec<&str> = state
        .attempts
        .iter()
        .map(|attempt| attempt.verdict.label())
        .collect();
    assert_eq!(verdicts, ["fail", "fail", "pass", "fail"]);

    let taken = stats(&topics(), &local);

    assert_eq!(taken.streak.longest, 3);
    assert_eq!(taken.streak.topic, "structured-output");
}

#[test]
fn без_провалов_серия_не_называет_тему() {
    let mut clean = recorded("attempts-sample");
    for (_, state) in &mut clean.topics {
        for attempt in &mut state.attempts {
            attempt.verdict = Verdict::Pass;
        }
    }

    let taken = stats(&topics(), &clean);

    assert_eq!(taken.streak.longest, 0);
    assert_eq!(taken.streak.topic, "");
}

#[test]
fn калибровка_остаётся_прозой_и_в_счёт_не_идёт() {
    let taken = sampled();

    assert_eq!(taken.calibration.len(), 3);
    let first = taken.calibration.first().unwrap();
    assert!(first.contains(' '), "калибровка сжата в метку: `{first}`");
}
