use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use tolearn_app::discard::Bin;

#[derive(Debug, Clone)]
pub struct Bucket {
    room: PathBuf,
    jam: Option<String>,
    taken: Arc<Mutex<Vec<String>>>,
}

impl Bucket {
    pub fn new(room: &Path, jam: Option<&str>) -> Self {
        Self {
            room: room.to_path_buf(),
            jam: jam.map(str::to_owned),
            taken: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn taken(&self) -> Vec<String> {
        self.taken
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn holds(&self, name: &str) -> bool {
        self.room.join(name).exists()
    }
}

impl Bin for Bucket {
    fn discard(&self, path: &Path) -> Result<(), String> {
        let name = named(path);
        if self.jam.as_ref().is_some_and(|mark| name.starts_with(mark)) {
            return Err(format!("подставная корзина не принимает `{name}`"));
        }
        std::fs::create_dir_all(&self.room).map_err(|error| error.to_string())?;
        std::fs::rename(path, self.room.join(&name)).map_err(|error| error.to_string())?;
        self.taken
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(name);
        Ok(())
    }
}

fn named(path: &Path) -> String {
    let leaf = |part: Option<&std::ffi::OsStr>| {
        part.map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let room = leaf(path.parent().and_then(Path::file_name));
    format!("{room}-{}", leaf(path.file_name()))
}
