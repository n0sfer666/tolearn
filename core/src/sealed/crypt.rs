use std::io::{Read, Write};
use std::iter;

use age::secrecy::SecretString;
use age::x25519::Identity;
use age::{Decryptor, Encryptor};

use super::error::SealError;

pub fn seal(plain: &[u8], identity: &Identity) -> Result<Vec<u8>, SealError> {
    let recipient = identity.to_public();
    let encryptor =
        Encryptor::with_recipients(iter::once(&recipient as &dyn age::Recipient)).map_err(broke)?;
    written(plain, encryptor)
}

pub fn sealed_by_phrase(plain: &[u8], phrase: &str) -> Result<Vec<u8>, SealError> {
    written(
        plain,
        Encryptor::with_user_passphrase(SecretString::from(phrase.to_owned())),
    )
}

pub fn open(cipher: &[u8], identity: &Identity) -> Result<Vec<u8>, SealError> {
    let decryptor = Decryptor::new_buffered(cipher).map_err(|_| SealError::WrongKey)?;
    let reader = decryptor
        .decrypt(iter::once(identity as &dyn age::Identity))
        .map_err(|_| SealError::WrongKey)?;
    taken(reader)
}

pub fn opened_by_phrase(cipher: &[u8], phrase: &str) -> Result<Vec<u8>, SealError> {
    let decryptor = Decryptor::new_buffered(cipher).map_err(|_| SealError::WrongKey)?;
    let identity = age::scrypt::Identity::new(SecretString::from(phrase.to_owned()));
    let reader = decryptor
        .decrypt(iter::once(&identity as &dyn age::Identity))
        .map_err(|_| SealError::WrongKey)?;
    taken(reader)
}

fn written(plain: &[u8], encryptor: Encryptor) -> Result<Vec<u8>, SealError> {
    let mut cipher = Vec::new();
    let mut writer = encryptor.wrap_output(&mut cipher).map_err(broke)?;
    writer.write_all(plain).map_err(broke)?;
    writer.finish().map_err(broke)?;
    Ok(cipher)
}

fn taken(mut reader: impl Read) -> Result<Vec<u8>, SealError> {
    let mut plain = Vec::new();
    reader
        .read_to_end(&mut plain)
        .map_err(|_| SealError::WrongKey)?;
    Ok(plain)
}

fn broke(error: impl std::fmt::Display) -> SealError {
    SealError::Unwritable {
        path: "<хранилище>".to_owned(),
        reason: error.to_string(),
    }
}
