#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked {
    pub topic: String,
    pub title: String,
    pub due: Option<String>,
    pub overdue: bool,
}
