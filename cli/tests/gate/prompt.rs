use tolearn_core::prompt::generation;

use crate::repo::read;
use crate::schema::paths::{described_fields, fixed_values};
use crate::schema::schema;

const SELF_CHECK: &str = "## Самопроверка";

fn written() -> [(&'static str, &'static str); 2] {
    [
        ("roadmap", generation::roadmap()),
        ("topic", generation::topic()),
    ]
}

#[test]
fn both_generation_prompts_ship_with_the_application() {
    for (kind, prompt) in written() {
        assert!(
            !prompt.trim().is_empty(),
            "the {kind} prompt is empty, and nothing ships with the application"
        );
    }
    for (id, prompt) in [
        ("learning-roadmap/v1", generation::roadmap()),
        ("learning-roadmap/progress/v1", generation::roadmap()),
        ("learning-roadmap/topic/v1", generation::topic()),
    ] {
        assert!(
            prompt.contains(id),
            "no prompt names `{id}`, so the model has nothing to write into `schema`"
        );
    }
}

#[test]
fn each_prompt_names_every_field_the_step_writes() {
    for (kind, prompt) in written() {
        for path in described_fields(&schema(kind)) {
            assert!(
                prompt.contains(&format!("`{path}`")),
                "{kind}.schema.json: the {kind} prompt never names `{path}`, \
                 and a field nobody asked for is a field the model leaves out"
            );
        }
    }
}

#[test]
fn splitting_the_prompt_lost_no_field_of_either_schema() {
    let both = format!("{}{}", generation::roadmap(), generation::topic());
    for kind in ["roadmap", "topic"] {
        for path in described_fields(&schema(kind)) {
            assert!(
                both.contains(&format!("`{path}`")),
                "{kind}.schema.json: `{path}` is named by neither prompt, \
                 so the split dropped a field the bundle needs"
            );
        }
    }
}

#[test]
fn each_prompt_shows_every_value_the_step_fixes() {
    for (kind, prompt) in written() {
        for (path, values) in fixed_values(&schema(kind)) {
            for value in values {
                assert!(
                    prompt.contains(&value),
                    "{kind}.schema.json: `{path}` is fixed to `{value}`, \
                     and the {kind} prompt never shows that value"
                );
            }
        }
    }
}

#[test]
fn the_self_checks_together_cover_every_integrity_rule() {
    let both = format!("{}{}", generation::roadmap(), generation::topic());
    let codes = integrity_codes();
    assert!(
        codes.len() >= 11,
        "only {} integrity codes found in the validator, \
         the reader that feeds this gate is broken",
        codes.len()
    );
    for code in codes {
        assert!(
            both.contains(&code),
            "neither self-check mentions `{code}`, \
             so the model can hand out a bundle `validate` will reject"
        );
    }
}

#[test]
fn each_self_check_is_a_numbered_procedure_that_gates_the_output() {
    for (kind, prompt) in written() {
        let procedure = procedure(prompt);
        let steps = procedure.lines().filter(|line| numbered(line)).count();
        assert!(
            steps >= 8,
            "the {kind} self-check has {steps} numbered steps, \
             too few for the rules it has to cover"
        );
        assert!(
            procedure.to_lowercase().contains("не выводи"),
            "the {kind} self-check never stops output that failed a step, \
             which makes the procedure advice rather than a gate"
        );
    }
    assert!(
        procedure(generation::roadmap()).contains("tolearn validate"),
        "the roadmap self-check never ends in `tolearn validate`, \
         so nothing checks the model against the machine"
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
