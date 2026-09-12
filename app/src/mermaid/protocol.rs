use std::borrow::Cow;

use tauri::http::{Request, Response, StatusCode, header};
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Runtime, Url};

use super::page;

pub const SCHEME: &str = "tolearn-mermaid";
pub const POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'unsafe-inline'";

#[cfg(windows)]
const BASE: &str = "http://tolearn-mermaid.localhost/";
#[cfg(not(windows))]
const BASE: &str = "tolearn-mermaid://localhost/";

const HTML: &str = "text/html; charset=utf-8";
const SCRIPT: &str = "text/javascript; charset=utf-8";

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mermaid")
        .register_uri_scheme_protocol(SCHEME, |_, request: Request<Vec<u8>>| {
            respond(request.uri().path())
        })
        .build()
}

pub fn respond(path: &str) -> Response<Cow<'static, [u8]>> {
    let (status, kind, body) = match path {
        "/" | "/index.html" => (StatusCode::OK, HTML, page::INDEX.as_bytes()),
        "/draw.js" => (StatusCode::OK, SCRIPT, page::DRAW.as_bytes()),
        "/mermaid.tiny.js" => (StatusCode::OK, SCRIPT, page::MERMAID),
        _ => (StatusCode::NOT_FOUND, HTML, [].as_slice()),
    };
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, kind)
        .header(header::CONTENT_SECURITY_POLICY, POLICY)
        .body(Cow::Borrowed(body))
        .unwrap_or_default()
}

pub(super) fn address(mermaid: &str) -> Result<Url, String> {
    Url::parse(&format!("{BASE}index.html#{}", encoded(mermaid))).map_err(|error| error.to_string())
}

pub(super) fn inside(url: &Url) -> bool {
    url.as_str().starts_with(BASE)
}

fn encoded(text: &str) -> String {
    text.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}
