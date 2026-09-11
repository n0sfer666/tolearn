use std::fs;
use std::path::{Path, PathBuf};

use tolearn_core::program;

use super::root;

pub fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("tolearn-{name}-{}", std::process::id()));
    if directory.exists() {
        fs::remove_dir_all(&directory).unwrap();
    }
    fs::create_dir_all(&directory).unwrap();
    directory
}

pub fn uuid_of(relative: &str) -> String {
    let source = fs::read_to_string(root().join(relative).join("program.yaml")).unwrap();
    program::parse(&source).unwrap().uuid
}

pub fn plant(data: &Path, relative: &str) -> String {
    let uuid = uuid_of(relative);
    copy_tree(&root().join(relative), &data.join("programs").join(&uuid));
    uuid
}

pub fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

pub fn listing(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

pub fn snapshot(directory: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut found = Vec::new();
    if directory.exists() {
        files(directory, directory, &mut found);
    }
    found.sort();
    found
}

fn files(top: &Path, directory: &Path, found: &mut Vec<(PathBuf, Vec<u8>)>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(top, &path, found);
        } else {
            let relative = path.strip_prefix(top).unwrap().to_owned();
            found.push((relative, fs::read(&path).unwrap()));
        }
    }
}

#[cfg(unix)]
pub fn chmod(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}
