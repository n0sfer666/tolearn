use std::fmt;
use std::sync::{Mutex, PoisonError};

use crate::error::VaultError;

pub trait Vault: fmt::Debug + Send + Sync {
    fn store(&self, key: &str) -> Result<(), VaultError>;

    fn key(&self) -> Result<Option<String>, VaultError>;

    fn forget(&self) -> Result<(), VaultError>;
}

#[derive(Debug)]
pub struct Keychain {
    service: String,
    account: String,
}

impl Keychain {
    pub fn new(service: &str, account: &str) -> Self {
        Self {
            service: service.to_owned(),
            account: account.to_owned(),
        }
    }

    fn entry(&self) -> Result<keyring::Entry, VaultError> {
        keyring::Entry::new(&self.service, &self.account).map_err(VaultError::new)
    }
}

impl Vault for Keychain {
    fn store(&self, key: &str) -> Result<(), VaultError> {
        self.entry()?.set_password(key).map_err(VaultError::new)
    }

    fn key(&self) -> Result<Option<String>, VaultError> {
        match self.entry()?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(VaultError::new(error)),
        }
    }

    fn forget(&self) -> Result<(), VaultError> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(VaultError::new(error)),
        }
    }
}

#[derive(Debug, Default)]
pub struct Remembered {
    key: Mutex<Option<String>>,
}

impl Remembered {
    fn slot(&self) -> std::sync::MutexGuard<'_, Option<String>> {
        self.key.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Vault for Remembered {
    fn store(&self, key: &str) -> Result<(), VaultError> {
        *self.slot() = Some(key.to_owned());
        Ok(())
    }

    fn key(&self) -> Result<Option<String>, VaultError> {
        Ok(self.slot().clone())
    }

    fn forget(&self) -> Result<(), VaultError> {
        *self.slot() = None;
        Ok(())
    }
}
