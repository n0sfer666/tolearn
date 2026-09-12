use std::path::PathBuf;

use tolearn_core::program::{self, Program};
use tolearn_generate::online;
use tolearn_generate::sources::Sources;
use tolearn_generate::stage::{self, Gathered, Place};
use tolearn_offline::page::{AsFetched, PageError, Source};
use tolearn_offline::store::Store;
use tolearn_provider::Stop;

use crate::support::{Scripted, Up};

pub const ARTICLE: &str = "https://a.test/article";
pub const DOCS: &str = "https://a.test/docs";
pub const MENU: &str = "https://a.test/menu";
pub const TAIL: &str = "КОНЕЦ-СТРАНИЦЫ";
const LONG: &str = "https://a.test/long/";
const API: &str = "https://commons.wikimedia.org/w/api.php?";
const OPEN_LIBRARY: &str = "https://openlibrary.org/search.json?";
const UNKNOWN: &str = "Unwritten";
const PROGRAM: &str = "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84";
const BUDGET: u64 = 16 * 1024 * 1024;

pub struct Web;

impl Source for Web {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        let fixture = match url {
            ARTICLE => "valid/reader/semantic.html",
            DOCS => "valid/reader/docs.html",
            MENU => "valid/reader/menu-only.html",
            _ if url.starts_with(LONG) => return Ok(long(url).into_bytes()),
            _ if url.starts_with(API) => "commons/found.json",
            _ if url.contains("wikimedia.org") => return Ok(b"\x89PNG thumbnail".to_vec()),
            _ if url.starts_with(OPEN_LIBRARY) && url.contains(UNKNOWN) => "openlibrary/none.json",
            _ if url.starts_with(OPEN_LIBRARY) => "openlibrary/isbn.json",
            _ => {
                return Err(PageError::Unreachable(
                    url.to_owned(),
                    "нет такого адреса".to_owned(),
                ));
            }
        };
        Ok(std::fs::read(format!("../fixtures/{fixture}")).unwrap())
    }
}

fn long(url: &str) -> String {
    let paragraph = format!(
        "<p>{}</p>",
        "Импульсный канал задаёт высоту тона делителем частоты. ".repeat(20)
    );
    format!(
        "<html><head><title>{url}</title></head><body><article><h1>Длинная страница</h1>{}<p>{TAIL}</p></article></body></html>",
        paragraph.repeat(30)
    )
}

pub struct Bench {
    pub dir: PathBuf,
    pub store: Store,
}

impl Bench {
    pub fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("tolearn-stage-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Store::open(&dir.join("cache"), BUDGET).unwrap();
        Self { dir, store }
    }

    fn sources(&mut self) -> Sources<'_> {
        Sources::new(&Web, &AsFetched, &mut self.store, &self.dir, PROGRAM, 1_000)
    }
}

impl Drop for Bench {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

pub fn chiptune() -> Program {
    program::parse(&std::fs::read_to_string("../examples/chiptune/program.yaml").unwrap()).unwrap()
}

pub fn scripted(answers: &[&str]) -> Scripted {
    Scripted::new(
        answers
            .iter()
            .map(|name| {
                if name.ends_with(".txt") {
                    std::fs::read_to_string(format!("../fixtures/generate/stage/{name}")).unwrap()
                } else {
                    (*name).to_owned()
                }
            })
            .collect(),
    )
}

pub fn gathered(bench: &mut Bench, program: &Program, answers: &[&str]) -> (Gathered, Vec<String>) {
    let model = scripted(answers);
    let place = Place::find(program, "voices").unwrap();
    let gathered = stage::gather(
        &online(&Up, &model).unwrap(),
        &mut bench.sources(),
        &place,
        &Stop::default(),
    )
    .unwrap();
    (gathered, model.prompts())
}

pub fn many() -> String {
    let books: Vec<String> = ["9780262033787", "9780262543231", "9780070004849"]
        .iter()
        .map(|isbn| format!(r#"{{"title": "Книга {isbn}", "author": "Автор", "isbn": "{isbn}", "chapter": "Глава 1"}}"#))
        .collect();
    let pages: Vec<String> = (1..=4)
        .map(|index| format!(r#"{{"url": "{LONG}{index}"}}"#))
        .collect();
    let images: Vec<String> = ["pulse", "triangle", "noise"]
        .iter()
        .map(|query| format!(r#"{{"query": "{query}", "caption": "Осциллограмма: {query}"}}"#))
        .collect();
    format!(
        r#"{{"books": [{}], "pages": [{}], "images": [{}]}}"#,
        books.join(", "),
        pages.join(", "),
        images.join(", ")
    )
}
