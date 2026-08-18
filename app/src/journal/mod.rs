mod record;
mod reveal;
mod room;

pub use reveal::room as reveal;
pub use room::{clear, records};

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use record::Answer;

pub const KEEP: usize = 20;

static TURNS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Journal {
    room: Option<PathBuf>,
}

impl Journal {
    pub fn new(room: PathBuf, on: bool) -> Self {
        Self {
            room: on.then_some(room),
        }
    }

    pub fn off() -> Self {
        Self { room: None }
    }

    pub fn said(&self, kind: &str, prompt: &str, said: &str) {
        self.note(kind, prompt, &Answer::Said(said));
    }

    pub fn refused(&self, kind: &str, prompt: &str, refusal: &str) {
        self.note(kind, prompt, &Answer::Refused(refusal));
    }

    fn note(&self, kind: &str, prompt: &str, answer: &Answer<'_>) {
        let Some(room) = self.room.as_ref() else {
            return;
        };
        if std::fs::create_dir_all(room).is_err() {
            return;
        }
        let name = record::name(seconds(), TURNS.fetch_add(1, Ordering::Relaxed));
        if std::fs::write(room.join(name), record::text(kind, prompt, answer)).is_err() {
            return;
        }
        room::rotate(room, KEEP);
    }
}

fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or_default()
}
