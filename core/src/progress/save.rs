use std::fs::{File, rename};
use std::io::{Error, ErrorKind, Write};
use std::path::Path;

use super::document::Document;

pub fn save(path: &Path, document: &Document) -> Result<(), Error> {
    let name = path
        .file_name()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "the path names no file"))?;
    let directory = path.parent().unwrap_or(Path::new("."));
    let temporary = directory.join(format!(
        ".{}.{}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));

    let written = write(&temporary, document.text());
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
        return written;
    }
    rename(&temporary, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temporary);
    })
}

fn write(temporary: &Path, text: &str) -> Result<(), Error> {
    let mut file = File::create(temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()
}
