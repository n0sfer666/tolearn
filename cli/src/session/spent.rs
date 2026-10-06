#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Spent {
    pub calls: u64,
    pub input: u64,
    pub output: u64,
    pub unknown: u64,
}

impl Spent {
    pub fn since(self, before: Self) -> Self {
        Self {
            calls: self.calls.saturating_sub(before.calls),
            input: self.input.saturating_sub(before.input),
            output: self.output.saturating_sub(before.output),
            unknown: self.unknown.saturating_sub(before.unknown),
        }
    }
}
