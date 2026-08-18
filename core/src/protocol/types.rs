use crate::progress::TopicState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    pub state: TopicState,
    pub retry: Vec<String>,
    pub split_suggested: bool,
}
