use super::dto::dto;
use super::forked::ForkOut;
use super::planned::{PlanOut, PlanView};
use super::shape::Shape;
use super::started::GenerationStep;

pub const STATE_EVENT: &str = "generation-state";

dto!(GenerationStateIn {});
dto!(GenerationStateOut {
    work: Option<GenerationWork>,
    outcome: Option<GenerationOutcome>,
});
dto!(GenerationWork {
    kind: String,
    program: String,
    node: String,
    stage: String,
    request: String,
    level: String,
    locale: String,
    plan: Option<PlanView>,
    wish: String,
    began: u64,
    marks: Vec<StepMark>,
});
dto!(StepMark {
    step: GenerationStep,
    at: u64,
});
dto!(GenerationOutcome {
    work: GenerationWork,
    stage: Option<ReadyStage>,
    fork: Option<ForkOut>,
    plan: Option<PlanOut>,
    refusal: Option<RefusalView>,
    seen: bool,
    ended: u64,
});
dto!(ReadyStage {
    program: String,
    node: String,
    stage: String,
});
dto!(RefusalView {
    code: String,
    message: String,
});

impl GenerationWork {
    pub fn asked(kind: &str, request: &str, level: &str, locale: &str) -> Self {
        Self {
            request: request.to_owned(),
            level: level.to_owned(),
            locale: locale.to_owned(),
            ..Self::blank(kind)
        }
    }

    pub fn placed(kind: &str, program: &str, node: &str, stage: &str) -> Self {
        Self {
            program: program.to_owned(),
            node: node.to_owned(),
            stage: stage.to_owned(),
            ..Self::blank(kind)
        }
    }

    pub fn drawn(mut self, plan: &PlanView, wish: &str) -> Self {
        self.plan = Some(plan.clone());
        self.wish = wish.to_owned();
        self
    }

    fn blank(kind: &str) -> Self {
        Self {
            kind: kind.to_owned(),
            program: String::new(),
            node: String::new(),
            stage: String::new(),
            request: String::new(),
            level: String::new(),
            locale: String::new(),
            plan: None,
            wish: String::new(),
            began: 0,
            marks: Vec::new(),
        }
    }
}

pub fn shapes() -> Vec<Shape> {
    vec![
        GenerationStateIn::shape(),
        GenerationStateOut::shape(),
        GenerationWork::shape(),
        StepMark::shape(),
        GenerationOutcome::shape(),
        ReadyStage::shape(),
        RefusalView::shape(),
    ]
}
