#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Due {
    pub topic: String,
    pub title: String,
    pub due: String,
    pub overdue: bool,
}
