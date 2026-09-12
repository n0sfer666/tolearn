use std::sync::{Mutex, MutexGuard, PoisonError};

use super::record::Record;

#[derive(Debug, Default)]
pub struct Tally(Mutex<Vec<Record>>);

impl Tally {
    pub fn len(&self) -> usize {
        self.held().len()
    }

    pub fn is_empty(&self) -> bool {
        self.held().is_empty()
    }

    pub fn records(&self) -> Vec<Record> {
        self.held().clone()
    }

    pub fn take(&self) -> Vec<Record> {
        std::mem::take(&mut *self.held())
    }

    pub fn extend(&self, records: Vec<Record>) {
        self.held().extend(records);
    }

    pub fn restore(&self, records: Vec<Record>) {
        let mut held = self.held();
        if held.is_empty() {
            *held = records;
        }
    }

    pub fn replace(&self, records: Vec<Record>) {
        *self.held() = records;
    }

    pub fn release(&self) -> Vec<Record> {
        let mut records = self.take();
        for record in &mut records {
            record.program = None;
            record.stage = None;
            record.at = None;
        }
        records
    }

    pub(crate) fn push(&self, record: Record) {
        self.held().push(record);
    }

    pub(crate) fn stamp(&self, from: usize, program: &str, stage: Option<&str>) {
        for record in self
            .held()
            .iter_mut()
            .skip(from)
            .filter(|record| record.program.is_none())
        {
            record.program = Some(program.to_owned());
            record.stage = stage.map(str::to_owned);
        }
    }

    pub(crate) fn date(&self, at: i64) {
        for record in self.held().iter_mut().filter(|record| record.at.is_none()) {
            record.at = Some(at);
        }
    }

    fn held(&self) -> MutexGuard<'_, Vec<Record>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
