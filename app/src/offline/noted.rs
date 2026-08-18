use std::cell::RefCell;

use tolearn_offline::page::{PageError, Source};

pub(super) struct Noted<'a> {
    inner: &'a dyn Source,
    url: String,
    seen: RefCell<Option<Vec<u8>>>,
}

impl<'a> Noted<'a> {
    pub(super) fn new(inner: &'a dyn Source, url: &str) -> Self {
        Self {
            inner,
            url: url.to_owned(),
            seen: RefCell::new(None),
        }
    }

    pub(super) fn taken(&self) -> Option<Vec<u8>> {
        self.seen.borrow().clone()
    }
}

impl Source for Noted<'_> {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
        let bytes = self.inner.fetch(url)?;
        if url == self.url && self.seen.borrow().is_none() {
            *self.seen.borrow_mut() = Some(bytes.clone());
        }
        Ok(bytes)
    }
}

impl std::fmt::Debug for Noted<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Noted").field("url", &self.url).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = "<html><body>тело</body></html>";

    struct Site;

    impl Source for Site {
        fn fetch(&self, url: &str) -> Result<Vec<u8>, PageError> {
            match url {
                "https://docs.test/robots.txt" => Ok(b"User-agent: *".to_vec()),
                "https://docs.test/page" => Ok(BODY.as_bytes().to_vec()),
                _ => Err(PageError::Unreachable(url.to_string(), "404".to_string())),
            }
        }
    }

    #[test]
    fn запоминаются_байты_только_запрошенной_страницы() {
        let noted = Noted::new(&Site, "https://docs.test/page");

        assert!(noted.fetch("https://docs.test/robots.txt").is_ok());
        assert_eq!(noted.taken(), None, "запомнился не тот адрес");

        assert!(noted.fetch("https://docs.test/page").is_ok());
        assert_eq!(noted.taken().as_deref(), Some(BODY.as_bytes()));
    }

    #[test]
    fn недоступная_страница_ничего_не_оставляет() {
        let noted = Noted::new(&Site, "https://docs.test/gone");

        assert!(noted.fetch("https://docs.test/gone").is_err());
        assert_eq!(noted.taken(), None);
    }
}
