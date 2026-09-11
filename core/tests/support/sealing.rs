use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::write::GzEncoder;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use tar::{Builder, Header};
use tolearn_core::package::{MANIFEST, SCHEMA, pack};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use super::programs::snapshot;
use super::root;

pub type Files = Vec<(String, Vec<u8>)>;

pub fn packed(room: &Path, relative: &str) -> PathBuf {
    let file = room.join("program.tolearn");
    fs::write(&file, pack(&root().join(relative)).unwrap()).unwrap();
    file
}

pub fn content(relative: &str) -> Files {
    let package = pack(&root().join(relative)).unwrap();
    let mut zip = ZipArchive::new(Cursor::new(package)).unwrap();
    let mut files = Vec::new();
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).unwrap();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        if entry.name() != MANIFEST {
            files.push((entry.name().to_owned(), data));
        }
    }
    files
}

pub fn loose(relative: &str) -> Files {
    snapshot(&root().join(relative))
        .into_iter()
        .map(|(path, data)| (path.to_string_lossy().replace('\\', "/"), data))
        .collect()
}

pub fn sealed(files: Files) -> Vec<u8> {
    let listed = manifest(SCHEMA, &files);
    zipped(&with(files, MANIFEST, &listed))
}

pub fn manifest(schema: &str, files: &Files) -> Vec<u8> {
    let sums: Map<String, Value> = files
        .iter()
        .map(|(name, data)| (name.clone(), Value::from(sha256(data))))
        .collect();
    serde_json::to_vec_pretty(&json!({ "schema": schema, "files": sums })).unwrap()
}

pub fn with(mut files: Files, name: &str, data: &[u8]) -> Files {
    files.retain(|(held, _)| held != name);
    files.push((name.to_owned(), data.to_vec()));
    files
}

pub fn without(mut files: Files, name: &str) -> Files {
    files.retain(|(held, _)| held != name);
    files
}

pub fn zipped(files: &Files) -> Vec<u8> {
    written(files).finish().unwrap().into_inner()
}

pub fn tarred(files: &Files) -> Vec<u8> {
    let mut builder = Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (name, data) in files {
        let mut header = Header::new_gnu();
        header.set_size(u64::try_from(data.len()).unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, name, data.as_slice())
            .unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap()
}

pub fn linked(files: &Files, name: &str, target: &str) -> Vec<u8> {
    let mut writer = written(files);
    writer
        .add_symlink(name, target, SimpleFileOptions::default())
        .unwrap();
    writer.finish().unwrap().into_inner()
}

fn written(files: &Files) -> ZipWriter<Cursor<Vec<u8>>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in files {
        writer
            .start_file(name.clone(), SimpleFileOptions::default())
            .unwrap();
        writer.write_all(data).unwrap();
    }
    writer
}

fn sha256(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
