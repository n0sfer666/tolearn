use std::collections::BTreeMap;
use std::path::Path;

use tolearn_core::library::Library;
use tolearn_generate::{GenerateError, Model, REPAIRS, Step};
use tolearn_provider::{CheckError, Said, Stop};

use crate::starting::{
    Canvas, Heeding, Recorder, drawn, files, flat, paired, run, split, untouched,
};
use crate::support::Scripted;
use crate::web::{Bench, scripted};

fn halted(step: Step) {
    let mut bench = Bench::new(&format!("cancel-{}", step.label()));
    Library::at(&bench.dir)
        .install(Path::new("../examples/chiptune"))
        .unwrap();
    let before = files(&bench.dir.join("programs"));
    let recorder = Recorder::halting(step);
    let (plan, script) = if step == Step::Part {
        ("split.txt", split())
    } else {
        ("flat.txt", flat())
    };
    let model = Heeding {
        script,
        stop: &recorder.stop,
    };

    let result = run(&mut bench, &drawn(plan), &model, &Canvas(true), &recorder);
    assert_eq!(result, Err(GenerateError::Cancelled));
    assert_eq!(recorder.pressed(), Some(true));
    untouched(&bench, &before);
    let heard = recorder.heard();
    paired(&heard);
    assert_eq!(heard.last(), Some(&("ended", step)));
}

#[test]
fn a_cancel_while_expanding_a_part_leaves_the_library_as_it_was() {
    halted(Step::Part);
}

#[test]
fn a_cancel_while_gathering_sources_leaves_the_library_as_it_was() {
    halted(Step::Sources);
}

#[test]
fn a_cancel_while_writing_the_text_leaves_the_library_as_it_was() {
    halted(Step::Text);
}

#[test]
fn a_cancel_while_drawing_diagrams_leaves_the_library_as_it_was() {
    halted(Step::Diagrams);
}

#[test]
fn a_cancel_while_writing_the_stage_leaves_the_library_as_it_was() {
    halted(Step::Write);
}

#[derive(Debug)]
struct Failing(CheckError);

impl Model for Failing {
    fn ask(&self, _prompt: &str) -> Result<Said, CheckError> {
        Err(self.0.clone())
    }
}

fn failed(label: &str, model: &dyn Model) -> (Result<String, GenerateError>, Recorder, Bench) {
    let mut bench = Bench::new(label);
    let recorder = Recorder::default();
    let result = run(
        &mut bench,
        &drawn("flat.txt"),
        model,
        &Canvas(true),
        &recorder,
    );
    untouched(&bench, &BTreeMap::new());
    paired(&recorder.heard());
    (result, recorder, bench)
}

#[test]
fn a_model_that_reports_a_cancel_cancels_the_start() {
    let (result, _, _) = failed("cancel-model", &Failing(CheckError::Cancelled));
    assert_eq!(result, Err(GenerateError::Cancelled));
}

#[test]
fn a_model_that_fails_leaves_no_program() {
    let (result, _, _) = failed("cancel-provider", &Failing(CheckError::TimedOut(30)));
    assert_eq!(
        result,
        Err(GenerateError::Provider(CheckError::TimedOut(30)))
    );
}

#[test]
fn a_stage_the_model_cannot_mend_leaves_no_program() {
    let model = scripted(&["sources.txt", "этап напишу позже"]);
    let (result, recorder, _) = failed("cancel-unrepaired", &model);
    assert!(
        matches!(result, Err(GenerateError::Unrepaired { .. })),
        "{result:?}"
    );
    let repairs = recorder
        .heard()
        .iter()
        .filter(|(_, step)| matches!(step, Step::Repair(_)))
        .count();
    assert_eq!(repairs, 2 * REPAIRS);
}

#[test]
fn a_library_that_cannot_take_the_program_leaves_no_leftovers() {
    let mut bench = Bench::new("cancel-library");
    std::fs::write(bench.dir.join("programs"), "").unwrap();
    let recorder = Recorder::default();

    let result = run(
        &mut bench,
        &drawn("flat.txt"),
        &flat(),
        &Canvas(true),
        &recorder,
    );
    assert!(
        matches!(result, Err(GenerateError::Library(_))),
        "{result:?}"
    );
    assert!(bench.dir.join("programs").is_file());
    untouched(&bench, &BTreeMap::new());
    paired(&recorder.heard());
}

#[derive(Debug)]
struct Pressing<'a> {
    script: Scripted,
    stop: &'a Stop,
}

impl Model for Pressing<'_> {
    fn ask(&self, prompt: &str) -> Result<Said, CheckError> {
        if self.stop.stopped() {
            return Err(CheckError::Cancelled);
        }
        let said = self.script.ask(prompt);
        self.stop.stop();
        said
    }
}

#[test]
fn a_cancel_right_after_the_sources_are_proposed_checks_none_of_them() {
    let mut bench = Bench::new("cancel-proposed");
    let recorder = Recorder::default();
    let model = Pressing {
        script: flat(),
        stop: &recorder.stop,
    };

    let result = run(
        &mut bench,
        &drawn("flat.txt"),
        &model,
        &Canvas(true),
        &recorder,
    );

    assert_eq!(result, Err(GenerateError::Cancelled));
    assert_eq!(
        files(&bench.dir.join("cache").join("objects")),
        BTreeMap::new()
    );
    untouched(&bench, &BTreeMap::new());
}

#[test]
fn a_cancel_once_the_program_is_installing_is_refused_and_the_program_stays() {
    let mut bench = Bench::new("cancel-late");
    let recorder = Recorder::late(Step::Write);
    let model = Heeding {
        script: flat(),
        stop: &recorder.stop,
    };

    let uuid = run(
        &mut bench,
        &drawn("flat.txt"),
        &model,
        &Canvas(true),
        &recorder,
    )
    .unwrap();

    assert_eq!(recorder.pressed(), Some(false));
    assert!(Library::at(&bench.dir).open(&uuid).is_ok());
}
