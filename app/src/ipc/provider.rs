use tolearn_provider::{
    Api, CheckError, Harness, Http, Kind, PRESETS, Provider, ProviderError, VaultError,
};

use crate::ipc::error::IpcError;
use crate::ipc::types::{HarnessView, HttpView, PresetView, ProviderView};

const TIMEOUT_MAX: u32 = 3_600;

pub fn view(provider: &Provider) -> ProviderView {
    ProviderView {
        enabled: provider.enabled,
        active: provider.active.label().to_owned(),
        local: seen(&provider.local),
        remote: seen(&provider.remote),
        harness: HarnessView {
            id: provider.harness.id.clone(),
            command: provider.harness.command.clone(),
            args: provider.harness.args.clone(),
            timeout_secs: provider.harness.timeout_secs,
        },
    }
}

pub fn presets() -> Vec<PresetView> {
    PRESETS
        .iter()
        .map(|preset| PresetView {
            id: preset.id.to_owned(),
            command: preset.command.to_owned(),
            args: preset.args.iter().map(|arg| (*arg).to_owned()).collect(),
        })
        .collect()
}

pub fn taken(view: &ProviderView) -> Result<Provider, IpcError> {
    let active = Kind::parse(&view.active).ok_or_else(|| refused("вид", &view.active))?;
    let asked = match active {
        Kind::Local => Some(&view.local),
        Kind::Remote => Some(&view.remote),
        Kind::Harness => None,
    };
    if let Some(http) = asked
        && http.endpoint.trim().is_empty()
    {
        return Err(refused("адрес", &http.endpoint));
    }
    let timeout_secs = view.harness.timeout_secs;
    if timeout_secs == 0 || timeout_secs > TIMEOUT_MAX {
        return Err(refused("таймаут", &timeout_secs.to_string()));
    }

    Ok(Provider {
        enabled: view.enabled,
        active,
        local: told(&view.local)?,
        remote: told(&view.remote)?,
        harness: Harness {
            id: view.harness.id.trim().to_owned(),
            command: view.harness.command.trim().to_owned(),
            args: view.harness.args.clone(),
            timeout_secs,
        },
    })
}

pub fn failed(error: ProviderError) -> IpcError {
    let code = match error {
        ProviderError::Unwritable(_) => "provider.unwritable",
        ProviderError::Unreadable(_) | ProviderError::Malformed(_) => "provider.unreadable",
    };
    IpcError::new(code, error.to_string())
}

pub fn denied(error: VaultError) -> IpcError {
    IpcError::new("provider.vault", error.to_string())
}

pub fn refute(error: CheckError) -> IpcError {
    IpcError::new(error.code(), error.to_string())
}

fn seen(http: &Http) -> HttpView {
    HttpView {
        endpoint: http.endpoint.clone(),
        api: http.api.label().to_owned(),
        model: http.model.clone(),
    }
}

fn told(view: &HttpView) -> Result<Http, IpcError> {
    Ok(Http {
        endpoint: view.endpoint.trim().to_owned(),
        api: Api::parse(&view.api).ok_or_else(|| refused("API", &view.api))?,
        model: view.model.trim().to_owned(),
    })
}

fn refused(what: &str, value: &str) -> IpcError {
    IpcError::new(
        "provider.unknown-value",
        format!("`{value}` — не значение для настройки «{what}»"),
    )
}
