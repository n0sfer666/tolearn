mod error;
mod prerender;
mod seal;
mod source;

pub use error::PageError;
pub use prerender::{AsFetched, Prerenderer};
pub use source::{Source, Web};

use monolith::core::{Options, create_monolithic_document_from_data};

#[derive(Debug, Clone)]
pub struct Fetching {
    pub timeout: u64,
    pub domains: Option<Vec<String>>,
}

impl Default for Fetching {
    fn default() -> Self {
        Self {
            timeout: 30,
            domains: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub url: String,
    pub html: Vec<u8>,
    pub title: Option<String>,
}

pub fn save(
    url: &str,
    source: &dyn Source,
    renderer: &dyn Prerenderer,
    fetching: &Fetching,
) -> Result<Page, PageError> {
    let fetched = source.fetch(url)?;
    let rendered = renderer.render(url, &fetched)?;
    let options = Options {
        domains: fetching.domains.clone(),
        ignore_errors: true,
        isolate: true,
        no_frames: true,
        no_js: true,
        silent: true,
        timeout: fetching.timeout,
        ..Options::default()
    };
    let (html, title) = create_monolithic_document_from_data(
        rendered,
        &options,
        &mut None,
        None,
        Some(url.to_string()),
    )
    .map_err(|error| PageError::Unreadable(error.to_string()))?;
    Ok(Page {
        url: url.to_string(),
        html: seal::seal(html),
        title,
    })
}
