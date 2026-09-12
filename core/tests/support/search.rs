use std::fs;
use std::path::{Path, PathBuf};

use tolearn_core::library::Library;
use tolearn_core::search::Index;

use super::programs::{plant, scratch};

pub const CHIPTUNE: &str = "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84";

pub fn shelf(name: &str) -> (PathBuf, Library) {
    let data = scratch(&format!("search-{name}"));
    plant(&data, "examples/chiptune");
    plant(&data, "fixtures/v2/valid/nes-dev");
    let library = Library::at(&data);
    (data, library)
}

pub fn home(data: &Path) -> PathBuf {
    data.join("programs").join(CHIPTUNE)
}

pub fn retitle(data: &Path, title: &str) {
    let path = home(data).join("stages/voices.yaml");
    let text = fs::read_to_string(&path).unwrap();
    let line = format!("title: {title}");
    fs::write(&path, text.replacen("title: Голоса чипа", &line, 1)).unwrap();
}

pub fn fresh(library: &Library) -> Index {
    let mut index = Index::default();
    index.refresh(library).unwrap();
    index
}
