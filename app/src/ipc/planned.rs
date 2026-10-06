use super::dto::dto;
use super::shape::Shape;
use super::types::Span;

dto!(PlanProgramIn {
    request: String,
    level: String,
    locale: String,
});
dto!(RevisePlanIn {
    request: String,
    level: String,
    locale: String,
    plan: PlanView,
    wish: String,
});
dto!(PlanOut {
    plan: PlanView,
    hours: Span,
});
dto!(PlanView {
    title: String,
    slug: String,
    goal: String,
    volatility: String,
    stages: Vec<PlanStageView>,
    children: Vec<PlanPartView>,
});
dto!(PlanStageView {
    id: String,
    title: String,
    hours: Span,
});
dto!(PlanPartView {
    title: String,
    goal: String,
    hours: Span,
});

pub fn shapes() -> Vec<Shape> {
    vec![
        PlanProgramIn::shape(),
        RevisePlanIn::shape(),
        PlanOut::shape(),
        PlanView::shape(),
        PlanStageView::shape(),
        PlanPartView::shape(),
    ]
}
