use tolearn_core::program::Tree;
use tolearn_core::state::State;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::reading::{LibraryIn, LibraryOut, RefusedView, ShelfView};
use crate::ipc::shelf;

pub fn run(context: &Context, _input: &LibraryIn) -> Result<LibraryOut, IpcError> {
    let mut programs = Vec::new();
    let mut refused = Vec::new();
    for entry in context.library().list()? {
        match entry.program {
            Ok(tree) => programs.push(card(context, &tree)),
            Err(refusal) => refused.push(RefusedView {
                directory: entry.directory,
                code: refusal.code().to_owned(),
                message: refusal.to_string(),
            }),
        }
    }
    programs.sort_by_cached_key(|shelf| shelf.title.to_lowercase());
    Ok(LibraryOut { programs, refused })
}

fn card(context: &Context, tree: &Tree) -> ShelfView {
    let state = State::read(context.data(), &tree.program.uuid);
    ShelfView {
        uuid: tree.program.uuid.clone(),
        title: tree.program.title.clone(),
        goal: tree.program.goal.clone(),
        hours: shelf::span(tree.program.map.hours()),
        stages: rows(tree.program.map.stages.len()),
        subprograms: rows(tree.program.map.children.len()),
        summary: state.as_ref().ok().map(|state| shelf::summary(tree, state)),
        active: state.as_ref().ok().and_then(State::active),
        unread: state.err().map(|error| error.to_string()),
    }
}

fn rows(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}
