use std::path::{Path, PathBuf};

use tolearn_core::progress::{Document, Format};
use tolearn_core::scan::{Scan, scan};

use super::error::IpcError;

#[derive(Debug)]
pub struct Opened {
    pub scan: Scan,
    pub document: Document,
}

pub fn read(root: &str) -> Result<Scan, IpcError> {
    scan(Path::new(root)).map_err(IpcError::from)
}

pub fn open(root: &str) -> Result<Opened, IpcError> {
    let scan = read(root)?;
    let file = progress_file(&scan);
    let document = Document::read(&text(&file)?, scan.format)?;
    Ok(Opened { scan, document })
}

pub fn progress_file(scan: &Scan) -> PathBuf {
    scan.root.join(match scan.format {
        Format::Json => "progress.json",
        Format::Yaml => "progress.yaml",
    })
}

pub fn text(path: &Path) -> Result<String, IpcError> {
    std::fs::read_to_string(path)
        .map_err(|error| IpcError::unreadable(&path.display().to_string(), &error.to_string()))
}
