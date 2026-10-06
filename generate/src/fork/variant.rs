use tolearn_core::program::StageRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    pub row: StageRow,
    pub why: String,
    pub recommended: bool,
}
