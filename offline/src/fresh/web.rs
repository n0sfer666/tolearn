use std::io::Read;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{ETAG, HeaderName, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::{StatusCode, header};

use super::{Answer, Probe};

const CAP: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Conditional {
    client: Client,
}

impl Conditional {
    pub fn new(timeout: u64) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self { client })
    }
}

impl Probe for Conditional {
    fn ask(
        &self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<Answer, String> {
        let mut request = self
            .client
            .get(url)
            .header(header::ACCEPT, "text/html, */*");
        if let Some(etag) = etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(last_modified) = last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_modified);
        }
        let response = request.send().map_err(|error| error.to_string())?;
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(Answer::Same);
        }
        if !response.status().is_success() {
            return Err(format!("ответ {}", response.status().as_u16()));
        }
        let etag = told(&response, ETAG);
        let last_modified = told(&response, LAST_MODIFIED);
        let Some(bytes) = capped(response)? else {
            return Ok(Answer::Big {
                etag,
                last_modified,
            });
        };
        Ok(Answer::Sent {
            bytes,
            etag,
            last_modified,
        })
    }
}

fn capped(response: Response) -> Result<Option<Vec<u8>>, String> {
    let mut bytes = Vec::new();
    response
        .take(CAP + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    match bytes.len() as u64 > CAP {
        true => Ok(None),
        false => Ok(Some(bytes)),
    }
}

fn told(response: &Response, name: HeaderName) -> Option<String> {
    response
        .headers()
        .get(name)?
        .to_str()
        .ok()
        .map(str::to_owned)
}
