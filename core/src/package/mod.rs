mod error;
mod import;
mod manifest;
mod renew;
mod scratch;
pub mod svg;
mod unpack;
mod unpack_error;
mod verify;

pub use error::PackError;
pub use import::{Imported, import};
pub use unpack::unpack;
pub use unpack_error::UnpackError;

use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;

use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::library;

pub const SCHEMA: &str = "tolearn/package/1";
pub const MANIFEST: &str = "manifest.json";

const ENTRY: SimpleFileOptions = SimpleFileOptions::DEFAULT
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

type Archive = ZipWriter<Cursor<Vec<u8>>>;

pub fn pack(directory: &Path) -> Result<Vec<u8>, PackError> {
    let tree = library::read(directory).map_err(PackError::Refused)?;
    let mut names = tree.files();
    names.sort();
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let mut sums = Vec::with_capacity(names.len());
    for name in names {
        let path = directory.join(&name);
        let data = fs::read(&path).map_err(|error| PackError::Unreadable {
            path,
            kind: error.kind(),
        })?;
        put(&mut archive, &name, &data)?;
        sums.push((name, hex(&data)));
    }
    put(&mut archive, MANIFEST, manifest::render(&sums).as_bytes())?;
    archive
        .finish()
        .map(Cursor::into_inner)
        .map_err(PackError::unwritable)
}

fn put(archive: &mut Archive, name: &str, data: &[u8]) -> Result<(), PackError> {
    archive
        .start_file(name, ENTRY)
        .map_err(PackError::unwritable)?;
    archive.write_all(data).map_err(PackError::unwritable)
}

fn hex(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
