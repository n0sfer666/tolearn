use std::path::Path;

use serde_json::{Value, json};
use tolearn_core::bundle::{Violation, validate};

use crate::bundle;
use crate::error::CliError;
use crate::out::Output;

pub fn run(root: &Path) -> Result<Output, CliError> {
    let scan = bundle::read(root)?;
    let violations = validate(&scan.roadmap, &scan.topics);
    let ok = violations.is_empty();
    Ok(Output::verdict(text(&violations), body(&violations), ok))
}

fn text(violations: &[Violation]) -> String {
    if violations.is_empty() {
        return "нарушений нет".to_owned();
    }
    let mut out = format!("нарушений: {}\n", violations.len());
    for violation in violations {
        out.push_str(&format!("  {} — {violation}\n", violation.code()));
    }
    out
}

fn body(violations: &[Violation]) -> Value {
    let listed: Vec<Value> = violations
        .iter()
        .map(|violation| {
            json!({
                "code": violation.code(),
                "message": violation.to_string(),
            })
        })
        .collect();
    json!({ "ok": violations.is_empty(), "violations": listed })
}
