use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};
use tolearn_core::topic::{Check, Topic};
use tolearn_runner::{Limits, Outcome, run};

const LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    silence: None,
    output_bytes: 64 * 1024,
};

pub fn of(topic: &Topic) -> Vec<&Check> {
    topic
        .practice
        .constraints
        .iter()
        .chain(topic.practice.acceptance.iter())
        .collect()
}

pub fn ran(topic: &Topic, root: &Path) -> Vec<Value> {
    of(topic)
        .into_iter()
        .map(|check| one(check, root))
        .collect()
}

fn one(check: &Check, root: &Path) -> Value {
    match run(&check.check, root, LIMITS) {
        Ok(finished) => json!({
            "id": check.id,
            "expect": check.expect,
            "passed": finished.outcome == Outcome::Finished { code: Some(0) },
            "timed_out": matches!(finished.outcome, Outcome::TimedOut | Outcome::WentQuiet),
            "stdout": finished.stdout,
            "stderr": finished.stderr,
            "truncated": finished.truncated,
        }),
        Err(error) => json!({
            "id": check.id,
            "expect": check.expect,
            "passed": false,
            "error": error.to_string(),
        }),
    }
}

pub fn text(ran: &[Value]) -> String {
    let mut out = String::new();
    for check in ran {
        let mark = if check["passed"] == true {
            "ok"
        } else {
            "нет"
        };
        out.push_str(&format!(
            "  [{mark}] {}\n",
            check["id"].as_str().unwrap_or("")
        ));
    }
    out
}
