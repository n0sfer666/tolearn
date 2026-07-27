use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tolearn_core::progress::save;
use tolearn_core::prompt::render;
use tolearn_core::protocol::{Applied, apply, record};
use tolearn_core::topic::Topic;
use tolearn_core::verdict::parse;

use crate::args::Command;
use crate::bundle::{self, Opened};
use crate::commands::checks;
use crate::error::CliError;
use crate::out::Output;
use crate::today;

pub fn run(root: &Path, command: &Command) -> Result<Output, CliError> {
    let Command::Exam {
        topic,
        verdict,
        template,
        run_checks,
    } = command
    else {
        return Err(CliError::Usage("это не `exam`".to_owned()));
    };

    let mut opened = bundle::open(root)?;
    let found = wanted(&opened, topic)?.clone();
    let prompt = render(
        &text(root, template.as_deref())?,
        &found,
        &opened.scan.roadmap,
    )
    .map_err(|error| CliError::Bundle(error.to_string()))?;

    let applied = match verdict {
        Some(path) => Some(applied(&mut opened, &found, path)?),
        None => None,
    };
    let ran = run_checks.then(|| checks::ran(&found, root));

    Ok(Output::new(
        words(&prompt, applied.as_ref(), ran.as_deref()),
        body(&prompt, applied.as_ref(), ran),
    ))
}

fn wanted<'a>(opened: &'a Opened, topic: &str) -> Result<&'a Topic, CliError> {
    opened
        .scan
        .topics
        .iter()
        .find(|found| found.id == topic)
        .ok_or_else(|| CliError::UnknownTopic(topic.to_owned()))
}

fn text(root: &Path, template: Option<&Path>) -> Result<String, CliError> {
    bundle::text(&template.map_or_else(|| root.join("examiner.md"), PathBuf::from))
}

fn applied(opened: &mut Opened, topic: &Topic, path: &Path) -> Result<Applied, CliError> {
    let day = today::today().ok_or_else(|| CliError::Bundle("часы системы врут".to_owned()))?;
    let source = bundle::text(path)?;
    let verdict = parse(&source, topic).map_err(|error| CliError::Verdict(error.to_string()))?;
    let state = opened.document.progress().state(&topic.id).cloned();
    let at = today::moment().ok_or_else(|| CliError::Bundle("часы системы врут".to_owned()))?;
    let applied = apply(&verdict, topic, state.as_ref(), &at, day);
    record(&mut opened.document, &topic.id, &applied)
        .map_err(|error| CliError::Write(error.to_string()))?;
    save(&opened.progress_file, &opened.document)
        .map_err(|error| CliError::Write(error.to_string()))?;
    Ok(applied)
}

fn words(prompt: &str, applied: Option<&Applied>, ran: Option<&[Value]>) -> String {
    let mut out = prompt.to_owned();
    if let Some(applied) = applied {
        out = format!(
            "статус: {}\nна повтор: {}\n",
            applied.state.status.label(),
            applied.retry.join(", ")
        );
        if applied.split_suggested {
            out.push_str("три провала подряд — тему стоит разрезать\n");
        }
    }
    if let Some(ran) = ran {
        out.push_str("check-команды:\n");
        out.push_str(&checks::text(ran));
    }
    out
}

fn body(prompt: &str, applied: Option<&Applied>, ran: Option<Vec<Value>>) -> Value {
    json!({
        "prompt": prompt,
        "applied": applied.map(|applied| json!({
            "status": applied.state.status.label(),
            "retry": applied.retry,
            "split_suggested": applied.split_suggested,
            "gaps": applied.state.gaps,
            "passed_at": applied.state.passed_at,
            "next_review_at": applied.state.next_review_at,
        })),
        "checks": ran,
    })
}
