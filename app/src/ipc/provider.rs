use tolearn_provider::{CheckError, Flavor, Provider, ProviderError, VaultError};

use crate::ipc::error::IpcError;
use crate::ipc::types::ProviderView;

pub fn view(provider: &Provider) -> ProviderView {
    ProviderView {
        enabled: provider.enabled,
        flavor: provider.flavor.label().to_owned(),
        endpoint: provider.endpoint.clone(),
        model: provider.model.clone(),
    }
}

pub fn taken(view: &ProviderView) -> Result<Provider, IpcError> {
    let endpoint = view.endpoint.trim();
    if endpoint.is_empty() {
        return Err(refused("адрес", &view.endpoint));
    }
    Ok(Provider {
        enabled: view.enabled,
        flavor: Flavor::parse(&view.flavor).ok_or_else(|| refused("вид", &view.flavor))?,
        endpoint: endpoint.to_owned(),
        model: view.model.trim().to_owned(),
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

fn refused(what: &str, value: &str) -> IpcError {
    IpcError::new(
        "provider.unknown-value",
        format!("`{value}` — не значение для настройки «{what}»"),
    )
}
