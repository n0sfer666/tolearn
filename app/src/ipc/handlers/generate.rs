use tolearn_core::generate::{Level, Request};
use tolearn_provider::{CheckError, Provider};

use crate::generate::start;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::provider::{denied, failed, refute};
use crate::ipc::settings;
use crate::ipc::types::{GenerateIn, GenerateOut};

pub fn run(context: &Context, input: &GenerateIn) -> Result<GenerateOut, IpcError> {
    let provider = Provider::read(&context.provider()).map_err(failed)?;
    if !provider.enabled {
        return Err(refute(CheckError::Disabled));
    }
    let level = Level::read(&input.level).ok_or_else(|| unknown(&input.level))?;
    if input.subject.trim().is_empty() {
        return Err(IpcError::new(
            "generate.no-subject",
            "не сказано, чему учиться".to_owned(),
        ));
    }
    let key = context.vault().key().map_err(denied)?;
    let request = Request {
        subject: input.subject.trim().to_owned(),
        level,
        weekly_hours: input.weekly_hours,
        weeks: input.weeks,
        locale: settings::stored(context)?.locale.label().to_owned(),
    };
    Ok(GenerateOut {
        job: start(provider, key, request),
    })
}

fn unknown(level: &str) -> IpcError {
    IpcError::new(
        "generate.unknown-level",
        format!("`{level}` — не уровень подготовки"),
    )
}
