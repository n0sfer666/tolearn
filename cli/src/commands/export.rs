use std::path::Path;

use serde_json::json;
use tolearn_core::Date;
use tolearn_core::export::{inside, markdown};
use tolearn_core::notes::{Note, index};
use tolearn_core::status::effective;

use crate::bundle;
use crate::error::CliError;
use crate::out::Output;
use crate::today;

pub fn run(
    root: &Path,
    out: Option<&Path>,
    notes: Option<&Path>,
    today: Option<&str>,
) -> Result<Output, CliError> {
    let opened = bundle::open(root)?;
    let day = day(today)?;
    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    let kept = kept(notes)?;
    let text = markdown(&opened.scan.roadmap, &opened.scan.topics, &statuses, &kept);
    let Some(path) = out else {
        return Ok(Output::new(text.clone(), json!({ "markdown": text })));
    };
    if inside(root, path) {
        return Err(CliError::Usage(
            "экспорт в каталог бандла запрещён: приложение пишет туда только progress".to_owned(),
        ));
    }
    written(path, &text)?;
    Ok(Output::new(
        format!("экспортировано в {}", path.display()),
        json!({ "path": path.display().to_string(), "bytes": text.len() }),
    ))
}

fn day(given: Option<&str>) -> Result<Date, CliError> {
    match given {
        Some(value) => Date::parse(value)
            .ok_or_else(|| CliError::Usage(format!("`--today {value}` — не дата"))),
        None => today::today().ok_or_else(|| CliError::Bundle("часы системы врут".to_owned())),
    }
}

fn kept(notes: Option<&Path>) -> Result<Vec<Note>, CliError> {
    let Some(root) = notes else {
        return Ok(Vec::new());
    };
    index(root).map_err(|error| CliError::Unreadable {
        path: root.display().to_string(),
        reason: error.to_string(),
    })
}

fn written(path: &Path, text: &str) -> Result<(), CliError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| CliError::Write(error.to_string()))?;
    }
    std::fs::write(path, text).map_err(|error| CliError::Write(error.to_string()))
}
