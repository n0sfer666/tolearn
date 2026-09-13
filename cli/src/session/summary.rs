use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};
use tolearn_generate::ledger::{Record, Total};

use super::shown::grouped;
use crate::out::Output;

const HEAD: &str = "| этап | шаг | вызовы | проверки | время, с | вход | выход |\n|---|---|---:|---:|---:|---:|---:|";
const UNSTAGED: &str = "—";

#[derive(Debug)]
pub struct Summary {
    groups: Vec<Group>,
    total: Total,
}

#[derive(Debug)]
struct Group {
    stage: Option<String>,
    steps: Vec<(String, Total)>,
    total: Total,
}

impl Summary {
    pub fn of(records: &[Record]) -> Self {
        let groups = distinct(records.iter().map(placed))
            .into_iter()
            .map(|(program, stage)| {
                let within: Vec<&Record> = records
                    .iter()
                    .filter(|record| placed(record) == (program, stage))
                    .collect();
                let steps = distinct(within.iter().map(|record| record.step.as_str()))
                    .into_iter()
                    .map(|step| {
                        let total =
                            Total::of(within.iter().copied().filter(|record| record.step == step));
                        (step.to_owned(), total)
                    })
                    .collect();
                Group {
                    stage: stage.map(str::to_owned),
                    steps,
                    total: Total::of(within.iter().copied()),
                }
            })
            .collect();
        Self {
            groups,
            total: Total::of(records),
        }
    }

    pub fn shown(&self, path: &Path, program: &str) -> Output {
        Output::new(self.text(path), self.json(path, program))
    }

    fn text(&self, path: &Path) -> String {
        if self.groups.is_empty() {
            return format!("журнал пуст: {}", path.display());
        }
        let mut lines = vec![format!("журнал: {}", path.display()), String::new()];
        lines.push(HEAD.to_owned());
        for group in &self.groups {
            let stage = group.stage.as_deref().unwrap_or(UNSTAGED);
            for (step, total) in &group.steps {
                lines.push(line(stage, step, total));
            }
            lines.push(line(stage, "итого", &group.total));
        }
        lines.push(line("всё", "итого", &self.total));
        lines.push(String::new());
        lines.push(format!(
            "токенов всего: {}",
            grouped(self.total.input + self.total.output)
        ));
        if self.total.unknown > 0 {
            lines.push(format!("без данных о токенах: {}", self.total.unknown));
        }
        lines.join("\n")
    }

    fn json(&self, path: &Path, program: &str) -> Value {
        let rows: Vec<Value> = self
            .groups
            .iter()
            .flat_map(|group| {
                group.steps.iter().map(|(step, total)| {
                    let mut row = counted(total);
                    row["stage"] = json!(group.stage);
                    row["step"] = json!(step);
                    row
                })
            })
            .collect();
        json!({
            "path": path.display().to_string(),
            "program": program,
            "rows": rows,
            "total": counted(&self.total),
        })
    }
}

fn line(stage: &str, step: &str, total: &Total) -> String {
    format!(
        "| {stage} | {step} | {} | {} | {:.1} | {} | {} |",
        total.calls,
        total.checks,
        Duration::from_millis(total.ms).as_secs_f64(),
        grouped(total.input),
        grouped(total.output)
    )
}

fn counted(total: &Total) -> Value {
    json!({
        "calls": total.calls,
        "checks": total.checks,
        "ms": total.ms,
        "input": total.input,
        "output": total.output,
        "unknown": total.unknown,
    })
}

fn placed(record: &Record) -> (Option<&str>, Option<&str>) {
    match record.stage.as_deref() {
        Some(stage) => (record.program.as_deref(), Some(stage)),
        None => (None, None),
    }
}

fn distinct<T: PartialEq>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut seen = Vec::new();
    for item in items {
        if !seen.contains(&item) {
            seen.push(item);
        }
    }
    seen
}
