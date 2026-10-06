use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tolearn_generate::ledger::{Record, Tally};

#[derive(Debug, Clone, Default)]
pub struct Ledger {
    tally: Arc<Tally>,
    drawn: Arc<AtomicU64>,
}

impl Ledger {
    pub fn tally(&self) -> &Tally {
        &self.tally
    }

    pub fn ticket(&self) -> u64 {
        self.drawn.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn take(&self) -> Vec<Record> {
        self.ticket();
        self.tally.take()
    }

    pub fn settle(&self, ticket: u64, records: Vec<Record>, keep: fn(&Tally, Vec<Record>)) {
        if self.drawn.load(Ordering::SeqCst) == ticket {
            keep(&self.tally, records);
            return;
        }
        self.tally.extend(records);
    }
}
