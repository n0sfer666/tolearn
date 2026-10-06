use tolearn_core::Hours;
use tolearn_core::library::Library;
use tolearn_core::program::{self, StageRow, Tree};
use tolearn_generate::fork::{self, After, Landed, NextError, Variant};
use tolearn_generate::{GenerateError, Step, ledger};

use crate::answers::fine;
use crate::forking::{TAKEN_AT, leaf, proposed, said, steps, taken};
use crate::starting::{Canvas, Recorder, drawn, files, run, split};
use crate::support::Scripted;
use crate::web::{Bench, scripted};

pub const ONE: &str = r#"{"title": "Квантование и форматы", "slug": "quantization", "goal": "Выбирать формат и квантование под своё железо.", "volatility": "volatile", "stages": [{"id": "formats", "title": "GGUF, safetensors и остальные", "hours": [2, 4]}]}"#;
const NESTED: &str = r#"{"title": "Запуск моделей", "slug": "running", "goal": "Запустить модель локально.", "volatility": "volatile", "stages": [], "children": [{"title": "Первый запуск", "goal": "Запустить модель в llama.cpp.", "hours": [10, 20]}, {"title": "Скорость", "goal": "Понять, от чего зависит скорость.", "hours": [10, 20]}]}"#;
const QUANTIZATION: &str = "Выбирать формат и квантование под своё железо.";
const LAST: &str = "последнем уровне";

pub fn answered(parts: &[&str]) -> Scripted {
    let fine = fine().to_string();
    let mut answers = parts.to_vec();
    answers.extend(["sources.txt", fine.as_str()]);
    scripted(&answers)
}

pub fn opened(bench: &mut Bench, children: usize, parts: &[&str]) -> String {
    let mut plan = drawn("split.txt");
    plan.children.truncate(children);
    let model = answered(parts);
    run(bench, &plan, &model, &Canvas(true), &Recorder::default()).unwrap()
}

pub fn from<'a>(program: &'a str, node: &'a str) -> After<'a> {
    After {
        program,
        node,
        stage: "formats",
    }
}

fn library(bench: &Bench, root: &str) -> Tree {
    Library::at(&bench.dir).open(root).unwrap()
}

fn stages(tree: &Tree) -> Vec<&String> {
    tree.stages.keys().collect()
}

#[test]
fn the_last_stage_of_a_part_offers_the_next_part_without_the_model() {
    let mut bench = Bench::new("onward-offer");
    let root = opened(&mut bench, 4, &[ONE]);
    let map = leaf(&bench, &root).map;
    let (first, second) = (&map.children[0], &map.children[1]);
    let at = from(&root, &first.uuid);
    let path = ledger::path(&bench.dir, &root);
    let kept = ledger::read(&path).unwrap().len();
    let model = said(&[]);

    let known = fork::known(&bench.dir, &at).unwrap();
    let offered = proposed(&bench, &at, &model).unwrap();

    assert_eq!(second.goal.as_deref(), Some(QUANTIZATION));
    let variant = Variant {
        row: StageRow {
            id: second.uuid.clone(),
            title: second.title.clone(),
            hours: Hours { min: 30, max: 50 },
        },
        why: QUANTIZATION.to_owned(),
        recommended: true,
    };
    assert_eq!(known.map(|fork| fork.variants), Some(vec![variant.clone()]));
    assert_eq!(offered.variants, [variant]);
    assert!(model.prompts().is_empty());
    assert_eq!(ledger::read(&path).unwrap().len(), kept);
    let cache = bench.dir.join("cache").join(&first.uuid);
    assert!(!cache.join("fork").exists());
}

