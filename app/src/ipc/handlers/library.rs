use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::reading::{LibraryIn, LibraryOut, RefusedView, ShelfView};
use crate::ipc::shelf;

pub fn run(context: &Context, _input: &LibraryIn) -> Result<LibraryOut, IpcError> {
    let mut programs = Vec::new();
    let mut refused = Vec::new();
    for entry in context.library().list()? {
        match entry.program {
            Ok(tree) => programs.push(ShelfView {
                uuid: tree.program.uuid.clone(),
                title: tree.program.title.clone(),
                goal: tree.program.goal.clone(),
                hours: shelf::span(tree.program.map.hours()),
                children: shelf::children(&tree),
            }),
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
