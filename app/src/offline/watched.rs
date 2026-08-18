use std::cell::RefCell;
use std::collections::HashMap;

use tolearn_core::topic::Material;
use tolearn_offline::fresh::{Kept, Probe, weighed};
use tolearn_offline::queue::{Checker, Look, Step, Strategy, plan};
use tolearn_offline::store::{Checked, Held, Store};

use super::jobs::Job;
use super::sight::{Sight, sight};

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
        let held = self.held(url);
        match (sight(how, held.as_ref(), self.at), held) {
            (Sight::Ask, Some(held)) => self.asked(url, held),
            (Sight::Fetch, _) | (Sight::Ask, None) => Ok(Look::Fetch),
            (Sight::Fresh, _) => Ok(Look::Same),
            (Sight::Unchecked, _) => Ok(Look::Unchecked),
        }
    }

    fn asked(&self, url: &str, held: Held) -> Result<Look, String> {
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
        self.job.starting(&material.url);
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

fn kept(held: Held) -> Kept {
    Kept {
        body_hash: held.body_hash,
        etag: held.etag,
        last_modified: held.last_modified,
    }
}
