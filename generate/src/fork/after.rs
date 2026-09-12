#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct After<'a> {
    pub program: &'a str,
    pub node: &'a str,
    pub stage: &'a str,
}
