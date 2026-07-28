use tolearn_core::prompt::generation;

use crate::repo::read;
use crate::schema::paths::{described_fields, fixed_values};
use crate::schema::schema;

const WRITTEN: [&str; 2] = ["roadmap", "topic"];
const SELF_CHECK: &str = "## Самопроверка";

#[test]
fn the_generation_prompt_ships_with_the_application() {
    let prompt = generation();
    assert!(
        !prompt.trim().is_empty(),
        "the generation prompt is empty, and nothing ships with the application"
    );
    for id in [
        "learning-roadmap/v1",
        "learning-roadmap/topic/v1",
        "learning-roadmap/progress/v1",
    ] {
        assert!(
            prompt.contains(id),
            "the prompt never names `{id}`, so the model has nothing to write into `schema`"
        );
    }
}

#[test]
fn the_generation_prompt_names_every_field_the_generator_writes() {
    let prompt = generation();
    for kind in WRITTEN {
        for path in described_fields(&schema(kind)) {
            assert!(
                prompt.contains(&format!("`{path}`")),
                "{kind}.schema.json: the prompt never names `{path}`, \
                 and a field nobody asked for is a field the model leaves out"
            );
        }
    }
}

#[test]
fn the_generation_prompt_shows_every_value_the_schemas_fix() {
    let prompt = generation();
    for kind in WRITTEN {
        for (path, values) in fixed_values(&schema(kind)) {
            for value in values {
                assert!(
                    prompt.contains(&value),
                    "{kind}.schema.json: `{path}` is fixed to `{value}`, \
                     and the prompt never shows that value"
                );
            }
        }
    }
}

#[test]
fn the_self_check_covers_every_integrity_rule() {
    let prompt = generation();
    let codes = integrity_codes();
    assert!(
        codes.len() >= 11,
        "only {} integrity codes found in the validator, \
         the reader that feeds this gate is broken",
        codes.len()
    );
    for code in codes {
        assert!(
            prompt.contains(&code),
            "the self-check never mentions `{code}`, \
             so the model can hand out a bundle `validate` will reject"
        );
    }
}

#[test]
fn the_self_check_is_a_numbered_procedure_that_gates_the_output() {
    let prompt = generation();
    let procedure = procedure(prompt);
    let steps = procedure.lines().filter(|line| numbered(line)).count();
    assert!(
        steps >= 11,
        "the self-check has {steps} numbered steps, \
         fewer than the integrity rules it has to cover"
    );
    assert!(
        procedure.contains("tolearn validate"),
        "the self-check never ends in `tolearn validate`, \
         so nothing checks the model against the machine"
    );
    assert!(
        procedure.to_lowercase().contains("не выводи"),
        "the self-check never stops a bundle that failed a step, \
         which makes the procedure advice rather than a gate"
    );
}

fn procedure(prompt: &str) -> &str {
    let (_, tail) = prompt
        .split_once(SELF_CHECK)
        .unwrap_or_else(|| panic!("the prompt has no `{SELF_CHECK}` section"));
    tail.split_once("\n## ")
        .map_or(tail, |(section, _)| section)
}

fn numbered(line: &str) -> bool {
    let head = line.trim_start();
    head.starts_with(|first: char| first.is_ascii_digit()) && head.contains(". ")
}

fn integrity_codes() -> Vec<String> {
    let source = read("core/src/bundle/violation.rs");
    let mut codes: Vec<String> = source
        .lines()
        .filter_map(|line| line.split_once("=> \""))
        .filter_map(|(_, rest)| rest.split_once('"'))
        .map(|(code, _)| code.to_owned())
        .filter(|code| code.starts_with("bundle."))
        .collect();
    codes.sort();
    codes.dedup();
    codes
}
