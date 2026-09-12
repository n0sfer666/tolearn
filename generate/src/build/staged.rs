use std::fs;
use std::path::Path;

use tolearn_core::program::{self, Program};
use tolearn_core::stage::{self, Stage};

use crate::error::GenerateError;

use super::assemble::Assets;

const STAGES: &str = "stages";
const ASSETS: &str = "assets";
const PROGRAM: &str = "program.yaml";

pub(crate) fn staged(
    folder: &Path,
    program: &Program,
    stage: &Stage,
    assets: &Assets,
) -> Result<(), GenerateError> {
    described(folder, program)?;
    let name = format!("{}.yaml", stage.id);
    let text = stage::write(stage).map_err(|error| unwritten(&name, &error))?;
    put(&folder.join(STAGES).join(name), text.as_bytes())?;
    for (file, bytes) in assets {
        put(&folder.join(ASSETS).join(file), bytes)?;
    }
    Ok(())
}

pub(crate) fn described(folder: &Path, program: &Program) -> Result<(), GenerateError> {
    let text = program::write(program).map_err(|error| unwritten(PROGRAM, &error))?;
    put(&folder.join(PROGRAM), text.as_bytes())
}

fn put(path: &Path, bytes: &[u8]) -> Result<(), GenerateError> {
    let folder = path.parent().unwrap_or(path);
    fs::create_dir_all(folder)
        .and_then(|()| fs::write(path, bytes))
        .map_err(|error| unwritten(&path.display().to_string(), &error))
}

fn unwritten(what: &str, error: &dyn std::fmt::Display) -> GenerateError {
    GenerateError::Unwritten(format!("{what}: {error}"))
}
