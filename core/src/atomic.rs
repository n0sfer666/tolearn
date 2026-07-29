use std::fs::{File, rename};
use std::io::{Error, ErrorKind, Write};
use std::path::Path;

pub fn write(path: &Path, text: &str) -> Result<(), Error> {
    bytes(path, text.as_bytes())
}

pub fn bytes(path: &Path, data: &[u8]) -> Result<(), Error> {
    let name = path
        .file_name()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "the path names no file"))?;
    let directory = path.parent().unwrap_or(Path::new("."));
    let temporary = directory.join(format!(
        ".{}.{}.tmp",
        name.to_string_lossy(),
        std::process::id()
    ));

    let written = spill(&temporary, data);
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
        return written;
    }
    rename(&temporary, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temporary);
    })
}

fn spill(temporary: &Path, data: &[u8]) -> Result<(), Error> {
    let mut file = File::create(temporary)?;
    file.write_all(data)?;
    file.sync_all()
}
