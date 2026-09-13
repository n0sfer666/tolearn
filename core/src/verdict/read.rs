use std::collections::BTreeSet;

use serde_json::Value;

use super::block::last;
use super::error::VerdictError;
use crate::stage::Stage;
use crate::state::{Answered, Grade};

pub fn read(text: &str, stage: &Stage) -> Result<Vec<Answered>, VerdictError> {
    let verdict = last(text).ok_or(VerdictError::Absent)?;
    let Some(Value::String(found)) = verdict.get("stage") else {
        return Err(shape("нет поля `stage` с id этапа".to_owned()));
    };
    if *found != stage.id {
        return Err(VerdictError::Stage {
            expected: stage.id.clone(),
            found: found.clone(),
        });
    }
    let Some(Value::Array(rows)) = verdict.get("per_question") else {
        return Err(shape("нет списка `per_question`".to_owned()));
    };
    let rows = rows.iter().map(answered).collect::<Result<Vec<_>, _>>()?;
    covered(&rows, stage)?;
    Ok(rows)
}

fn answered(row: &Value) -> Result<Answered, VerdictError> {
    let Value::Object(row) = row else {
        return Err(shape("в `per_question` не объект".to_owned()));
    };
    let Some(Value::String(id)) = row.get("id") else {
        return Err(shape("в `per_question` вопрос без `id`".to_owned()));
    };
    let result = row
        .get("result")
        .and_then(Value::as_str)
        .and_then(Grade::named)
        .ok_or_else(|| shape(format!("у `{id}` `result` не ok, partial или miss")))?;
    let missed = missed(id, row.get("missed"))?;
    if result != Grade::Ok && missed.is_empty() {
        return Err(shape(format!(
            "у незачтённого `{id}` не сказано, что упущено"
        )));
    }
    Ok(Answered {
        id: id.clone(),
        result,
        missed,
    })
}

fn missed(id: &str, field: Option<&Value>) -> Result<Vec<String>, VerdictError> {
    let Some(field) = field else {
        return Ok(Vec::new());
    };
    let broken = || shape(format!("у `{id}` `missed` не список строк"));
    let Value::Array(lines) = field else {
        return Err(broken());
    };
    lines
        .iter()
        .map(|line| match line {
            Value::String(line) if !line.is_empty() => Ok(line.clone()),
            _ => Err(broken()),
        })
        .collect()
}

fn covered(rows: &[Answered], stage: &Stage) -> Result<(), VerdictError> {
    let asked: BTreeSet<&str> = stage.questions.iter().map(|q| q.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    let mut stray = Vec::new();
    let mut repeated = Vec::new();
    for row in rows {
        if !asked.contains(row.id.as_str()) {
            stray.push(row.id.clone());
        } else if !seen.insert(row.id.as_str()) {
            repeated.push(row.id.clone());
        }
    }
    let missing: Vec<String> = stage
        .questions
        .iter()
        .filter(|question| !seen.contains(question.id.as_str()))
        .map(|question| question.id.clone())
        .collect();
    if missing.is_empty() && stray.is_empty() && repeated.is_empty() {
        return Ok(());
    }
    Err(VerdictError::Questions {
        missing,
        stray,
        repeated,
    })
}

fn shape(reason: String) -> VerdictError {
    VerdictError::Shape(reason)
}
