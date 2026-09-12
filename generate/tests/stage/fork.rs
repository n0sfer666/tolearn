use tolearn_core::Hours;
use tolearn_core::library::Library;
use tolearn_core::program::{self, StageRow};
use tolearn_generate::fork;
use tolearn_generate::{GenerateError, REPAIRS, ledger};

use crate::forking::{ALONE, FORKED_AT, after, begun, leaf, noise, offer, proposed, said};
use crate::starting::{Canvas, Recorder, drawn, flat, run};
use crate::web::Bench;

#[test]
fn the_fork_offers_the_next_map_row_first_and_then_its_alternatives() {
    let mut bench = Bench::new("fork-offer");
    let uuid = begun(&mut bench);
    let answer = noise();
    let model = said(&[&answer]);

    let fork = proposed(&bench, &after(&uuid, "tracker"), &model).unwrap();

    assert_eq!(fork.variants.len(), 2);
    assert_eq!(fork.variants[0].row, leaf(&bench, &uuid).map.stages[1]);
    assert!(!fork.variants[0].recommended);
    assert!(!fork.variants[0].why.is_empty());
    let noise = StageRow {
        id: "noise".to_owned(),
        title: "Этап noise".to_owned(),
        hours: Hours { min: 2, max: 3 },
    };
    assert_eq!(fork.variants[1].row, noise);
    assert!(fork.variants[1].recommended);
    let prompts = model.prompts();
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains("✓ 1. tracker"), "{}", prompts[0]);
    assert!(prompts[0].contains("→ 2. voices"), "{}", prompts[0]);
}

#[test]
fn without_alternatives_the_next_row_is_the_recommended_one() {
    let mut bench = Bench::new("fork-alone");
    let uuid = begun(&mut bench);

    let fork = proposed(&bench, &after(&uuid, "tracker"), &said(&[ALONE])).unwrap();

    assert_eq!(fork.variants.len(), 1);
    assert!(fork.variants[0].recommended);
    assert_eq!(fork.variants[0].why, "Дальше по карте");
}

#[test]
fn two_recommended_rows_go_back_for_repair_and_both_calls_land_in_the_ledger() {
    let mut bench = Bench::new("fork-repair");
    let uuid = begun(&mut bench);
    let path = ledger::path(&bench.dir, &uuid);
    let before = ledger::read(&path).unwrap();
    let (twice, fine) = (offer(true, &[("noise", [2, 3], true)]), noise());
    let model = said(&[&twice, &fine]);

    let fork = proposed(&bench, &after(&uuid, "tracker"), &model).unwrap();

    assert!(!fork.variants[0].recommended && fork.variants[1].recommended);
    let repair = &model.prompts()[1];
    assert!(
        repair.contains("recommended стоит у 2 вариантов"),
        "{repair}"
    );
    let records = ledger::read(&path).unwrap();
    assert_eq!(records[..before.len()], before[..]);
    let added = &records[before.len()..];
    let rounds: Vec<Option<usize>> = added.iter().map(|record| record.round).collect();
    assert_eq!(rounds, [None, Some(1)]);
    assert!(
        added.iter().all(|record| record.step == "fork"
            && record.program.as_deref() == Some(uuid.as_str())
            && record.stage.is_none()
            && record.at == Some(FORKED_AT)),
        "{added:?}"
    );
}

#[test]
fn alternatives_with_no_recommended_one_go_back_for_repair() {
    let mut bench = Bench::new("fork-unrecommended");
    let uuid = begun(&mut bench);
    let (none, fine) = (offer(false, &[("noise", [2, 3], false)]), noise());
    let model = said(&[&none, &fine]);

    let fork = proposed(&bench, &after(&uuid, "tracker"), &model).unwrap();

    assert!(fork.variants[1].recommended);
    let prompts = model.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(
        prompts[1].contains("recommended стоит у 0 вариантов"),
        "{}",
        prompts[1]
    );
}

#[test]
fn an_alternative_that_overflows_the_leaf_is_sent_back_for_repair() {
    let mut bench = Bench::new("fork-overflow");
    let mut plan = drawn("flat.txt");
    plan.stages.truncate(2);
    plan.stages[1].hours = Hours { min: 2, max: 2 };
    plan.stages.extend((1..=16).map(|number| StageRow {
        id: format!("extra-{number}"),
        title: format!("Этап {number}"),
        hours: Hours { min: 2, max: 4 },
    }));
    let uuid = run(
        &mut bench,
        &plan,
        &flat(),
        &Canvas(true),
        &Recorder::default(),
    )
    .unwrap();
    let (wide, fine) = (offer(false, &[("noise", [2, 4], true)]), noise());
    let model = said(&[&wide, &fine]);

    let fork = proposed(&bench, &after(&uuid, "tracker"), &model).unwrap();

    assert_eq!(fork.variants[1].row.hours, Hours { min: 2, max: 3 });
    let prompts = model.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(prompts[1].contains("71"), "{}", prompts[1]);
}

#[test]
fn a_fork_that_keeps_breaking_the_rules_is_refused_and_nothing_is_kept() {
    let mut bench = Bench::new("fork-unrepaired");
    let uuid = begun(&mut bench);
    let broken = offer(
        false,
        &[
            ("Шум", [2, 3], true),
            ("notes", [2, 3], false),
            ("noise", [1, 5], false),
        ],
    );
    let model = said(&[&broken]);
    let after = after(&uuid, "tracker");

    let result = proposed(&bench, &after, &model);

    let Err(GenerateError::Unrepaired { what, flaws }) = result else {
        panic!("{result:?}");
    };
    assert_eq!(what, "развилку");
    assert_eq!(flaws.len(), 4, "{flaws:?}");
    assert_eq!(model.prompts().len(), REPAIRS + 1);
    assert_eq!(fork::known(&bench.dir, &after), Ok(None));
}

#[test]
fn a_reopened_fork_comes_from_the_cache_without_the_model() {
    let mut bench = Bench::new("fork-kept");
    let uuid = begun(&mut bench);
    let after = after(&uuid, "tracker");
    let fork = proposed(&bench, &after, &said(&[&noise()])).unwrap();
    let silent = said(&["не спрашивали"]);

    assert_eq!(fork::known(&bench.dir, &after), Ok(Some(fork.clone())));
    assert_eq!(proposed(&bench, &after, &silent), Ok(fork));
    assert!(silent.prompts().is_empty());
}

#[test]
fn a_fork_kept_for_an_older_map_is_not_offered() {
    let mut bench = Bench::new("fork-stale");
    let uuid = begun(&mut bench);
    let after = after(&uuid, "tracker");
    proposed(&bench, &after, &said(&[&noise()])).unwrap();
    let library = Library::at(&bench.dir);
    let tree = library.open(&uuid).unwrap();
    let edit = bench.dir.join("edit");
    library.copy(&tree, &edit).unwrap();
    let mut program = tree.program.clone();
    program.map.stages[1].title = "Голоса NES, заново".to_owned();
    std::fs::write(edit.join("program.yaml"), program::write(&program).unwrap()).unwrap();
    library.replace(&edit).unwrap();

    assert_eq!(fork::known(&bench.dir, &after), Ok(None));
}
