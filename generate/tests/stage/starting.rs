use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tolearn_generate::diagram::Painter;
use tolearn_generate::plan::{self, Plan};
use tolearn_generate::start::{self, Kit};
use tolearn_generate::{GenerateError, Model, Online, Progress, Step, online};
use tolearn_offline::page::AsFetched;
use tolearn_provider::{CheckError, Said, Stop};

use crate::answers::fine;
use crate::support::{Scripted, Up, answer, model, request};
use crate::web::{Bench, Web, scripted};

pub const SVG: &str =
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><text>Схема</text></svg>"#;
const STORE: [&str; 2] = ["objects", "artifacts"];

pub struct Canvas(pub bool);

impl Painter for Canvas {
    fn paint(&self, _mermaid: &str) -> Result<String, String> {
        if self.0 {
            Ok(SVG.to_owned())
        } else {
            Err("окно схемы закрыто".to_owned())
        }
    }
}

#[derive(Default)]
pub struct Recorder {
    pub stop: Stop,
    halt: Option<(&'static str, Step)>,
    pressed: Cell<Option<bool>>,
    heard: RefCell<Vec<(&'static str, Step)>>,
}

impl Recorder {
    pub fn halting(step: Step) -> Self {
        Self {
            halt: Some(("began", step)),
            ..Self::default()
        }
    }

    pub fn late(step: Step) -> Self {
        Self {
            halt: Some(("ended", step)),
            ..Self::default()
        }
    }

    pub fn pressed(&self) -> Option<bool> {
        self.pressed.get()
    }

    pub fn heard(&self) -> Vec<(&'static str, Step)> {
        self.heard.borrow().clone()
    }

    fn note(&self, state: &'static str, step: Step) {
        self.heard.borrow_mut().push((state, step));
        if self.halt == Some((state, step)) {
            self.pressed.set(Some(self.stop.stop()));
        }
    }
}

impl Progress for Recorder {
    fn began(&self, step: Step) {
        self.note("began", step);
    }

    fn ended(&self, step: Step) {
        self.note("ended", step);
    }
}

#[derive(Debug)]
pub struct Heeding<'a> {
    pub script: Scripted,
    pub stop: &'a Stop,
}

impl Model for Heeding<'_> {
    fn ask(&self, prompt: &str) -> Result<Said, CheckError> {
        if self.stop.stopped() {
            return Err(CheckError::Cancelled);
        }
        self.script.ask(prompt)
    }
}

pub fn flat() -> Scripted {
    scripted(&["sources.txt", &fine().to_string()])
}

pub fn split() -> Scripted {
    scripted(&[&answer("part.txt"), "sources.txt", &fine().to_string()])
}

pub fn drawn(name: &str) -> Plan {
    plan::plan(&online(&Up, &model(&[name])).unwrap(), &request()).unwrap()
}

pub fn run(
    bench: &mut Bench,
    plan: &Plan,
    model: &dyn Model,
    painter: &dyn Painter,
    recorder: &Recorder,
) -> Result<String, GenerateError> {
    started(bench, plan, &online(&Up, model).unwrap(), painter, recorder)
}

pub fn started(
    bench: &mut Bench,
    plan: &Plan,
    online: &Online<'_>,
    painter: &dyn Painter,
    recorder: &Recorder,
) -> Result<String, GenerateError> {
    let kit = Kit {
        online,
        source: &Web,
        renderer: &AsFetched,
        store: &mut bench.store,
        painter,
        progress: recorder,
        stop: &recorder.stop,
        data: &bench.dir,
        at: 1_000,
    };
    start::start(kit, &request(), plan)
}

pub fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut open = vec![root.to_path_buf()];
    while let Some(folder) = open.pop() {
        let Ok(listing) = std::fs::read_dir(&folder) else {
            continue;
        };
        for item in listing {
            let path = item.unwrap().path();
            if path.is_dir() {
                open.push(path);
            } else {
                let name = path.strip_prefix(root).unwrap().to_path_buf();
                found.insert(name, std::fs::read(&path).unwrap());
            }
        }
    }
    found
}

pub fn names(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .map(|listing| {
            listing
                .map(|item| item.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

pub fn leftovers(bench: &Bench) -> Vec<String> {
    let cache = bench.dir.join("cache");
    names(&cache)
        .into_iter()
        .filter(|name| cache.join(name).is_dir() && !STORE.contains(&name.as_str()))
        .collect()
}

pub fn untouched(bench: &Bench, before: &BTreeMap<PathBuf, Vec<u8>>) {
    assert_eq!(files(&bench.dir.join("programs")), *before);
    assert_eq!(leftovers(bench), Vec::<String>::new());
}

pub fn paired(heard: &[(&'static str, Step)]) {
    assert_eq!(heard.len() % 2, 0, "{heard:?}");
    for pair in heard.chunks(2) {
        assert_eq!(pair[0].0, "began", "{heard:?}");
        assert_eq!(pair[1], ("ended", pair[0].1), "{heard:?}");
    }
}
