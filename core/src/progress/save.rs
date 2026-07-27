use std::io::Error;
use std::path::Path;

use super::document::Document;
use crate::atomic;

pub fn save(path: &Path, document: &Document) -> Result<(), Error> {
    atomic::write(path, document.text())
}
