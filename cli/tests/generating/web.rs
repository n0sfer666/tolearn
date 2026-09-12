use tolearn_offline::page::{PageError, Source};

use super::answers::repository;

const THUMBNAIL: &[u8] = b"\x89PNG thumbnail";
const API: &str = "https://commons.wikimedia.org/w/api.php?";
const OPEN_LIBRARY: &str = "https://openlibrary.org/search.json?";

#[derive(Debug)]
pub struct Web;

impl Source for Web {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        let fixture = match url {
            "https://a.test/article" => "valid/reader/semantic.html",
            "https://a.test/docs" => "valid/reader/docs.html",
            _ if url.starts_with(API) => "commons/found.json",
            _ if url.contains("wikimedia.org") => return Ok(THUMBNAIL.to_vec()),
            _ if url.starts_with(OPEN_LIBRARY) => "openlibrary/isbn.json",
            _ => {
                return Err(PageError::Unreachable(
                    url.to_owned(),
                    "нет такого адреса".to_owned(),
                ));
            }
        };
        Ok(std::fs::read(repository().join("fixtures").join(fixture)).unwrap())
    }
}
