use std::io::Read;
use std::thread::{JoinHandle, spawn};

use super::types::Seen;

const CHUNK: usize = 8 * 1024;

pub type Drained = JoinHandle<(String, bool)>;

pub fn start(
    pipe: Option<impl Read + Send + 'static>,
    limit: usize,
    seen: Option<Seen>,
) -> Drained {
    spawn(move || {
        let Some(mut pipe) = pipe else {
            return (String::new(), false);
        };
        let mut kept: Vec<u8> = Vec::new();
        let mut pending: Vec<u8> = Vec::new();
        let mut truncated = false;
        let mut chunk = [0_u8; CHUNK];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    if let Some(seen) = seen.as_ref() {
                        pending.extend_from_slice(&chunk[..read]);
                        let text = whole(&mut pending);
                        if !text.is_empty() {
                            seen(&text);
                        }
                    }
                    let room = limit.saturating_sub(kept.len());
                    if room == 0 {
                        truncated = true;
                        continue;
                    }
                    let taken = room.min(read);
                    kept.extend_from_slice(&chunk[..taken]);
                    truncated = truncated || taken < read;
                }
            }
        }
        (String::from_utf8_lossy(&kept).into_owned(), truncated)
    })
}

pub fn done(handle: Drained) -> (String, bool) {
    handle.join().unwrap_or_else(|_| (String::new(), false))
}

fn whole(pending: &mut Vec<u8>) -> String {
    let good = match std::str::from_utf8(pending) {
        Ok(_) => pending.len(),
        Err(error) => match error.error_len() {
            Some(broken) => error.valid_up_to() + broken,
            None => error.valid_up_to(),
        },
    };
    let text = String::from_utf8_lossy(&pending[..good]).into_owned();
    pending.drain(..good);
    text
}