#[test]
fn taking_the_next_part_expands_it_and_lands_its_first_stage() {
    let mut bench = Bench::new("onward-take");
    let root = opened(&mut bench, 4, &[ONE]);
    let map = leaf(&bench, &root).map;
    let (first, second) = (&map.children[0], &map.children[1]);
    let at = from(&root, &first.uuid);
    let folder = bench.dir.join("programs").join(&root);
    let before = files(&folder);
    let path = ledger::path(&bench.dir, &root);
    let kept = ledger::read(&path).unwrap().len();
    let (model, recorder) = (split(), Recorder::default());

    let landed = taken(&mut bench, &at, 0, &model, &recorder).unwrap();

    let node = second.uuid.clone();
    let stage = "formats".to_owned();
    assert_eq!(landed, Landed { node, stage });
    let tree = library(&bench, &root);
    let child = &tree.children[&second.uuid];
    assert_eq!(child.program.goal, QUANTIZATION);
    assert_eq!(child.program.map.stages.len(), 3);
    assert_eq!(stages(child), ["formats"]);
    assert_eq!(child.program.sources.pages.len(), 2);
    let now = files(&folder);
    for (name, bytes) in &before {
        assert_eq!(now.get(name), Some(bytes), "{name:?}");
    }
    assert_eq!(
        steps(&recorder),
        [
            Step::Part,
            Step::Sources,
            Step::Text,
            Step::Diagrams,
            Step::Write
        ]
    );
    let prompt = &model.prompts()[0];
    let goal = format!("«{}». Её цель: {QUANTIZATION}", second.title);
    assert!(prompt.contains(&goal) && !prompt.contains(LAST), "{prompt}");
    let records = ledger::read(&path).unwrap();
    let added = &records[kept..];
    let part = &added[0];
    assert_eq!(part.step, Step::Part.label());
    assert_eq!(part.program.as_deref(), Some(second.uuid.as_str()));
    assert_eq!(part.stage, None);
    let last = added.last().unwrap();
    assert_eq!(last.program.as_deref(), Some(second.uuid.as_str()));
    assert_eq!(last.stage.as_deref(), Some("formats"));
    assert!(added.iter().all(|record| record.at == Some(TAKEN_AT)));
    assert!(!bench.dir.join("cache").join(&root).join("build").exists());
    let again = fork::known(&bench.dir, &at).unwrap_err();
    assert_eq!(again.code(), "next.taken");
    assert!(again.to_string().contains(&second.title), "{again}");
}

#[test]
fn the_transition_moves_to_the_next_sibling_and_climbs_past_the_last_one() {
    let mut bench = Bench::new("onward-climb");
    let root = opened(&mut bench, 2, &[NESTED, ONE]);
    let top = leaf(&bench, &root).map.children;
    let inner = library(&bench, &root).children[&top[0].uuid]
        .program
        .map
        .children
        .clone();
    let sibling = answered(&[ONE]);

    let near = taken(
        &mut bench,
        &from(&root, &inner[0].uuid),
        0,
        &sibling,
        &Recorder::default(),
    )
    .unwrap();

    assert_eq!(near.node, inner[1].uuid);
    let prompts = sibling.prompts();
    assert!(prompts[0].contains(LAST), "{prompts:?}");
    let at = from(&root, &inner[1].uuid);
    let offered = fork::known(&bench.dir, &at).unwrap().unwrap();
    assert_eq!(offered.variants[0].row.id, top[1].uuid);
    let climbed = answered(&[NESTED, ONE]);

    let far = taken(&mut bench, &at, 0, &climbed, &Recorder::default()).unwrap();

    let tree = library(&bench, &root);
    let near = &tree.children[&top[0].uuid].children[&inner[1].uuid];
    assert_eq!(stages(near), ["formats"]);
    let part = &tree.children[&top[1].uuid];
    let first = &part.program.map.children[0].uuid;
    let (node, stage) = (first.clone(), "formats".to_owned());
    assert_eq!(far, Landed { node, stage });
    assert!(part.stages.is_empty());
    assert_eq!(stages(&part.children[first]), ["formats"]);
    let prompts = climbed.prompts();
    assert!(
        !prompts[0].contains(LAST) && prompts[1].contains(LAST),
        "{prompts:?}"
    );
}

#[test]
fn the_last_stage_of_the_last_part_ends_the_program() {
    let mut bench = Bench::new("onward-end");
    let root = opened(&mut bench, 1, &[ONE]);
    let only = leaf(&bench, &root).map.children[0].uuid.clone();

    let error = fork::known(&bench.dir, &from(&root, &only)).unwrap_err();

    assert_eq!(
        error,
        GenerateError::Next(NextError::End("formats".to_owned()))
    );
    let text = error.to_string();
    assert!(
        text.contains("последний в программе") && !text.contains("позже"),
        "{text}"
    );
}

#[test]
fn a_part_row_without_a_goal_is_expanded_with_the_goal_of_its_parent() {
    let mut bench = Bench::new("onward-goal");
    let root = opened(&mut bench, 2, &[ONE]);
    let mut parent = leaf(&bench, &root);
    parent.map.children[1].goal = None;
    let file = bench.dir.join("programs").join(&root).join("program.yaml");
    std::fs::write(&file, program::write(&parent).unwrap()).unwrap();
    let at = from(&root, &parent.map.children[0].uuid);
    let model = split();

    let offered = fork::known(&bench.dir, &at).unwrap().unwrap();
    taken(&mut bench, &at, 0, &model, &Recorder::default()).unwrap();

    assert_eq!(offered.variants[0].why, parent.goal);
    let prompt = &model.prompts()[0];
    let goal = format!("Её цель: {}", parent.goal);
    assert!(prompt.contains(&goal), "{prompt}");
}
