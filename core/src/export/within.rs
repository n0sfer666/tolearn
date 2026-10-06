use std::fs;
use std::io;
use std::path::Path;

use super::ExportError;

pub fn within(home: &Path) -> impl Fn(&str) -> Result<Vec<u8>, ExportError> + use<> {
    let home = home.to_path_buf();
    move |file| {
        let refused = |reason: String| ExportError::Asset {
            file: file.to_owned(),
            reason,
        };
        let failed = |error: io::Error| refused(error.kind().to_string());
        let root = home.canonicalize().map_err(failed)?;
        let path = home.join(file).canonicalize().map_err(failed)?;
        if !path.starts_with(&root) {
            return Err(refused("it leads outside the program".to_owned()));
        }
        fs::read(&path).map_err(failed)
    }
}
