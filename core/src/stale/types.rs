#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Digest {
    pub topics: Vec<Expired>,
    pub materials: Vec<Aging>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expired {
    pub topic: String,
    pub title: String,
    pub verified_at: String,
    pub expired_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aging {
    pub topic: String,
    pub topic_title: String,
    pub title: String,
    pub url: String,
    pub stale: bool,
    pub delta: Option<String>,
    pub covers_version: Option<String>,
    pub pin: Pin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pin {
    Absent,
    Known,
    Unknown,
}

impl Pin {
    pub fn label(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Known => "known",
            Self::Unknown => "unknown",
        }
    }
}
