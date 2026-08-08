use tolearn_provider::{PROBE_PROMPT, Provider, check, probe};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::provider::{advice, denied, failed, presets, refute, taken, view};
use crate::ipc::types::{CheckedView, ProbedView, ProviderIn, ProviderOut};
use crate::journal::Journal;

const KIND: &str = "Пробный запрос";

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
        true => Some(checked(&provider, key.as_deref())?),
        false => None,
    };
    let probed = match input.probe {
        true => Some(probed(
            &provider,
            key.as_deref(),
            &Journal::new(context.llm_log(), provider.journal),
        )?),
        false => None,
    };
    Ok(ProviderOut {
        provider: view(&provider),
        has_key: key.is_some(),
        advised: advice(checked.as_ref()),
        checked,
        probed,
        presets: presets(),
    })
}

fn checked(provider: &Provider, key: Option<&str>) -> Result<CheckedView, IpcError> {
    let checked = check(provider, key).map_err(refute)?;
    Ok(CheckedView {
        models: checked.models,
        version: checked.version,
        took_ms: checked.took_ms,
    })
}

fn probed(
    provider: &Provider,
    key: Option<&str>,
    journal: &Journal,
) -> Result<ProbedView, IpcError> {
    let probed = match probe(provider, key) {
        Ok(probed) => probed,
        Err(error) => {
            let refused = refute(error);
            journal.refused(KIND, PROBE_PROMPT, &refused.message);
            return Err(refused);
        }
    };
    journal.said(KIND, PROBE_PROMPT, &probed.said);
    Ok(ProbedView {
        said: probed.said,
        took_ms: probed.took_ms,
        thinking: probed.thinking,
    })
}

fn given(input: &ProviderIn) -> Option<&str> {
    input
        .key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
}
