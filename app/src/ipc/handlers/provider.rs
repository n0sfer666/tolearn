use tolearn_provider::{Provider, check};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::provider::{denied, failed, refute, taken, view};
use crate::ipc::types::{CheckedView, ProviderIn, ProviderOut};

pub fn run(context: &Context, input: &ProviderIn) -> Result<ProviderOut, IpcError> {
    let mut provider = Provider::read(&context.provider()).map_err(failed)?;
    if let Some(asked) = input.save.as_ref() {
        provider = taken(asked)?;
        provider.save(&context.provider()).map_err(failed)?;
    }
    if input.forget {
        context.vault().forget().map_err(denied)?;
    }
    if let Some(given) = given(input) {
        context.vault().store(given).map_err(denied)?;
    }

    let key = context.vault().key().map_err(denied)?;
    let checked = match input.check {
        true => Some(CheckedView {
            models: check(&provider, key.as_deref()).map_err(refute)?.models,
        }),
        false => None,
    };
    Ok(ProviderOut {
        provider: view(&provider),
        has_key: key.is_some(),
        checked,
    })
}

fn given(input: &ProviderIn) -> Option<&str> {
    input
        .key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
}
