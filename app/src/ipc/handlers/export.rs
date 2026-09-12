use std::path::Path;

use tolearn_core::export::{ExportError, render, write};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::reading::{ExportIn, ExportOut};
use crate::ipc::shelf;

pub fn run(context: &Context, input: &ExportIn) -> Result<ExportOut, IpcError> {
    let folder = Path::new(&input.folder);
    if !folder.is_absolute() {
        return Err(IpcError::new(
            "export.folder",
            format!("`{}` — не абсолютный путь к папке", input.folder),
        ));
    }
    let library = context.library();
    let tree = library.open(&input.program)?;
    let branch = shelf::branch(&tree, &input.node)?;
    let target = folder.join(&branch.tree.program.slug);
    if library.holds(&target) {
        return Err(IpcError::new(
            "export.inside-library",
            format!(
                "`{}` лежит в библиотеке: туда пишут только генерация и импорт",
                target.display()
            ),
        ));
    }
    let files = write(&render(branch.tree), &target, |file| {
        library
            .asset(&tree, &format!("{}{file}", branch.prefix))
            .map_err(|error| ExportError::Asset {
                file: file.to_owned(),
                reason: error.to_string(),
            })
    })?;
    Ok(ExportOut {
        path: target.display().to_string(),
        files: files as u64,
    })
}
