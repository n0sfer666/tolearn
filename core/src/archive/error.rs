use std::fmt;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveError {
    UnknownFormat,
    Damaged { reason: String },
    Unreadable { path: PathBuf, kind: ErrorKind },
    Unwritable { path: PathBuf, kind: ErrorKind },
    Escaping { entry: String },
    Link { entry: String },
    TooMany { entries: u32 },
    TooBig { bytes: u64 },
    TooDense { ratio: u64 },
    NoBundle,
    ManyBundles,
}

impl ArchiveError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownFormat => "archive.unknown-format",
            Self::Damaged { .. } => "archive.damaged",
            Self::Unreadable { .. } => "archive.unreadable",
            Self::Unwritable { .. } => "archive.unwritable",
            Self::Escaping { .. } => "archive.escaping-path",
            Self::Link { .. } => "archive.link",
            Self::TooMany { .. } => "archive.too-many-entries",
            Self::TooBig { .. } => "archive.too-big",
            Self::TooDense { .. } => "archive.too-dense",
            Self::NoBundle => "archive.no-bundle",
            Self::ManyBundles => "archive.many-bundles",
        }
    }
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFormat => write!(
                out,
                "the file is neither a `.zip` nor a `.tar.gz`, and the format of an archive is not a guess"
            ),
            Self::Damaged { reason } => write!(out, "the archive is damaged: {reason}"),
            Self::Unreadable { path, kind } => {
                write!(out, "`{}` cannot be read: {kind}", path.display())
            }
            Self::Unwritable { path, kind } => {
                write!(out, "`{}` cannot be written: {kind}", path.display())
            }
            Self::Escaping { entry } => write!(
                out,
                "`{entry}` leads out of the directory it is unpacked into"
            ),
            Self::Link { entry } => write!(out, "`{entry}` is a link, and a bundle holds no links"),
            Self::TooMany { entries } => {
                write!(out, "the archive holds more than {entries} entries")
            }
            Self::TooBig { bytes } => write!(out, "unpacked, the archive is over {bytes} bytes"),
            Self::TooDense { ratio } => write!(
                out,
                "the archive unpacks to over {ratio} times its own size"
            ),
            Self::NoBundle => write!(out, "the archive holds no `roadmap.yaml`"),
            Self::ManyBundles => write!(
                out,
                "the archive holds several bundles, and which one to import is not a guess"
            ),
        }
    }
}
