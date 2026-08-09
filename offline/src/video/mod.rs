mod error;
mod group;
mod percent;
mod tools;

pub use error::VideoError;
pub use tools::{Absent, Os, Tool, Tools, ready};

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

const POLL: Duration = Duration::from_millis(20);

#[derive(Debug, Clone, Copy)]
pub struct Wanted {
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Saved {
    pub path: PathBuf,
}

impl Default for Wanted {
    fn default() -> Self {
        Self { height: 720 }
    }
}

pub fn download(
    tools: &Tools,
    url: &str,
    into: &Path,
    wanted: Wanted,
    watch: &mut dyn FnMut(f32),
    stop: &AtomicBool,
) -> Result<Saved, VideoError> {
    let height = wanted.height;
    let mut command = Command::new(&tools.ytdlp);
    command
        .args(["--newline", "--no-playlist", "--ffmpeg-location"])
        .arg(&tools.ffmpeg)
        .arg("--format")
        .arg(format!("bv*[height<={height}]+ba/b[height<={height}]"))
        .arg("--output")
        .arg(into)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = group::lead(&mut command)
        .spawn()
        .map_err(VideoError::NotStarted)?;

    let lines = listen(&mut child);
    let cancelled = follow(&lines, watch, stop);
    if cancelled {
        kill(&mut child);
        return Err(VideoError::Cancelled);
    }
    finish(child, into)
}

fn listen(child: &mut Child) -> Receiver<String> {
    let (sender, receiver) = channel();
    if let Some(stdout) = child.stdout.take() {
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    return;
                }
            }
        });
    }
    receiver
}

fn follow(lines: &Receiver<String>, watch: &mut dyn FnMut(f32), stop: &AtomicBool) -> bool {
    loop {
        if stop.load(Ordering::Relaxed) {
            return true;
        }
        match lines.recv_timeout(POLL) {
            Ok(line) => {
                if let Some(share) = percent::percent(&line) {
                    watch(share);
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return stop.load(Ordering::Relaxed),
        }
    }
}

fn finish(mut child: Child, into: &Path) -> Result<Saved, VideoError> {
    let mut complaint = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _ = std::io::Read::read_to_string(&mut stderr, &mut complaint);
    }
    let status = child.wait().map_err(VideoError::NotStarted)?;
    if !status.success() {
        return Err(VideoError::Failed(complaint.trim().to_string()));
    }
    if !into.is_file() {
        return Err(VideoError::Failed(format!(
            "файл {} не появился",
            into.display()
        )));
    }
    Ok(Saved {
        path: into.to_path_buf(),
    })
}

fn kill(child: &mut Child) {
    group::tree(child.id());
    let _ = child.wait();
}
