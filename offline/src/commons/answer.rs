use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Answer {
    pub(super) query: Option<Query>,
}

#[derive(Deserialize)]
pub(super) struct Query {
    pub(super) pages: Vec<File>,
}

#[derive(Deserialize)]
pub(super) struct File {
    pub(super) index: u32,
    pub(super) title: String,
    #[serde(default)]
    pub(super) imageinfo: Vec<Info>,
}

#[derive(Deserialize)]
pub(super) struct Info {
    pub(super) mime: String,
    pub(super) thumburl: Option<String>,
    pub(super) descriptionurl: String,
    #[serde(default)]
    pub(super) extmetadata: Meta,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Meta {
    pub(super) license: Option<Field>,
    pub(super) license_short_name: Option<Field>,
    pub(super) artist: Option<Field>,
}

#[derive(Deserialize)]
pub(super) struct Field {
    pub(super) value: String,
}
