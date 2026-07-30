use std::path::PathBuf;

use tolearn_offline::reader;
use tolearn_offline::store::Held;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{ReadOfflineIn, ReadOfflineOut};
use crate::ipc::{open, settings};
use crate::offline;

pub fn run(context: &Context, input: &ReadOfflineIn) -> Result<ReadOfflineOut, IpcError> {
    let settings = settings::stored(context)?;
    let held = offline::opened(&context.offline(), settings.budget_bytes(), &input.url)
        .ok_or_else(|| absent(&input.url))?;
    let path = held.path.display().to_string();

    let Some(file) = entry(&held) else {
        return Ok(ReadOfflineOut {
            kind: held.kind,
            title: String::new(),
            html: String::new(),
            text: String::new(),
            path,
            extracted: false,
        });
    };

    let archive = open::text(&file)?;
    let reading = reader::read(&archive, &input.url);
    Ok(ReadOfflineOut {
        kind: held.kind,
        title: reading.title.unwrap_or_default(),
        html: reading.html,
        text: reading.text,
        path,
        extracted: reading.extracted,
    })
}

fn entry(held: &Held) -> Option<PathBuf> {
    match held.kind.as_str() {
        "archive" => Some(held.path.clone()),
        "mirror" => Some(held.path.join("index.html")),
        _ => None,
    }
}

fn absent(url: &str) -> IpcError {
    IpcError::new(
        "offline.absent",
        format!("`{url}` не сохранён — открывать нечего"),
    )
}
