use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use super::live::{Live, tailed};
use super::made::{Made, Summary};

#[derive(Debug, Default)]
pub struct Job {
    stop: AtomicBool,
    go: AtomicBool,
    again: AtomicBool,
    live: Mutex<Live>,
    made: Mutex<Option<Made>>,
    since: Mutex<Option<Instant>>,
    stepped: Mutex<Option<Instant>>,
}

impl Job {
    pub fn stepping(&self, step: &str, current: &str) {
        self.mark(&self.stepped);
        self.change(|live| {
            step.clone_into(&mut live.step);
            current.clone_into(&mut live.current);
            live.attempt = 1;
            live.retry = 0;
        });
    }

    pub fn attempting(&self, attempt: u32) {
        self.change(|live| live.attempt = attempt);
    }

    pub fn retrying(&self, retry: u32) {
        self.change(|live| live.retry = retry);
    }

    pub fn counted(&self, total: usize, done: usize) {
        self.change(|live| {
            live.total = total;
            live.done = done;
        });
    }

    pub fn stepped(&self) {
        self.change(|live| live.done += 1);
    }

    pub fn asking(&self) {
        self.mark(&self.since);
        self.change(|live| {
            live.chars = 0;
            live.ticks = 0;
            live.tail = String::new();
        });
    }

    pub fn heard(&self, piece: &str) {
        let letters = u64::try_from(piece.chars().count()).unwrap_or_default();
        self.change(|live| {
            live.chars += letters;
            live.ticks += 1;
            live.tail = tailed(&live.tail, piece);
        });
    }

    pub fn spent(&self, tokens: Option<u32>) {
        let seconds = self
            .since
            .lock()
            .ok()
            .and_then(|mut since| since.take())
            .map_or(0, |at| at.elapsed().as_secs());
        self.change(|live| {
            live.seconds += seconds;
            if let Some(counted) = tokens {
                live.tokens = Some(live.tokens.unwrap_or_default() + counted);
            }
        });
    }

    pub fn told(&self, made: Made, summary: Summary) {
        if let Ok(mut kept) = self.made.lock() {
            *kept = Some(made);
        }
        self.change(|live| {
            live.step = "done".to_owned();
            live.current = String::new();
            live.finished = true;
            live.summary = Some(summary);
        });
    }

    pub fn broke(&self, refused: Vec<String>) {
        self.change(|live| {
            live.finished = true;
            live.cancelled = refused.is_empty();
            live.refused = refused;
            live.current = String::new();
        });
    }

    pub fn waits(&self) {
        self.change(|live| live.waiting = true);
    }

    pub fn goes(&self) {
        self.go.store(true, Ordering::Relaxed);
        self.again.store(true, Ordering::Relaxed);
        self.change(|live| live.waiting = false);
    }

    pub fn missing(&self, missed: Vec<String>) {
        self.again.store(false, Ordering::Relaxed);
        self.change(|live| live.missed = missed);
    }

    pub fn mending(&self) {
        self.again.store(false, Ordering::Relaxed);
        self.change(|live| live.missed = Vec::new());
    }

    pub fn stopping(&self) {
        self.stop.store(true, Ordering::Relaxed);
        self.goes();
    }

    pub fn made(&self) -> Option<Made> {
        self.made.lock().ok()?.clone()
    }

    pub fn allowed(&self) -> bool {
        self.go.load(Ordering::Relaxed)
    }

    pub fn asked(&self) -> bool {
        self.again.load(Ordering::Relaxed)
    }

    pub fn stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    pub fn look(&self) -> Live {
        let waiting = self.waited(&self.since);
        let stepping = self.waited(&self.stepped);
        let mut live = self
            .live
            .lock()
            .map(|live| live.clone())
            .unwrap_or_default();
        live.seconds += waiting;
        live.step_seconds = stepping;
        live
    }

    fn mark(&self, at: &Mutex<Option<Instant>>) {
        if let Ok(mut at) = at.lock() {
            *at = Some(Instant::now());
        }
    }

    fn waited(&self, at: &Mutex<Option<Instant>>) -> u64 {
        at.lock()
            .ok()
            .and_then(|at| *at)
            .map_or(0, |at| at.elapsed().as_secs())
    }

    fn change(&self, edit: impl FnOnce(&mut Live)) {
        if let Ok(mut live) = self.live.lock() {
            edit(&mut live);
        }
    }
}
