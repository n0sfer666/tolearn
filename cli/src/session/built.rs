use std::path::Path;
use std::time::Duration;

use serde_json::json;
use tolearn_core::library::PROGRAMS;

use super::shown::spent;
use super::spent::Spent;
use crate::out::Output;

#[derive(Debug)]
pub struct Built<'a> {
    pub data: &'a Path,
    pub program: &'a str,
    pub node: &'a str,
    pub stage: &'a str,
    pub title: &'a str,
    pub elapsed: Duration,
    pub spent: Spent,
    pub mermaid: usize,
}

impl Built<'_> {
    pub fn shown(&self) -> Output {
        let path = self.data.join(PROGRAMS).join(self.program);
        let text = format!(
            "программа: {}\nэтап: {} · {}\nдальше: tolearn next {}",
            path.display(),
            self.stage,
            spent(self.elapsed, self.spent).join(" · "),
            self.data.display()
        );
        let json = json!({
            "program": self.program,
            "path": path.display().to_string(),
            "node": self.node,
            "stage": self.stage,
            "title": self.title,
            "ms": u64::try_from(self.elapsed.as_millis()).unwrap_or(u64::MAX),
            "calls": self.spent.calls,
            "input": self.spent.input,
            "output": self.spent.output,
            "unknown": self.spent.unknown,
            "mermaid": self.mermaid,
        });
        Output::new(text, json)
    }
}
