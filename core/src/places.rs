use std::path::{Path, PathBuf};

const APP: &str = "tolearn";

#[derive(Debug, Clone)]
pub struct Places {
    pub config: PathBuf,
    pub data: PathBuf,
}

pub fn places(home: &Path) -> Places {
    Places {
        config: root("XDG_CONFIG_HOME", home, ".config"),
        data: root("XDG_DATA_HOME", home, ".local/share"),
    }
}

fn root(variable: &str, home: &Path, fallback: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| home.join(fallback))
        .join(APP)
}
