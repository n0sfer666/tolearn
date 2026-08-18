#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tolearn_offline::video::{Os, Tool, ready};

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("tolearn-tools-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(path.join("bin")).unwrap();
    path
}

fn put(at: &Path, tool: Tool) {
    std::fs::write(at.join("bin").join(tool.file()), "заглушка").unwrap();
}

fn bin(at: &Path) -> String {
    at.join("bin").to_str().unwrap().to_string()
}

#[test]
fn без_бинарников_действие_неактивно() {
    let at = root("none");

    let absent = ready(&bin(&at)).unwrap_err();

    assert_eq!(
        absent.iter().map(|gap| gap.tool).collect::<Vec<_>>(),
        vec![Tool::Ytdlp, Tool::Ffmpeg]
    );
    for gap in &absent {
        assert!(
            gap.install.contains(gap.tool.binary()),
            "команда установки не называет сам инструмент: {}",
            gap.install
        );
    }
}

#[test]
fn команда_установки_своя_для_каждой_ос() {
    let commands = [Os::Macos, Os::Linux, Os::Windows]
        .map(|os| Tool::Ytdlp.install(os))
        .to_vec();

    assert!(commands.contains(&"brew install yt-dlp".to_string()));
    assert_eq!(
        commands
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        3,
        "команды для разных ОС совпали: {commands:?}"
    );
}

#[test]
fn не_хватает_одного_бинарника() {
    let at = root("half");
    put(&at, Tool::Ytdlp);

    let absent = ready(&bin(&at)).unwrap_err();

    assert_eq!(
        absent.iter().map(|gap| gap.tool).collect::<Vec<_>>(),
        vec![Tool::Ffmpeg]
    );
}

#[test]
fn оба_бинарника_найдены() {
    let at = root("both");
    put(&at, Tool::Ytdlp);
    put(&at, Tool::Ffmpeg);

    let tools = ready(&bin(&at)).unwrap();

    assert!(tools.ytdlp.ends_with(Tool::Ytdlp.file()));
    assert!(tools.ffmpeg.ends_with(Tool::Ffmpeg.file()));
}

#[test]
fn путь_режется_разделителем_своей_ос() {
    let at = root("path");
    put(&at, Tool::Ytdlp);
    put(&at, Tool::Ffmpeg);
    let path = format!(
        "{1}{0}{2}{0}",
        tolearn_offline::video::separator(),
        at.join("пусто").display(),
        bin(&at)
    );

    let tools = ready(&path).unwrap();

    assert!(tools.ytdlp.starts_with(&at));
}
