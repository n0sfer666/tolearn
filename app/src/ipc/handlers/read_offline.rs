use std::path::{Path, PathBuf};

use tolearn_offline::reader::{self, Kind, Piece};
use tolearn_offline::store::Held;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{Block, ReadOfflineIn, ReadOfflineOut};
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
            blocks: Vec::new(),
            path,
            extracted: false,
        });
    };

    let archive = open::text(&file)?;
    let reading = reader::read(&archive, &input.url);
    let beside = file.parent().unwrap_or(&held.path).to_path_buf();
    let blocks = if reading.extracted {
        laid(&reading.html, &beside)
    } else {
        Vec::new()
    };
    Ok(ReadOfflineOut {
        kind: held.kind,
        title: reading.title.unwrap_or_default(),
        blocks,
        html: reading.html,
        text: reading.text,
        path,
        extracted: reading.extracted,
    })
}

fn laid(html: &str, beside: &Path) -> Vec<Block> {
    reader::pieces(html)
        .into_iter()
        .filter_map(|piece| shown(piece, beside))
        .collect()
}

fn shown(piece: Piece, beside: &Path) -> Option<Block> {
    let src = match piece.kind {
        Kind::Image => Some(reader::inlined(&piece.src, beside)?),
        _ => None,
    };
    Some(Block {
        kind: piece.kind.label().to_owned(),
        level: piece.level,
        text: piece.text,
        src: src.unwrap_or_default(),
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
