use std::str::FromStr;

use age::secrecy::ExposeSecret;
use age::x25519::Identity;

use super::crypt::{opened_by_phrase, sealed_by_phrase};
use super::error::SealError;

#[derive(Clone)]
pub struct Keys {
    identity: Identity,
}

impl std::fmt::Debug for Keys {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("Keys(<ключ устройства>)")
    }
}

impl Keys {
    pub fn generate() -> Self {
        Self {
            identity: Identity::generate(),
        }
    }

    pub fn parse(secret: &str) -> Result<Self, SealError> {
        Identity::from_str(secret.trim())
            .map(|identity| Self { identity })
            .map_err(|_| SealError::MalformedKey)
    }

    pub fn secret(&self) -> String {
        self.identity.to_string().expose_secret().to_owned()
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    pub fn wrapped(&self, phrase: &str) -> Result<Vec<u8>, SealError> {
        sealed_by_phrase(self.secret().as_bytes(), phrase)
    }

    pub fn unwrapped(cipher: &[u8], phrase: &str) -> Result<Self, SealError> {
        let plain = opened_by_phrase(cipher, phrase)?;
        let secret = String::from_utf8(plain).map_err(|_| SealError::MalformedKey)?;
        Self::parse(&secret)
    }
}
