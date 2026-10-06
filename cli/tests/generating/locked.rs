use tolearn_provider::{Vault, VaultError};

#[derive(Debug)]
pub struct Locked;

impl Vault for Locked {
    fn store(&self, _key: &str) -> Result<(), VaultError> {
        Err(VaultError::new("доступ запрещён"))
    }

    fn key(&self) -> Result<Option<String>, VaultError> {
        Err(VaultError::new("доступ запрещён"))
    }

    fn forget(&self) -> Result<(), VaultError> {
        Err(VaultError::new("доступ запрещён"))
    }
}
