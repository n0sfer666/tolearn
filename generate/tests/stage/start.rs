use tolearn_core::block::Kind;
use tolearn_core::library::Library;
use tolearn_generate::start::day;
use tolearn_generate::{GenerateError, Step};

use crate::answers::{fine, ids};
use crate::starting::{Canvas, Recorder, SVG, drawn, flat, leftovers, names, paired, run, split};
use crate::support::request;
use crate::web::{ARTICLE, Bench, DOCS};

fn flat_start(bench: &mut Bench, painter: &Canvas, recorder: &Recorder) -> String {
    run(bench, &drawn("flat.txt"), &flat(), painter, recorder).unwrap()
}

#[test]
fn a_flat_plan_lands_with_its_first_stage_only() {
    let mut bench = Bench::new("start-flat");
    let uuid = flat_start(&mut bench, &Canvas(true), &Recorder::default());

    let tree = Library::at(&bench.dir).open(&uuid).unwrap();
    assert_eq!(tree.program.level, request().level);
    assert_eq!(tree.program.generation.request, request().request);
    assert_eq!(tree.program.map.stages.len(), 8);
    assert_eq!(tree.stages.keys().collect::<Vec<_>>(), ["tracker"]);
    assert_eq!(
        names(&bench.dir.join("programs")),
        std::slice::from_ref(&uuid)
    );
    assert!(!bench.dir.join("cache").join(&uuid).join("build").exists());
}

#[test]
fn the_stage_carries_its_drawn_diagram_and_licensed_image() {
    let mut bench = Bench::new("start-assets");
    let uuid = flat_start(&mut bench, &Canvas(true), &Recorder::default());

    let library = Library::at(&bench.dir);
    let tree = library.open(&uuid).unwrap();
    let stage = &tree.stages["tracker"];
    let every: Vec<String> = stage.every_block().map(|block| block.id.clone()).collect();
    assert_eq!(every, ids(&fine()));
    let diagram = stage
        .every_block()
        .find(|block| block.kind == Kind::Diagram)
        .unwrap();
    let svg = diagram.asset.as_deref().unwrap();
    assert!(svg.ends_with(".svg"), "{svg}");
    assert_eq!(library.asset(&tree, svg).unwrap(), SVG.as_bytes());
    let image = stage
        .every_block()
        .find(|block| block.kind == Kind::Image)
        .unwrap();
    assert!(image.license.is_some() && image.attribution.is_some());
    let png = image.asset.as_deref().unwrap();
    assert_eq!(library.asset(&tree, png).unwrap(), b"\x89PNG thumbnail");
}

#[test]
fn a_diagram_the_window_cannot_draw_stays_mermaid_code_under_its_id() {
    let mut bench = Bench::new("start-undrawn");
    let uuid = flat_start(&mut bench, &Canvas(false), &Recorder::default());

    let tree = Library::at(&bench.dir).open(&uuid).unwrap();
    let stage = &tree.stages["tracker"];
    let every: Vec<String> = stage.every_block().map(|block| block.id.clone()).collect();
    assert_eq!(every, ids(&fine()));
    let code = stage
        .every_block()
        .find(|block| block.lang.as_deref() == Some("mermaid"))
        .unwrap();
    assert_eq!(code.kind, Kind::Code);
    assert!(code.asset.is_none());
    assert!(tree.assets.iter().all(|asset| !asset.ends_with(".svg")));
}

#[test]
fn the_program_lists_the_sources_its_stage_cites() {
    let mut bench = Bench::new("start-sources");
    let uuid = flat_start(&mut bench, &Canvas(true), &Recorder::default());

    let sources = Library::at(&bench.dir).open(&uuid).unwrap().program.sources;
    let urls: Vec<&str> = sources.pages.iter().map(|page| page.url.as_str()).collect();
    assert_eq!(urls, [ARTICLE, DOCS]);
    assert!(
        sources
            .pages
            .iter()
            .all(|page| page.checked_at == "1970-01-01" && !page.title.is_empty())
    );
    assert_eq!(sources.books.len(), 1);
    assert_eq!(sources.books[0].chapter, "2. Push Start Button");
    assert_eq!(sources.books[0].checked_at, "1970-01-01");
}

#[test]
fn every_step_is_announced_at_its_start_and_its_end() {
    let mut bench = Bench::new("start-steps");
    let recorder = Recorder::default();
    flat_start(&mut bench, &Canvas(true), &recorder);

    let heard = recorder.heard();
    paired(&heard);
    let steps: Vec<Step> = heard.iter().step_by(2).map(|(_, step)| *step).collect();
    assert_eq!(
        steps,
        [Step::Sources, Step::Text, Step::Diagrams, Step::Write]
    );
}

#[test]
fn a_split_plan_expands_its_first_part_down_to_a_leaf() {
    let mut bench = Bench::new("start-split");
    let recorder = Recorder::default();
    let model = split();
    let uuid = run(
        &mut bench,
        &drawn("split.txt"),
        &model,
        &Canvas(true),
        &recorder,
    )
    .unwrap();

    let tree = Library::at(&bench.dir).open(&uuid).unwrap();
    assert_eq!(tree.program.map.children.len(), 4);
    assert!(tree.stages.is_empty());
    assert!(tree.program.sources.pages.is_empty());
    let first = &tree.program.map.children[0];
    assert_eq!(tree.children.keys().collect::<Vec<_>>(), [&first.uuid]);
    let child = &tree.children[&first.uuid];
    assert_eq!(child.stages.keys().collect::<Vec<_>>(), ["formats"]);
    assert_eq!(child.program.sources.pages.len(), 2);
    assert!(model.prompts()[0].contains(&first.title));
    assert_eq!(
        recorder.heard()[..2],
        [("began", Step::Part), ("ended", Step::Part)]
    );
    assert!(!leftovers(&bench).contains(&uuid));
}

#[test]
fn an_unfit_plan_is_refused_before_the_model() {
    let mut bench = Bench::new("start-unfit");
    let recorder = Recorder::default();
    let model = flat();
    let mut plan = drawn("flat.txt");
    plan.title.clear();

    let result = run(&mut bench, &plan, &model, &Canvas(true), &recorder);
    assert!(
        matches!(&result, Err(GenerateError::Unfit { flaws }) if !flaws.is_empty()),
        "{result:?}"
    );
    assert_eq!(result.unwrap_err().code(), "generate.unfit");
    assert!(model.prompts().is_empty());
    assert!(recorder.heard().is_empty());
}

#[test]
fn days_are_counted_from_the_epoch() {
    assert_eq!(day(0), "1970-01-01");
    assert_eq!(day(1_000), "1970-01-01");
    assert_eq!(day(951_782_400), "2000-02-29");
    assert_eq!(day(1_757_635_200), "2025-09-12");
    assert_eq!(day(-1), "1969-12-31");
}
