#![cfg(unix)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use tolearn_offline::video::{VideoError, Wanted, download, ready};

const YTDLP: &str = r#"#!/bin/sh
echo "$@" > "$(dirname "$0")/../args.txt"
echo "[download]   0.0% of 10.00MiB at 1.00MiB/s ETA 00:10"
echo "[download]  47.5% of 10.00MiB at 1.00MiB/s ETA 00:05"
echo "[download] 100% of 10.00MiB in 00:10"
out=$(echo "$@" | sed 's/.*--output //;s/ .*//')
echo "видео" > "$out"
exit 0
"#;

const SLOW: &str = r#"#!/bin/sh
echo "[download]   0.0% of 10.00MiB at 1.00MiB/s ETA 00:10"
sleep 30
exit 0
"#;

const EMPTY: &str = r#"#!/bin/sh
echo "[download] 100% of 10.00MiB in 00:10"
exit 0
"#;

const BROKEN: &str = r#"#!/bin/sh
echo "ERROR: Video unavailable" >&2
exit 1
"#;

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("tolearn-video-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(path.join("bin")).unwrap();
    path
}

fn put(at: &Path, name: &str, body: &str) {
    let path = at.join("bin").join(name);
    std::fs::write(&path, body).unwrap();
    let mut rights = std::fs::metadata(&path).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut rights, 0o755);
    std::fs::set_permissions(&path, rights).unwrap();
}

fn bin(at: &Path) -> String {
    at.join("bin").to_str().unwrap().to_string()
}

fn silent() -> impl FnMut(f32) {
    |_| {}
}

fn alone() -> MutexGuard<'static, ()> {
    static GATE: Mutex<()> = Mutex::new(());

    GATE.lock().unwrap_or_else(PoisonError::into_inner)
}

#[test]
fn скачивание_показывает_прогресс() {
    let _alone = alone();
    let at = root("progress");
    put(&at, "yt-dlp", YTDLP);
    put(&at, "ffmpeg", YTDLP);
    let tools = ready(&bin(&at)).unwrap();
    let seen = Mutex::new(Vec::new());

    let saved = download(
        &tools,
        "https://example.test/lecture",
        &at.join("video.mp4"),
        Wanted::default(),
        &mut |percent| seen.lock().unwrap().push(percent),
        &AtomicBool::new(false),
    )
    .unwrap();

    assert!(saved.path.exists(), "файл видео не появился");
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen, vec![0.0, 47.5, 100.0], "прогресс не тот");
}

#[test]
fn по_умолчанию_не_выше_720p() {
    let _alone = alone();
    let at = root("height");
    put(&at, "yt-dlp", YTDLP);
    put(&at, "ffmpeg", YTDLP);
    let tools = ready(&bin(&at)).unwrap();

    download(
        &tools,
        "https://example.test/lecture",
        &at.join("video.mp4"),
        Wanted::default(),
        &mut silent(),
        &AtomicBool::new(false),
    )
    .unwrap();

    let args = std::fs::read_to_string(at.join("args.txt")).unwrap();
    assert!(
        args.contains("height<=720"),
        "потолок разрешения не задан: {args}"
    );
    let ffmpeg = tools.ffmpeg.to_str().unwrap();
    assert!(
        args.contains(&format!("--ffmpeg-location {ffmpeg}")),
        "yt-dlp получил не найденный нами ffmpeg: {args}"
    );
}

#[test]
fn отмена_прерывает_скачивание() {
    let _alone = alone();
    let at = root("cancel");
    put(&at, "yt-dlp", SLOW);
    put(&at, "ffmpeg", YTDLP);
    let tools = ready(&bin(&at)).unwrap();
    let stop = AtomicBool::new(false);
    let started = std::time::Instant::now();

    let failure = download(
        &tools,
        "https://example.test/lecture",
        &at.join("video.mp4"),
        Wanted::default(),
        &mut |_| stop.store(true, Ordering::Relaxed),
        &stop,
    )
    .unwrap_err();

    assert!(matches!(failure, VideoError::Cancelled), "{failure}");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "отмена не прервала процесс"
    );
}

#[test]
fn провал_бинарника_это_ошибка() {
    let _alone = alone();
    let at = root("broken");
    put(&at, "yt-dlp", BROKEN);
    put(&at, "ffmpeg", YTDLP);
    let tools = ready(&bin(&at)).unwrap();

    let failure = download(
        &tools,
        "https://example.test/lecture",
        &at.join("video.mp4"),
        Wanted::default(),
        &mut silent(),
        &AtomicBool::new(false),
    )
    .unwrap_err();

    assert!(
        matches!(&failure, VideoError::Failed(reason) if reason.contains("Video unavailable")),
        "{failure}"
    );
}

#[test]
fn успех_без_файла_это_ошибка() {
    let _alone = alone();
    let at = root("empty");
    put(&at, "yt-dlp", EMPTY);
    put(&at, "ffmpeg", EMPTY);
    let tools = ready(&bin(&at)).unwrap();

    let failure = download(
        &tools,
        "https://example.test/lecture",
        &at.join("video.mp4"),
        Wanted::default(),
        &mut silent(),
        &AtomicBool::new(false),
    )
    .unwrap_err();

    assert!(matches!(failure, VideoError::Failed(_)), "{failure}");
}
