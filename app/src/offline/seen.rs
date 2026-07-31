use std::path::Path;

use tolearn_offline::store::{Held, Store};

const ABSENT: &str = "absent";
const SAVED: &str = "saved";

#[derive(Debug, Default)]
pub struct Seen {
    store: Option<Store>,
}

pub fn seen(root: &Path, budget: u64) -> Seen {
    if !root.join("index.sqlite").is_file() {
        return Seen::default();
    }
    Seen {
        store: Store::open(root, budget).ok(),
    }
}

pub fn label(held: Option<&Held>) -> &'static str {
    match held {
        Some(_) => SAVED,
        None => ABSENT,
    }
}

impl Seen {
    pub fn held(&self, url: &str) -> Option<Held> {
        self.store.as_ref()?.held(url).ok().flatten()
    }

    pub fn opened(&mut self, url: &str, at: i64) -> Option<Held> {
        self.store.as_mut()?.get(url, at).ok().flatten()
    }
}
