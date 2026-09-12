use std::cell::RefCell;
use std::io::Write;
use std::time::Instant;

use tolearn_generate::{Progress, Step};

use super::crew::Crew;
use super::shown::{named, spent};
use super::spent::Spent;
use super::unpainted::UNPAINTED;

#[derive(Debug)]
struct Open {
    step: Step,
    began: Instant,
    spent: Spent,
    painted: usize,
}

#[derive(Debug)]
pub struct Herald<'a> {
    crew: &'a Crew,
    open: RefCell<Vec<Open>>,
}

impl<'a> Herald<'a> {
    pub fn new(crew: &'a Crew) -> Self {
        Self {
            crew,
            open: RefCell::new(Vec::new()),
        }
    }
}

impl Progress for Herald<'_> {
    fn began(&self, step: Step) {
        self.open.borrow_mut().push(Open {
            step,
            began: Instant::now(),
            spent: self.crew.speaker.meter.spent(),
            painted: self.crew.painter.count(),
        });
    }

    fn ended(&self, step: Step) {
        let mut open = self.open.borrow_mut();
        let Some(index) = open.iter().rposition(|opened| opened.step == step) else {
            return;
        };
        let opened = open.remove(index);
        let mut parts = spent(
            opened.began.elapsed(),
            self.crew.speaker.meter.spent().since(opened.spent),
        );
        if self.crew.painter.count() > opened.painted {
            parts.push(format!("схемы остались исходником Mermaid: {UNPAINTED}"));
        }
        let _ = writeln!(
            self.crew.log.borrow_mut(),
            "{} · {}",
            named(step),
            parts.join(" · ")
        );
    }
}
