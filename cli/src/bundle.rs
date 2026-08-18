use std::path::{Path, PathBuf};

use tolearn_core::progress::{Document, Format};
use tolearn_core::scan::{Scan, scan};

use crate::error::CliError;

#[derive(Debug)]
pub struct Opened {
    pub scan: Scan,
    pub document: Document,
    pub progress_file: PathBuf,
}

pub fn read(root: &Path) -> Result<Scan, CliError> {
    scan(root).map_err(|error| CliError::Bundle(error.to_string()))
}

pub fn open(root: &Path) -> Result<Opened, CliError> {
    let scan = read(root)?;
    let progress_file = root.join(match scan.format {
        Format::Json => "progress.json",
        Format::Yaml => "progress.yaml",
    });
    let source = text(&progress_file)?;
    let document = Document::read(&source, scan.format)
        .map_err(|error| CliError::Bundle(error.to_string()))?;
    Ok(Opened {
        scan,
        document,
        progress_file,
    })
}

pub fn text(path: &Path) -> Result<String, CliError> {
    std::fs::read_to_string(path).map_err(|error| CliError::Unreadable {
        path: path.display().to_string(),
        reason: error.to_string(),
    })
}
