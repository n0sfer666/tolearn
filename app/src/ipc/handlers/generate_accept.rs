use std::path::Path;

use crate::generate::{forget, take, unpin};
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::handlers::generate_state::unknown;
use crate::ipc::handlers::import;
use crate::ipc::types::{GenerateAcceptIn, ImportIn, ImportOut};

pub fn run(context: &Context, input: &GenerateAcceptIn) -> Result<ImportOut, IpcError> {
    let made = take(&input.job).ok_or_else(|| unknown(&input.job))?;
    let home = context.unpacked().join(&made.id);
    if home.exists() {
        return Err(busy(&home, &made.id));
    }
    made.write(&home)
        .map_err(|error| IpcError::unwritable(&home, &error.to_string()))?;

    let imported = import::run(
        context,
        &ImportIn {
            path: home.display().to_string(),
            today: input.today.clone(),
        },
    );
    match &imported {
        Ok(out) if out.ok => {
            forget(&input.job);
            unpin(&context.draft());
        }
        _ => {
            let _ = std::fs::remove_dir_all(&home);
        }
    }
    imported
}

fn busy(home: &Path, id: &str) -> IpcError {
    IpcError::new(
        "generate.already-there",
        format!("программа `{id}` уже лежит в `{}`", home.display()),
    )
}
