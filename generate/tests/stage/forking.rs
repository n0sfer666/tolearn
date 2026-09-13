use serde_json::{Value, json};
use tolearn_core::library::Library;
use tolearn_core::program::Program;
use tolearn_generate::fork::{self, After, Fork};
use tolearn_generate::regenerate;
use tolearn_generate::start::Kit;
use tolearn_generate::{GenerateError, Model, Step, online};
use tolearn_offline::page::AsFetched;

use crate::starting::{Canvas, Recorder, drawn, flat, paired, run};
use crate::support::{Scripted, Up};
use crate::web::{Bench, Web};

pub const FORKED_AT: i64 = 2_000;
pub const TAKEN_AT: i64 = 3_000;
pub const ALONE: &str = r#"{"next": {"why": "Дальше по карте"}}"#;

pub fn begun(bench: &mut Bench) -> String {
    run(
        bench,
        &drawn("flat.txt"),
        &flat(),
        &Canvas(true),
        &Recorder::default(),
    )
    .unwrap()
}

pub fn after<'a>(program: &'a str, stage: &'a str) -> After<'a> {
    After {
        program,
        node: program,
        stage,
    }
}

pub fn offer(next: bool, alternatives: &[(&str, [u32; 2], bool)]) -> String {
    let alternatives: Vec<Value> = alternatives
        .iter()
        .map(|(id, hours, recommended)| {
            json!({
                "id": id,
                "title": format!("Этап {id}"),
                "hours": hours,
                "why": format!("{id} ближе к цели"),
                "recommended": recommended,
            })
        })
        .collect();
    json!({
        "next": { "why": "Голоса идут сразу за первым звуком", "recommended": next },
        "alternatives": alternatives,
    })
    .to_string()
}

pub fn noise() -> String {
    offer(false, &[("noise", [2, 3], true)])
}

pub fn steps(recorder: &Recorder) -> Vec<Step> {
    let heard = recorder.heard();
    paired(&heard);
    heard.iter().step_by(2).map(|(_, step)| *step).collect()
}

pub fn said(answers: &[&str]) -> Scripted {
    Scripted::new(answers.iter().map(|answer| (*answer).to_owned()).collect())
}

pub fn proposed(
    bench: &Bench,
    after: &After<'_>,
    model: &dyn Model,
) -> Result<Fork, GenerateError> {
    fork::propose(
        &online(&Up, model).unwrap(),
        &bench.dir,
        after,
        FORKED_AT,
        &[],
    )
}

pub fn forked(bench: &mut Bench, answer: &str) -> String {
    let uuid = begun(bench);
    proposed(bench, &after(&uuid, "tracker"), &said(&[answer])).unwrap();
    uuid
}

pub fn taken(
    bench: &mut Bench,
    after: &After<'_>,
    choice: usize,
    model: &dyn Model,
    recorder: &Recorder,
) -> Result<String, GenerateError> {
    kitted(bench, model, recorder, |kit| {
        fork::take(kit, after, choice, &[])
    })
}

pub fn regenerated(
    bench: &mut Bench,
    at: &After<'_>,
    model: &dyn Model,
    recorder: &Recorder,
) -> Result<(), GenerateError> {
    kitted(bench, model, recorder, |kit| {
        regenerate::regenerate(kit, at)
    })
}

fn kitted<T>(
    bench: &mut Bench,
    model: &dyn Model,
    recorder: &Recorder,
    work: impl FnOnce(Kit<'_>) -> T,
) -> T {
    let online = online(&Up, model).unwrap();
    let kit = Kit {
        online: &online,
        source: &Web,
        renderer: &AsFetched,
        store: &mut bench.store,
        painter: &Canvas(true),
        progress: recorder,
        stop: &recorder.stop,
        data: &bench.dir,
        at: TAKEN_AT,
    };
    work(kit)
}

pub fn leaf(bench: &Bench, uuid: &str) -> Program {
    Library::at(&bench.dir).open(uuid).unwrap().program
}
