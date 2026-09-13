use super::dto::dto;
use super::shape::Shape;

dto!(ClarifyIn {
    program: String,
    node: String,
    stage: String,
    block: String,
    question: String,
    chain: Option<u32>,
});
dto!(ChainIn {
    program: String,
    node: String,
    stage: String,
    chain: u32,
});
dto!(ClarificationsOut {
    clarifications: Vec<ClarificationView>,
});
dto!(ClarificationView {
    chain: u32,
    block: String,
    excerpt: String,
    turns: Vec<TurnView>,
    clear: bool,
});
dto!(TurnView {
    asked: Option<String>,
    answer: String,
});

pub fn shapes() -> Vec<Shape> {
    vec![
        ClarifyIn::shape(),
        ChainIn::shape(),
        ClarificationsOut::shape(),
        ClarificationView::shape(),
        TurnView::shape(),
    ]
}
