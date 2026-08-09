#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};
use std::process::Command;

use tolearn_offline::repo::{RepoError, clone};

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("tolearn-repo-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn git(at: &Path, args: &[&str]) {
    let done = Command::new("git")
        .current_dir(at)
        .args(args)
        .output()
        .unwrap();
    assert!(
        done.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&done.stderr)
    );
}

fn source(at: &Path, big: usize) -> PathBuf {
    let path = at.join("source");
    std::fs::create_dir_all(&path).unwrap();
    git(&path, &["init", "--initial-branch=main"]);
    git(&path, &["config", "user.email", "learner@tolearn.test"]);
    git(&path, &["config", "user.name", "Ученик"]);
    std::fs::write(path.join("README.md"), "первый заход\n").unwrap();
    git(&path, &["add", "."]);
    git(&path, &["commit", "-m", "первый"]);
    std::fs::write(path.join("README.md"), "второй заход\n").unwrap();
    if big > 0 {
        std::fs::write(path.join("blob.bin"), vec![b'x'; big]).unwrap();
    }
    git(&path, &["add", "."]);
    git(&path, &["commit", "-m", "второй"]);
    path
}

fn commits(at: &Path) -> usize {
    let done = Command::new("git")
        .current_dir(at)
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&done.stdout)
        .trim()
        .parse()
        .unwrap()
}

#[test]
fn клон_поверхностный() {
    let root = root("shallow");
    let source = source(&root, 0);
    let into = root.join("clone");

    let cloned = clone(source.to_str().unwrap(), &into, 16 * 1024 * 1024).unwrap();

    assert_eq!(commits(&cloned.path), 1, "притянута вся история");
    assert!(
        cloned.path.join(".git/shallow").exists(),
        "клон не помечен поверхностным"
    );
}

#[test]
fn рабочее_дерево_разложено() {
    let root = root("worktree");
    let source = source(&root, 0);
    let into = root.join("clone");

    let cloned = clone(source.to_str().unwrap(), &into, 16 * 1024 * 1024).unwrap();

    let laid = std::fs::read_to_string(cloned.path.join("README.md")).unwrap();

    assert_eq!(laid.replace("\r\n", "\n"), "второй заход\n");
}

#[test]
fn превышение_объёма_прерывает_клон() {
    let root = root("limit");
    let source = source(&root, 4 * 1024 * 1024);
    let into = root.join("clone");

    let failure = clone(source.to_str().unwrap(), &into, 64 * 1024).unwrap_err();

    assert!(
        matches!(failure, RepoError::TooBig { limit, .. } if limit == 64 * 1024),
        "объём не назван причиной: {failure}"
    );
    assert!(!into.exists(), "оборванный клон остался на диске");
}

#[test]
fn объём_клона_известен() {
    let root = root("size");
    let source = source(&root, 0);
    let into = root.join("clone");

    let cloned = clone(source.to_str().unwrap(), &into, 16 * 1024 * 1024).unwrap();

    assert!(cloned.bytes > 0, "объём принятых данных не посчитан");
}

#[test]
fn несуществующий_адрес_это_ошибка() {
    let root = root("missing");
    let into = root.join("clone");
    std::fs::create_dir_all(&into).unwrap();

    let failure = clone(root.join("nowhere").to_str().unwrap(), &into, 1024).unwrap_err();

    assert!(matches!(failure, RepoError::Clone(_)), "{failure}");
    assert!(!into.exists(), "пустой каталог остался на диске");
}
