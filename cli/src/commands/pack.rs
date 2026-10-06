use std::path::Path;

use serde_json::json;
use tolearn_core::atomic;
use tolearn_core::export::inside;
use tolearn_core::package::pack;

use crate::error::CliError;
use crate::out::Output;

pub fn run(program: &Path, file: &Path) -> Result<Output, CliError> {
    if inside(program, file) {
        return Err(CliError::Usage(
            "пакет в каталог программы не пишется: туда пишут только генерация и импорт".to_owned(),
        ));
    }
    let package =
        pack(program).map_err(|error| CliError::Package(format!("{} — {error}", error.code())))?;
    written(file, &package)?;
    Ok(Output::new(
        format!("упаковано в {}", file.display()),
        json!({ "path": file.display().to_string(), "bytes": package.len() }),
    ))
}

fn written(file: &Path, data: &[u8]) -> Result<(), CliError> {
    let failed = |error: std::io::Error| {
        CliError::Package(format!("`{}` не записан: {error}", file.display()))
    };
    if let Some(parent) = file
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(failed)?;
    }
    atomic::bytes(file, data).map_err(failed)
}
