use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::settings::{failed, stored, taken, view};
use crate::ipc::types::{SettingsIn, SettingsView};

pub fn run(context: &Context, input: &SettingsIn) -> Result<SettingsView, IpcError> {
    if let Some(asked) = input.save.as_ref() {
        let settings = taken(asked)?;
        settings.save(&context.settings()).map_err(failed)?;
        return Ok(view(&settings));
    }
    Ok(view(&stored(context)?))
}
