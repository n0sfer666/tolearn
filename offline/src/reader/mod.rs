mod local;
mod pieces;

pub use local::inlined;
pub use pieces::{Kind, Piece, pieces};

use dom_smoothie::{Article, Config, Readability, TextMode};

const ENOUGH: usize = 500;

#[derive(Debug, Clone)]
pub struct Reading {
    pub title: Option<String>,
    pub html: String,
    pub text: String,
    pub extracted: bool,
}

pub fn read(archive: &str, url: &str) -> Reading {
    match extract(archive, url) {
        Some(article) => Reading {
            title: title(&article),
            html: article.content.to_string(),
            text: article.text_content.to_string(),
            extracted: true,
        },
        None => Reading {
            title: None,
            html: archive.to_string(),
            text: String::new(),
            extracted: false,
        },
    }
}

fn extract(archive: &str, url: &str) -> Option<Article> {
    let config = Config {
        text_mode: TextMode::Formatted,
        ..Config::default()
    };
    let mut readability = Readability::new(archive, Some(url), Some(config)).ok()?;
    let article = readability.parse().ok()?;
    if article.text_content.trim().chars().count() < ENOUGH {
        return None;
    }
    Some(article)
}

fn title(article: &Article) -> Option<String> {
    let title = article.title.trim();
    if title.is_empty() {
        return None;
    }
    Some(title.to_string())
}
