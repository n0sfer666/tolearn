use std::path::Path;

use tolearn_core::package;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::reading::{ImportPackageIn, ImportPackageOut};

const ARCHIVES: [&str; 3] = [".zip", ".gz", ".tgz"];
const V1: [&str; 2] = ["roadmap.yaml", "roadmap.json"];

pub fn run(context: &Context, input: &ImportPackageIn) -> Result<ImportPackageOut, IpcError> {
    let path = Path::new(&input.path);
    admitted(path)?;
    let library = context.library();
    let imported = package::import(path, &library)?;
    let title = library
        .open(&imported.uuid)
        .map_or_else(|_| imported.uuid.clone(), |tree| tree.program.title);
    Ok(ImportPackageOut {
        uuid: imported.uuid,
        title,
        copy_of: imported.copy_of,
    })
}

fn admitted(path: &Path) -> Result<(), IpcError> {
    if path.join("program.yaml").is_file() {
        return Err(IpcError::new(
            "package.folder",
            "это папка программы, а не пакет: соберите его командой `tolearn pack`".to_owned(),
        ));
    }
    if V1.iter().any(|marker| path.join(marker).is_file()) {
        return Err(v1());
    }
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let lowered = name.to_lowercase();
    let file = !path.is_dir();
    if file && lowered.ends_with(".tolearn") {
        return Ok(());
    }
    if file && ARCHIVES.iter().any(|suffix| lowered.ends_with(suffix)) {
        return Err(v1());
    }
    Err(IpcError::new(
        "package.foreign",
        format!("`{name}` — не пакет .tolearn"),
    ))
}

fn v1() -> IpcError {
    IpcError::new(
        "package.v1",
        "программы v1 не открываются: откройте пакет .tolearn".to_owned(),
    )
}
