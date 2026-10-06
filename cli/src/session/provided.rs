use std::path::Path;

use tolearn_provider::{Kind, Provider, Vault};

use super::speaker::Speaker;
use crate::error::CliError;

const PROVIDER: &str = "provider.yaml";

pub fn provided(
    config: &Path,
    named: Option<&Path>,
    vault: &dyn Vault,
) -> Result<Speaker, CliError> {
    let path = match named {
        Some(path) if !path.is_file() => {
            return Err(CliError::Setup(format!(
                "файла провайдера `{}` нет",
                path.display()
            )));
        }
        Some(path) => path.to_path_buf(),
        None => config.join(PROVIDER),
    };
    let provider = Provider::read(&path)
        .map_err(|error| CliError::Setup(format!("`{}`: {error}", path.display())))?;
    if !provider.enabled {
        return Err(CliError::Setup(match named {
            Some(_) => format!(
                "в `{}` провайдер выключен (`enabled: false`)",
                path.display()
            ),
            None => format!(
                "провайдер выключен в `{}`: включи его в настройках приложения или назови файл через `--provider`",
                path.display()
            ),
        }));
    }
    let key = match provider.active {
        Kind::Remote => vault
            .key()
            .map_err(|error| CliError::Setup(format!("ключ провайдера не прочитан: {error}")))?,
        Kind::Local | Kind::Harness => None,
    };
    Ok(Speaker::new(provider, key))
}
