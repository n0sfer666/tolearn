use super::answer::{Field, File};
use super::license::free;
use super::text::plain;

const EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

pub(super) struct Candidate {
    pub(super) title: String,
    pub(super) page: String,
    pub(super) code: String,
    pub(super) license: String,
    pub(super) author: String,
    pub(super) thumb: String,
    pub(super) extension: String,
}

impl Candidate {
    pub(super) fn from(file: File) -> Option<Self> {
        let info = file.imageinfo.into_iter().next()?;
        if !info.mime.starts_with("image/") {
            return None;
        }
        let thumb = info.thumburl?;
        let extension = extension(&thumb)?;
        let meta = info.extmetadata;
        let code = value(meta.license);
        let license = meta
            .license_short_name
            .map_or_else(|| code.clone(), |field| field.value);
        Some(Self {
            title: file.title,
            page: info.descriptionurl,
            author: plain(&value(meta.artist)),
            code,
            license,
            thumb,
            extension,
        })
    }

    pub(super) fn usable(&self) -> bool {
        free(&self.code) && !self.author.is_empty()
    }

    pub(super) fn refusal(&self) -> String {
        if free(&self.code) {
            format!("у файла {} на Commons не указан автор", self.title)
        } else {
            format!(
                "лицензия {} файла {} не из свободных (CC0, PD, CC BY, CC BY-SA)",
                self.license, self.title
            )
        }
    }
}

fn value(field: Option<Field>) -> String {
    field.map(|field| field.value).unwrap_or_default()
}

fn extension(thumb: &str) -> Option<String> {
    let path = thumb.split('?').next()?;
    let name = path.rsplit('/').next()?;
    let extension = name.rsplit_once('.')?.1.to_ascii_lowercase();
    EXTENSIONS
        .contains(&extension.as_str())
        .then_some(extension)
}
