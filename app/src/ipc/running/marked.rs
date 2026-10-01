use tolearn_generate::{Progress, Step};

use super::super::announced::told;
use super::Running;

pub struct Marked {
    running: Running,
    inner: Box<dyn Progress>,
}

impl Marked {
    pub fn new(running: Running, inner: Box<dyn Progress>) -> Self {
        Self { running, inner }
    }
}

impl Progress for Marked {
    fn began(&self, step: Step) {
        self.running.mark(told("began", step));
        self.inner.began(step);
    }

    fn ended(&self, step: Step) {
        self.running.mark(told("ended", step));
        self.inner.ended(step);
    }
}
