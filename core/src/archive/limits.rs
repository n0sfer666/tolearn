use super::error::ArchiveError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    pub entries: u32,
    pub bytes: u64,
    pub ratio: u64,
}

impl Limits {
    pub const DEFAULT: Self = Self {
        entries: 10_000,
        bytes: 256 << 20,
        ratio: 200,
    };
}

#[derive(Debug)]
pub struct Tally<'a> {
    limits: &'a Limits,
    packed: u64,
    entries: u32,
    bytes: u64,
}

impl<'a> Tally<'a> {
    pub fn new(limits: &'a Limits, packed: u64) -> Self {
        Self {
            limits,
            packed,
            entries: 0,
            bytes: 0,
        }
    }

    pub fn entry(&mut self) -> Result<(), ArchiveError> {
        self.entries += 1;
        if self.entries > self.limits.entries {
            return Err(ArchiveError::TooMany {
                entries: self.limits.entries,
            });
        }
        Ok(())
    }

    pub fn grow(&mut self, bytes: u64) -> Result<(), ArchiveError> {
        self.bytes += bytes;
        if self.bytes > self.limits.bytes {
            return Err(ArchiveError::TooBig {
                bytes: self.limits.bytes,
            });
        }
        if self.packed > 0 && self.bytes / self.packed > self.limits.ratio {
            return Err(ArchiveError::TooDense {
                ratio: self.limits.ratio,
            });
        }
        Ok(())
    }
}
