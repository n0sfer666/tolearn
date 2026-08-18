use std::path::Path;
use std::process::Command;

pub fn room(room: &Path) -> Result<(), String> {
    std::fs::create_dir_all(room).map_err(|error| error.to_string())?;
    Command::new(opener())
        .arg(room)
        .status()
        .map_err(|error| error.to_string())
        .and_then(|status| match status.success() {
            true => Ok(()),
            false => Err(format!("`{}` вернул {status}", opener())),
        })
}

fn opener() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    }
}
