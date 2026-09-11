use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_core::package;

use super::repository;

pub const CHIPTUNE: &str = "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84";
pub const NES_DEV: &str = "7a1d4e90-2c3b-4f58-8d6e-1b9a0c5e7f23";
pub const TOOLS: &str = "b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56";
pub const ROM: &str = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";
pub const SOUND: &str = "5c8e1f2a-7b3d-4e69-9a04-2d6f8b1c3e75";

pub struct Shelf {
    pub context: Context,
    pub data: PathBuf,
    pub incoming: PathBuf,
}

impl Shelf {
    pub fn new(name: &str) -> Self {
        let top = std::env::temp_dir().join(format!("tolearn-shelf-{name}-{}", std::process::id()));
        if top.exists() {
            std::fs::remove_dir_all(&top).unwrap();
        }
        let data = top.join("data");
        let incoming = top.join("incoming");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::create_dir_all(&incoming).unwrap();
        Self {
            context: Context::new(&data),
            data,
            incoming,
        }
    }

    pub fn packed(&self, relative: &str) -> PathBuf {
        let name = Path::new(relative).file_name().unwrap().to_string_lossy();
        let file = self.incoming.join(format!("{name}.tolearn"));
        let bytes = package::pack(&repository().join(relative)).unwrap();
        std::fs::write(&file, bytes).unwrap();
        file
    }

    pub fn ask(&self, name: &str, payload: Value) -> Result<Value, IpcError> {
        call(&self.context, name, &payload)
    }

    pub fn import(&self, file: &Path) -> Result<Value, IpcError> {
        self.ask(
            "import_package",
            json!({ "path": file.display().to_string() }),
        )
    }

    pub fn shelved(&self, relative: &str) -> Value {
        self.import(&self.packed(relative)).unwrap()
    }

    pub fn library(&self) -> Value {
        self.ask("library", json!({})).unwrap()
    }
}

pub fn original(relative: &str) -> Vec<u8> {
    std::fs::read(repository().join(relative)).unwrap()
}

pub fn uri(mime: &str, relative: &str) -> String {
    format!("data:{mime};base64,{}", STANDARD.encode(original(relative)))
}

pub fn ids(rows: &Value, key: &str) -> Vec<String> {
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| row[key].as_str().unwrap().to_owned())
        .collect()
}
