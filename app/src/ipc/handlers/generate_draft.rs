use tolearn_core::Date;
use tolearn_provider::{CheckError, Provider};

use crate::generate::{Waiting, carry, unpin, waiting};
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::provider::{denied, failed, refute};
use crate::ipc::types::{DraftView, GenerateDraftIn, GenerateDraftOut};

pub fn run(context: &Context, input: &GenerateDraftIn) -> Result<GenerateDraftOut, IpcError> {
    let root = context.draft();
    if input.drop {
        unpin(&root);
        return Ok(GenerateDraftOut {
            draft: None,
            job: None,
        });
    }
    if !input.take {
        return Ok(GenerateDraftOut {
            draft: waiting(&root).map(view),
            job: None,
        });
    }
    let provider = Provider::read(&context.provider()).map_err(failed)?;
    if !provider.enabled {
        return Err(refute(CheckError::Disabled));
    }
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let key = context.vault().key().map_err(denied)?;
    let job = carry(provider, key, root, day.to_string()).ok_or_else(gone)?;
    Ok(GenerateDraftOut {
        draft: None,
        job: Some(job),
    })
}

fn view(waiting: Waiting) -> DraftView {
    DraftView {
        id: waiting.id,
        title: waiting.title,
        total: u32::try_from(waiting.total).unwrap_or(u32::MAX),
        done: u32::try_from(waiting.done).unwrap_or(u32::MAX),
    }
}

fn gone() -> IpcError {
    IpcError::new(
        "generate.no-draft",
        "черновика сборки больше нет".to_owned(),
    )
}
