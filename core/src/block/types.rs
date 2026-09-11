#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Paragraph,
    Heading,
    Code,
    Callout,
    Diagram,
    Image,
}

impl Kind {
    pub const ALL: [Self; 6] = [
        Self::Paragraph,
        Self::Heading,
        Self::Code,
        Self::Callout,
        Self::Diagram,
        Self::Image,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Heading => "heading",
            Self::Code => "code",
            Self::Callout => "callout",
            Self::Diagram => "diagram",
            Self::Image => "image",
        }
    }

    pub fn fields(self) -> &'static [&'static str] {
        match self {
            Self::Paragraph | Self::Heading | Self::Callout => &[],
            Self::Code => &["lang"],
            Self::Diagram => &["asset"],
            Self::Image => &["asset", "license", "attribution", "source"],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: String,
    pub kind: Kind,
    pub text: String,
    pub lang: Option<String>,
    pub asset: Option<String>,
    pub license: Option<String>,
    pub attribution: Option<String>,
    pub source: Option<String>,
}

impl Block {
    pub fn extras(&self) -> [(&'static str, Option<&str>); 5] {
        [
            ("lang", self.lang.as_deref()),
            ("asset", self.asset.as_deref()),
            ("license", self.license.as_deref()),
            ("attribution", self.attribution.as_deref()),
            ("source", self.source.as_deref()),
        ]
    }
}
