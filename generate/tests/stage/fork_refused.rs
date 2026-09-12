use tolearn_generate::GenerateError;
use tolearn_generate::fork::{self, After, NextError};

use crate::forking::{after, begun, noise, proposed, said};
use crate::starting::{Canvas, Recorder, drawn, flat, run};
use crate::web::Bench;

const STRANGER: &str = "00000000-0000-4000-8000-000000000000";

fn refused(bench: &Bench, after: &After<'_>) -> GenerateError {
    let model = said(&[&noise()]);
    let error = proposed(bench, after, &model).unwrap_err();
    assert!(model.prompts().is_empty());
    assert_eq!(fork::known(&bench.dir, after), Err(error.clone()));
    error
}

#[test]
fn a_fork_is_refused_before_the_model_where_there_is_nothing_to_fork() {
    let mut bench = Bench::new("fork-refused");
    let uuid = begun(&mut bench);
    let stranger = After {
        program: &uuid,
        node: STRANGER,
        stage: "tracker",
    };

    for (after, code) in [
        (after(&uuid, "voices"), "next.ungenerated"),
        (after(&uuid, "nope"), "next.stage"),
        (stranger, "next.node"),
        (after(STRANGER, "tracker"), "library.absent"),
    ] {
        assert_eq!(refused(&bench, &after).code(), code, "{after:?}");
    }
}

#[test]
fn the_last_stage_of_the_map_has_no_fork_yet() {
    let mut bench = Bench::new("fork-end");
    let mut plan = drawn("flat.txt");
    plan.stages.truncate(1);
    let uuid = run(
        &mut bench,
        &plan,
        &flat(),
        &Canvas(true),
        &Recorder::default(),
    )
    .unwrap();

    let error = refused(&bench, &after(&uuid, "tracker"));

    assert_eq!(
        error,
        GenerateError::Next(NextError::End("tracker".to_owned()))
    );
    assert!(error.to_string().contains("последний"), "{error}");
}
