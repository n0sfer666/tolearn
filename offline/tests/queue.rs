#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "offline gate: a panic here is the report"
)]

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tolearn_core::topic::{Liveness, Material, MaterialTier, MaterialType};
use tolearn_offline::queue::{Saver, Skip, Strategy, again, unload};

fn material(url: &str, kind: MaterialType, liveness: Liveness) -> Material {
    Material {
        title: url.to_string(),
        url: url.to_string(),
        kind,
        tier: MaterialTier::T1,
        lang: "ru".to_string(),
        liveness,
        published: None,
        covers_version: None,
        checked_at: "2026-07-28".to_string(),
        stale: false,
        delta: None,
        note: String::new(),
    }
}

fn ok(url: &str) -> Material {
    material(url, MaterialType::Article, Liveness::Ok)
}

struct Fake {
    broken: Vec<String>,
    taken: Mutex<Vec<String>>,
    how: Mutex<Vec<Strategy>>,
    stop: Option<(usize, &'static AtomicBool)>,
}

impl Fake {
    fn new(broken: &[&str]) -> Self {
        Self {
            broken: broken.iter().map(|url| (*url).to_string()).collect(),
            taken: Mutex::new(Vec::new()),
            how: Mutex::new(Vec::new()),
            stop: None,
        }
    }

    fn taken(&self) -> Vec<String> {
        self.taken.lock().unwrap().clone()
    }
}

impl Saver for Fake {
    fn save(&self, material: &Material, how: Strategy) -> Result<u64, String> {
        self.taken.lock().unwrap().push(material.url.clone());
        self.how.lock().unwrap().push(how);
        if let Some((after, stop)) = self.stop
            && self.taken.lock().unwrap().len() >= after
        {
            stop.store(true, Ordering::Relaxed);
        }
        if self.broken.contains(&material.url) {
            return Err("таймаут".to_string());
        }
        Ok(1024)
    }
}

#[test]
fn выгружаются_все_материалы_программы() {
    let materials = vec![ok("https://a.test"), ok("https://b.test")];
    let saver = Fake::new(&[]);

    let report = unload(&materials, &saver, &AtomicBool::new(false));

    assert_eq!(report.saved.len(), 2);
    assert_eq!(saver.taken(), vec!["https://a.test", "https://b.test"]);
    assert_eq!(report.bytes, 2048, "объём выгруженного не сложен");
    assert!(!report.cancelled);
}

#[test]
fn платный_и_закрытый_логином_пропускаются_до_попытки() {
    let materials = vec![
        material(
            "https://paid.test",
            MaterialType::Article,
            Liveness::Paywall,
        ),
        material(
            "https://login.test",
            MaterialType::Docs,
            Liveness::LoginRequired,
        ),
        ok("https://free.test"),
    ];
    let saver = Fake::new(&[]);

    let report = unload(&materials, &saver, &AtomicBool::new(false));

    assert_eq!(saver.taken(), vec!["https://free.test"], "тянули закрытое");
    assert_eq!(
        report
            .skipped
            .iter()
            .map(|(url, why)| (url.as_str(), *why))
            .collect::<Vec<_>>(),
        vec![
            ("https://paid.test", Skip::Paywall),
            ("https://login.test", Skip::LoginRequired),
        ]
    );
}

#[test]
fn провал_попадает_в_отчёт_с_причиной() {
    let materials = vec![ok("https://a.test"), ok("https://bad.test")];
    let saver = Fake::new(&["https://bad.test"]);

    let report = unload(&materials, &saver, &AtomicBool::new(false));

    assert_eq!(report.saved.len(), 1);
    assert_eq!(
        report.failed,
        vec![("https://bad.test".to_string(), "таймаут".to_string())]
    );
}

#[test]
fn отчёт_разбит_по_причинам() {
    let materials = vec![
        material(
            "https://paid.test",
            MaterialType::Article,
            Liveness::Paywall,
        ),
        material(
            "https://paid2.test",
            MaterialType::Article,
            Liveness::Paywall,
        ),
        material(
            "https://login.test",
            MaterialType::Docs,
            Liveness::LoginRequired,
        ),
        ok("https://a.test"),
        ok("https://bad.test"),
    ];
    let saver = Fake::new(&["https://bad.test"]);

    let report = unload(&materials, &saver, &AtomicBool::new(false));
    let shown = report.to_string();

    assert!(shown.contains("Скачано     1"), "{shown}");
    assert!(
        shown.contains("paywall (2)") && shown.contains("login_required (1)"),
        "разбивки пропусков нет: {shown}"
    );
    assert!(
        shown.contains("таймаут (1)"),
        "разбивки провалов нет: {shown}"
    );
}

#[test]
fn отмена_останавливает_очередь() {
    static STOP: AtomicBool = AtomicBool::new(false);
    STOP.store(false, Ordering::Relaxed);
    let materials = vec![
        ok("https://a.test"),
        ok("https://b.test"),
        ok("https://c.test"),
    ];
    let mut saver = Fake::new(&[]);
    saver.stop = Some((1, &STOP));

    let report = unload(&materials, &saver, &STOP);

    assert!(report.cancelled, "отмена не отмечена в отчёте");
    assert_eq!(saver.taken(), vec!["https://a.test"], "очередь не встала");
    assert_eq!(report.saved.len(), 1);
    assert!(report.failed.is_empty(), "прерванное записано как провал");
}

#[test]
fn повтор_берёт_только_провалившиеся() {
    let materials = vec![
        ok("https://a.test"),
        ok("https://bad.test"),
        material(
            "https://paid.test",
            MaterialType::Article,
            Liveness::Paywall,
        ),
    ];
    let first = unload(
        &materials,
        &Fake::new(&["https://bad.test"]),
        &AtomicBool::new(false),
    );
    let saver = Fake::new(&[]);

    let second = again(&materials, &first, &saver, &AtomicBool::new(false));

    assert_eq!(saver.taken(), vec!["https://bad.test"], "повторили лишнее");
    assert_eq!(second.saved.len(), 1);
    assert!(second.failed.is_empty());
}

#[test]
fn повторять_нечего_если_всё_прошло() {
    let materials = vec![ok("https://a.test")];
    let first = unload(&materials, &Fake::new(&[]), &AtomicBool::new(false));
    let saver = Fake::new(&[]);

    let second = again(&materials, &first, &saver, &AtomicBool::new(false));

    assert!(saver.taken().is_empty(), "повтор тронул успешные");
    assert!(second.saved.is_empty() && second.failed.is_empty());
}

#[test]
fn способ_сохранения_выбран_по_типу_материала() {
    let materials = vec![
        material("https://docs.test", MaterialType::Docs, Liveness::Ok),
        material("https://repo.test", MaterialType::Repo, Liveness::Ok),
        material("https://video.test", MaterialType::Video, Liveness::Ok),
        material("https://paper.test", MaterialType::Paper, Liveness::Ok),
    ];
    let saver = Fake::new(&[]);

    unload(&materials, &saver, &AtomicBool::new(false));

    assert_eq!(
        saver.how.lock().unwrap().clone(),
        vec![
            Strategy::Mirror,
            Strategy::Clone,
            Strategy::Video,
            Strategy::Direct
        ]
    );
}
