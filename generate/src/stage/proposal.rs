use serde::Deserialize;

use crate::object::object;

#[derive(Debug, Default, Deserialize)]
pub(super) struct Proposal {
    #[serde(default)]
    pub books: Vec<BookWish>,
    #[serde(default)]
    pub pages: Vec<PageWish>,
    #[serde(default)]
    pub images: Vec<ImageWish>,
}

#[derive(Debug, Deserialize)]
pub(super) struct BookWish {
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub isbn: Option<String>,
    #[serde(default)]
    pub chapter: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct PageWish {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ImageWish {
    pub query: String,
    #[serde(default)]
    pub caption: String,
}

pub(super) fn read(text: &str) -> Result<Proposal, String> {
    serde_json::from_str(object(text)?).map_err(|error| error.to_string())
}
