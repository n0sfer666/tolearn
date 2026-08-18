use super::error::PageError;

pub trait Prerenderer {
    fn render(&self, url: &str, html: &[u8]) -> Result<Vec<u8>, PageError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AsFetched;

impl Prerenderer for AsFetched {
    fn render(&self, _url: &str, html: &[u8]) -> Result<Vec<u8>, PageError> {
        Ok(html.to_vec())
    }
}
