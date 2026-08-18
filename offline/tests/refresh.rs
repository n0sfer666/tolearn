#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use tolearn_core::topic::{Liveness, Material, MaterialTier, MaterialType};
use tolearn_offline::queue::{Checker, Look, Saver, Skip, Strategy, refresh};

fn material(url: &str, kind: MaterialType) -> Material {
    Material {
        title: url.to_string(),
        url: url.to_string(),
        kind,
        tier: MaterialTier::T1,
        lang: "ru".to_string(),
        liveness: Liveness::Ok,
        published: None,
        covers_version: None,
        checked_at: "2026-07-28".to_string(),
        stale: false,
        delta: None,
        note: String::new(),
    }
}

struct Fake {
    changed: Vec<String>,
    broken: Vec<String>,
    taken: Mutex<Vec<String>>,
    asked: Mutex<Vec<String>>,
}

impl Fake {
    fn new(changed: &[&str], broken: &[&str]) -> Self {
        Self {
            changed: changed.iter().map(|url| (*url).to_string()).collect(),
            broken: broken.iter().map(|url| (*url).to_string()).collect(),
            taken: Mutex::new(Vec::new()),
            asked: Mutex::new(Vec::new()),
        }
    }

    fn taken(&self) -> Vec<String> {
        self.taken.lock().unwrap().clone()
    }

    fn asked(&self) -> Vec<String> {
        self.asked.lock().unwrap().clone()
    }
}

impl Checker for Fake {
    fn look(&self, material: &Material, how: Strategy) -> Result<Look, String> {
        if !how.checkable() {
            return Ok(Look::Unchecked);
        }
        self.asked.lock().unwrap().push(material.url.clone());
        if self.broken.contains(&material.url) {
            return Err("сеть молчит".to_string());
        }
        match self.changed.contains(&material.url) {
            true => Ok(Look::Fetch),
            false => Ok(Look::Same),
        }
    }
}

impl Saver for Fake {
    fn save(&self, material: &Material, _how: Strategy) -> Result<u64, String> {
        self.taken.lock().unwrap().push(material.url.clone());
        Ok(2048)
    }
}

#[test]
fn перекачивается_только_изменившееся() {
    let materials = vec![
        material("https://same.test", MaterialType::Article),
        material("https://moved.test", MaterialType::Docs),
    ];
    let fake = Fake::new(&["https://moved.test"], &[]);

    let report = refresh(&materials, &fake, &fake, &AtomicBool::new(false));

    assert_eq!(fake.taken(), vec!["https://moved.test"], "тронули лишнее");
    assert_eq!(report.saved, vec!["https://moved.test".to_string()]);
    assert_eq!(
        report.skipped,
        vec![("https://same.test".to_string(), Skip::Unchanged)]
    );
    assert_eq!(report.bytes, 2048);
}

#[test]
fn клон_и_видео_не_проверяются_и_не_качаются() {
    let materials = vec![
        material("https://repo.test", MaterialType::Repo),
        material("https://video.test", MaterialType::Video),
    ];
    let fake = Fake::new(&[], &[]);

    let report = refresh(&materials, &fake, &fake, &AtomicBool::new(false));

    assert!(fake.asked().is_empty(), "непроверяемое пошли проверять");
    assert!(fake.taken().is_empty(), "непроверяемое пошли качать");
    assert_eq!(
        report.skipped,
        vec![
            ("https://repo.test".to_string(), Skip::Unchecked),
            ("https://video.test".to_string(), Skip::Unchecked),
        ]
    );
}

#[test]
fn провал_проверки_попадает_в_отчёт_и_не_качает() {
    let materials = vec![material("https://bad.test", MaterialType::Article)];
    let fake = Fake::new(&["https://bad.test"], &["https://bad.test"]);

    let report = refresh(&materials, &fake, &fake, &AtomicBool::new(false));

    assert!(fake.taken().is_empty(), "качали после провала проверки");
    assert_eq!(
        report.failed,
        vec![("https://bad.test".to_string(), "сеть молчит".to_string())]
    );
}

#[test]
fn закрытое_логином_до_проверки_не_доходит() {
    let mut closed = material("https://login.test", MaterialType::Docs);
    closed.liveness = Liveness::LoginRequired;
    let fake = Fake::new(&[], &[]);

    let report = refresh(&[closed], &fake, &fake, &AtomicBool::new(false));

    assert!(fake.asked().is_empty(), "закрытое пошли проверять");
    assert_eq!(
        report.skipped,
        vec![("https://login.test".to_string(), Skip::LoginRequired)]
    );
}

#[test]
fn отмена_останавливает_обновление() {
    let materials = vec![
        material("https://a.test", MaterialType::Article),
        material("https://b.test", MaterialType::Article),
    ];
    let fake = Fake::new(&[], &[]);

    let report = refresh(&materials, &fake, &fake, &AtomicBool::new(true));

    assert!(report.cancelled);
    assert!(fake.asked().is_empty(), "очередь не встала");
}
