use tolearn_core::sealed::{Keys, lock, sealed, settle, unlock};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::settings::stored;
use crate::ipc::types::{EncryptionIn, EncryptionOut};
use crate::ipc::vaulted::{broke, keys};

const SHORT: usize = 8;

pub fn run(context: &Context, input: &EncryptionIn) -> Result<EncryptionOut, IpcError> {
    let root = context.notes();
    settle(&root).map_err(broke)?;

    match input.enable {
        None => (),
        Some(true) if sealed(&root) => (),
        Some(true) => enable(context, input)?,
        Some(false) if !sealed(&root) => (),
        Some(false) => disable(context, input)?,
    }

    Ok(EncryptionOut {
        enabled: sealed(&context.notes()),
        external: stored(context)?.notes_directory.is_some(),
    })
}

fn enable(context: &Context, input: &EncryptionIn) -> Result<(), IpcError> {
    if stored(context)?.notes_directory.is_some() {
        return Err(IpcError::new(
            "encryption.external",
            "шифрование работает только с внутренним каталогом: сначала уберите внешний".to_owned(),
        ));
    }
    let phrase = phrase(input)?;
    let made = lock(&context.notes(), phrase).map_err(broke)?;
    remember(context, &made)
}

fn disable(context: &Context, input: &EncryptionIn) -> Result<(), IpcError> {
    let held = match input
        .phrase
        .as_deref()
        .map(str::trim)
        .filter(|it| !it.is_empty())
    {
        Some(phrase) => tolearn_core::sealed::keys(&context.notes(), phrase).map_err(broke)?,
        None => keys(context)?,
    };
    unlock(&context.notes(), &held).map_err(broke)?;
    context
        .keys()
        .forget()
        .map_err(|error| IpcError::new("notes.vault", error.to_string()))
}

fn remember(context: &Context, made: &Keys) -> Result<(), IpcError> {
    context
        .keys()
        .store(&made.secret())
        .map_err(|error| IpcError::new("notes.vault", error.to_string()))
}

fn phrase(input: &EncryptionIn) -> Result<&str, IpcError> {
    let given = input.phrase.as_deref().map(str::trim).unwrap_or_default();
    if given.chars().count() < SHORT {
        return Err(IpcError::new(
            "encryption.short-phrase",
            format!("парольная фраза короче {SHORT} знаков: её подберут"),
        ));
    }
    Ok(given)
}
