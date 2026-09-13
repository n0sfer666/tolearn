mod choices;
mod error;
mod key;
mod parse;
mod render;
mod status;
mod summary;
mod tick;
mod types;

pub use choices::{Grade, Pass, Sitting};
pub use error::StateError;
pub use key::key;
pub use parse::parse;
pub use render::{SCHEMA, render};
pub use status::Status;
pub use summary::Summary;
pub use types::{Answered, Attempt, Clarification, Passed, StageState, State, Turn};

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::atomic;
use crate::yaml::is_uuid;

pub const STATE: &str = "state";

const FILE: &str = "state.yaml";

static WRITES: Mutex<()> = Mutex::new(());

impl State {
    pub fn read(data: &Path, program: &str) -> Result<Self, StateError> {
        load(&file(data, program)?, program)
    }

    pub fn update<T>(
        data: &Path,
        program: &str,
        change: impl FnOnce(&mut Self) -> T,
    ) -> Result<T, StateError> {
        let path = file(data, program)?;
        let _writing = WRITES.lock().unwrap_or_else(PoisonError::into_inner);
        let mut state = load(&path, program)?;
        let before = state.clone();
        let outcome = change(&mut state);
        if state == before {
            return Ok(outcome);
        }
        let text =
            render(&state).map_err(|error| StateError::Unwritable(io::Error::other(error)))?;
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory).map_err(StateError::Unwritable)?;
        }
        atomic::write(&path, &text).map_err(StateError::Unwritable)?;
        Ok(outcome)
    }
}

fn file(data: &Path, program: &str) -> Result<PathBuf, StateError> {
    if !is_uuid(program) {
        return Err(StateError::Stray(program.to_owned()));
    }
    Ok(data.join(STATE).join(program).join(FILE))
}

fn load(path: &Path, program: &str) -> Result<State, StateError> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(State::new(program)),
        Err(error) => return Err(StateError::Unreadable(error)),
    };
    let state = parse(&source).map_err(StateError::Malformed)?;
    if state.program() != program {
        return Err(StateError::Foreign(state.program().to_owned()));
    }
    Ok(state)
}
