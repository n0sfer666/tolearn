mod answer;
mod candidate;
mod error;
mod license;
mod text;

pub use error::CommonsError;
pub use license::free;

use url::form_urlencoded::Serializer;

use crate::page::Source;
use answer::Answer;
use candidate::Candidate;

const API: &str = "https://commons.wikimedia.org/w/api.php";
const RESULTS: &str = "5";
const METADATA: &str = "License|LicenseShortName|Artist";
pub const THUMB_WIDTH: u32 = 960;
pub const MAX_BYTES: usize = 400 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    pub title: String,
    pub page: String,
    pub license: String,
    pub author: String,
    pub extension: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    Picture(Picture),
    Refused(String),
    Missing,
}

pub fn find(source: &dyn Source, query: &str) -> Result<Found, CommonsError> {
    let bytes = source
        .fetch(&search(query))
        .map_err(CommonsError::Unreachable)?;
    let answer: Answer = serde_json::from_slice(&bytes)
        .map_err(|error| CommonsError::Malformed(error.to_string()))?;
    let mut files = answer.query.map(|query| query.pages).unwrap_or_default();
    files.sort_by_key(|file| file.index);
    let images: Vec<Candidate> = files.into_iter().filter_map(Candidate::from).collect();
    let Some(first) = images.first() else {
        return Ok(Found::Missing);
    };
    let Some(chosen) = images.iter().find(|image| image.usable()) else {
        return Ok(Found::Refused(first.refusal()));
    };
    let bytes = source
        .fetch(&chosen.thumb)
        .map_err(CommonsError::Unreachable)?;
    if bytes.len() > MAX_BYTES {
        return Ok(Found::Refused(format!(
            "миниатюра {} весит {} КБ, а предел {} КБ",
            chosen.title,
            bytes.len().div_ceil(1024),
            MAX_BYTES / 1024
        )));
    }
    Ok(Found::Picture(Picture {
        title: chosen.title.clone(),
        page: chosen.page.clone(),
        license: chosen.license.clone(),
        author: chosen.author.clone(),
        extension: chosen.extension.clone(),
        bytes,
    }))
}

fn search(query: &str) -> String {
    let width = THUMB_WIDTH.to_string();
    let params = Serializer::new(String::new())
        .append_pair("action", "query")
        .append_pair("format", "json")
        .append_pair("formatversion", "2")
        .append_pair("generator", "search")
        .append_pair("gsrnamespace", "6")
        .append_pair("gsrsearch", query)
        .append_pair("gsrlimit", RESULTS)
        .append_pair("prop", "imageinfo")
        .append_pair("iiprop", "url|mime|extmetadata")
        .append_pair("iiurlwidth", &width)
        .append_pair("iiextmetadatafilter", METADATA)
        .finish();
    format!("{API}?{params}")
}
