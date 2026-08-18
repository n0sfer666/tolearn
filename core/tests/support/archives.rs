use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::write::GzEncoder;
use tar::{EntryType, Header};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone)]
pub enum Entry {
    File { path: String, body: Vec<u8> },
    Symlink { path: String, target: String },
    Hardlink { path: String, target: String },
}

impl Entry {
    pub fn file(path: &str, body: Vec<u8>) -> Self {
        Self::File {
            path: path.to_owned(),
            body,
        }
    }

    pub fn link(path: &str, target: &str) -> Self {
        Self::Symlink {
            path: path.to_owned(),
            target: target.to_owned(),
        }
    }

    pub fn hard(path: &str, target: &str) -> Self {
        Self::Hardlink {
            path: path.to_owned(),
            target: target.to_owned(),
        }
    }

    pub fn under(&self, prefix: &str) -> Self {
        match self {
            Self::File { path, body } => Self::file(&format!("{prefix}/{path}"), body.clone()),
            Self::Symlink { path, target } => Self::link(&format!("{prefix}/{path}"), target),
            Self::Hardlink { path, target } => Self::hard(&format!("{prefix}/{path}"), target),
        }
    }
}

pub fn zipped(room: &Path, entries: &[Entry]) -> PathBuf {
    let path = room.join("бандл.zip");
    let mut writer = ZipWriter::new(std::fs::File::create(&path).unwrap());
    for entry in entries {
        match entry {
            Entry::File { path, body } => {
                writer
                    .start_file(path.clone(), SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(body).unwrap();
            }
            Entry::Symlink { path, target } | Entry::Hardlink { path, target } => {
                writer
                    .add_symlink(path.clone(), target.clone(), SimpleFileOptions::default())
                    .unwrap();
            }
        }
    }
    writer.finish().unwrap();
    path
}

pub fn gzipped(room: &Path, entries: &[Entry]) -> PathBuf {
    let path = room.join("бандл.tar.gz");
    let file = std::fs::File::create(&path).unwrap();
    let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::default()));
    for entry in entries {
        match entry {
            Entry::File { path, body } => {
                let mut header = Header::new_gnu();
                header.set_size(body.len() as u64);
                header.set_mode(0o644);
                named(&mut header, path);
                header.set_cksum();
                builder.append(&header, body.as_slice()).unwrap();
            }
            Entry::Symlink { path, target } => {
                builder
                    .append_link(&mut linking(EntryType::Symlink), path, target)
                    .unwrap();
            }
            Entry::Hardlink { path, target } => {
                builder
                    .append_link(&mut linking(EntryType::Link), path, target)
                    .unwrap();
            }
        }
    }
    builder.into_inner().unwrap().finish().unwrap();
    path
}

fn named(header: &mut Header, path: &str) {
    let raw = path.as_bytes();
    header.as_gnu_mut().unwrap().name[..raw.len()].copy_from_slice(raw);
}

fn linking(kind: EntryType) -> Header {
    let mut header = Header::new_gnu();
    header.set_size(0);
    header.set_mode(0o644);
    header.set_entry_type(kind);
    header
}
