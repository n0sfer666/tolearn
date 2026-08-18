#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Heading,
    Paragraph,
    Item,
    Quote,
    Code,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Heading => "heading",
            Self::Paragraph => "paragraph",
            Self::Item => "item",
            Self::Quote => "quote",
            Self::Code => "code",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: Kind,
    pub level: u32,
    pub text: String,
}

impl Block {
    pub(super) fn new(kind: Kind, level: u32, text: String) -> Self {
        Self { kind, level, text }
    }
}
