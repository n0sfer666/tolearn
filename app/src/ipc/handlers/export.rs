use std::path::Path;

use tolearn_core::Date;
use tolearn_core::export::{inside, markdown};
use tolearn_core::status::effective;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{ExportIn, ExportOut};

pub fn run(context: &Context, input: &ExportIn) -> Result<ExportOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let path = Path::new(&input.path);
    if inside(Path::new(&input.bundle), path) {
        return Err(IpcError::new(
            "export.inside-bundle",
            "экспорт в каталог бандла запрещён: приложение пишет туда только прогресс".to_owned(),
        ));
    }
    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    let store = crate::ipc::vaulted::store(context, input.directory.as_ref())?;
    let kept = store.index()?;
    let text = markdown(&opened.scan.roadmap, &opened.scan.topics, &statuses, &kept);
    written(path, &text)?;

    Ok(ExportOut {
        path: path.display().to_string(),
        bytes: text.len() as u64,
        plaintext: store.locked(),
    })
}

fn written(path: &Path, text: &str) -> Result<(), IpcError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| unwritable(path, &error))?;
    }
    std::fs::write(path, text).map_err(|error| unwritable(path, &error))
}

fn unwritable(path: &Path, error: &std::io::Error) -> IpcError {
    IpcError::new(
        "export.unwritable",
        format!("не удалось записать `{}`: {error}", path.display()),
    )
}
