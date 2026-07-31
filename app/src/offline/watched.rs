use std::cell::RefCell;
use std::collections::HashMap;

use tolearn_core::topic::Material;
use tolearn_offline::fresh::{Kept, Probe, WINDOW, weighed};
use tolearn_offline::queue::{Checker, Look, Step, Strategy, plan};
use tolearn_offline::store::{Checked, Held, Store};

use super::jobs::Job;

pub struct Watched<'a> {
    store: &'a RefCell<Store>,
    probe: &'a dyn Probe,
    at: i64,
    job: &'a Job,
    learned: RefCell<HashMap<String, Checked>>,
}

impl<'a> Watched<'a> {
    pub fn new(store: &'a RefCell<Store>, probe: &'a dyn Probe, at: i64, job: &'a Job) -> Self {
        Self {
            store,
            probe,
            at,
            job,
            learned: RefCell::new(HashMap::new()),
        }
    }

    pub fn stamped(&self, materials: &[Material], saved: &[String]) {
        for material in materials.iter().filter(|material| watchable(material)) {
            if saved.iter().any(|url| url == &material.url) {
                self.mark(&material.url);
            }
        }
    }

    fn mark(&self, url: &str) {
        let known = self.learned.borrow().get(url).cloned();
        let mut store = self.store.borrow_mut();
        let _ = match known {
            Some(checked) => store.stamp(url, &checked),
            None => store.checked(url, self.at),
        };
    }

    fn looked(&self, url: &str, how: Strategy) -> Result<Look, String> {
        if !how.checkable() {
            return Ok(Look::Unchecked);
        }
        let Some(held) = self.held(url) else {
            return Ok(Look::Fetch);
        };
        if recent(&held, self.at) {
            return Ok(Look::Same);
        }
        let kept = kept(held);
        let answer = self
            .probe
            .ask(url, kept.etag.as_deref(), kept.last_modified.as_deref())?;
        let change = weighed(&kept, &answer, self.at);
        if !change.fetch {
            let _ = self.store.borrow_mut().stamp(url, &change.mark);
            return Ok(Look::Same);
        }
        self.learned
            .borrow_mut()
            .insert(url.to_owned(), change.mark);
        Ok(Look::Fetch)
    }

    fn held(&self, url: &str) -> Option<Held> {
        self.store.borrow().held(url).ok().flatten()
    }
}

impl Checker for Watched<'_> {
    fn look(&self, material: &Material, how: Strategy) -> Result<Look, String> {
        let looked = self.looked(&material.url, how);
        if !matches!(looked, Ok(Look::Fetch)) {
            self.job.stepped();
        }
        looked
    }
}

impl std::fmt::Debug for Watched<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Watched").field("at", &self.at).finish()
    }
}

fn watchable(material: &Material) -> bool {
    matches!(plan(material), Step::Take(how) if how.checkable())
}

fn recent(held: &Held, at: i64) -> bool {
    held.checked_at.is_some_and(|checked| at - checked < WINDOW)
}

fn kept(held: Held) -> Kept {
    Kept {
        body_hash: held.body_hash,
        etag: held.etag,
        last_modified: held.last_modified,
    }
}
