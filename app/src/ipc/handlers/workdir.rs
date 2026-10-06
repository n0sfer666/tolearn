use std::path::Path;

use tolearn_core::state::State;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::practiced::{WorkdirIn, WorkdirOut};

pub fn run(context: &Context, input: &WorkdirIn) -> Result<WorkdirOut, IpcError> {
    let tree = context.library().open(&input.program)?;
    let place = Path::new(&input.path);
    let real = place
        .canonicalize()
        .ok()
        .filter(|real| place.is_absolute() && real.is_dir())
        .ok_or_else(|| absent(&input.path))?;
    let data = context
        .data()
        .canonicalize()
        .unwrap_or_else(|_| context.data().to_path_buf());
    if real.starts_with(&data) {
        return Err(IpcError::new(
            "workdir.inside",
            format!(
                "`{}` лежит в данных приложения: практику ведут в своей папке",
                input.path
            ),
        ));
    }
    State::update(context.data(), &tree.program.uuid, |state| {
        state.workdir = Some(input.path.clone());
    })?;
    Ok(WorkdirOut {
        workdir: input.path.clone(),
    })
}

pub fn absent(path: &str) -> IpcError {
    IpcError::new(
        "workdir.absent",
        format!("папки практики `{path}` нет — выберите её заново"),
    )
}
