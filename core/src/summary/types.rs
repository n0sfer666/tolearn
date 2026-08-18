use crate::Hours;
use crate::progress::Status;
use crate::status::is_done;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub program: Tally,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub n: u32,
    pub title: String,
    pub tally: Tally,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    pub done: u32,
    pub total: u32,
    pub stale: u32,
    pub hours_done: Hours,
    pub hours_total: Hours,
}

impl Tally {
    pub fn share(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        f64::from(self.done) / f64::from(self.total)
    }

    pub(super) fn add(&mut self, hours: Hours, status: Status) {
        self.total += 1;
        self.hours_total = plus(self.hours_total, hours);
        if !is_done(status) {
            return;
        }
        self.done += 1;
        self.hours_done = plus(self.hours_done, hours);
        if status == Status::StalePassed {
            self.stale += 1;
        }
    }
}

fn plus(left: Hours, right: Hours) -> Hours {
    Hours {
        min: left.min + right.min,
        max: left.max + right.max,
    }
}
