use tolearn_core::block::{self, Block, Kind};
use tolearn_offline::commons::Picture;
use tolearn_offline::digest::digest;

const NAME_CHARS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Illustration {
    pub file: String,
    pub bytes: Vec<u8>,
    pub block: Block,
}

impl Illustration {
    pub fn new(picture: Picture, caption: &str) -> Self {
        let hash = digest(&picture.bytes);
        let file = format!("{}.{}", &hash[..NAME_CHARS], picture.extension);
        let block = Block {
            id: block::id(caption),
            kind: Kind::Image,
            text: caption.to_owned(),
            lang: None,
            asset: Some(format!("assets/{file}")),
            attribution: Some(format!(
                "{}, {}, Wikimedia Commons",
                picture.author, picture.license
            )),
            license: Some(picture.license),
            source: Some(picture.page),
        };
        Self {
            file,
            bytes: picture.bytes,
            block,
        }
    }
}
