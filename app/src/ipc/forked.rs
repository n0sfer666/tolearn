use super::dto::dto;
use super::shape::Shape;
use super::types::Span;

dto!(ForkIn {
    program: String,
    node: String,
    stage: String,
});
dto!(ForkOut {
    variants: Vec<VariantView>,
});
dto!(VariantView {
    id: String,
    title: String,
    hours: Span,
    why: String,
    recommended: bool,
});
dto!(TakeNextIn {
    program: String,
    node: String,
    stage: String,
    choice: u32,
});
dto!(TakeNextOut {
    node: String,
    stage: String,
});

pub fn shapes() -> Vec<Shape> {
    vec![
        ForkIn::shape(),
        ForkOut::shape(),
        VariantView::shape(),
        TakeNextIn::shape(),
        TakeNextOut::shape(),
    ]
}
