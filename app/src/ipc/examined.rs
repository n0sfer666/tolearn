use super::dto::dto;
use super::shape::Shape;

dto!(AnswerIn {
    program: String,
    node: String,
    stage: String,
    question: String,
    text: String,
});
dto!(AnswerOut { draft: String });
dto!(ExamIn {
    program: String,
    node: String,
    stage: String,
    answers: Vec<AnswerView>,
});
dto!(AnswerView {
    id: String,
    text: String,
});
dto!(ExamOut { passed: bool });
dto!(ExamPromptOut { prompt: String });
dto!(ExamPasteIn {
    program: String,
    node: String,
    stage: String,
    text: String,
});

pub fn shapes() -> Vec<Shape> {
    vec![
        AnswerIn::shape(),
        AnswerOut::shape(),
        ExamIn::shape(),
        AnswerView::shape(),
        ExamOut::shape(),
        ExamPromptOut::shape(),
        ExamPasteIn::shape(),
    ]
}
