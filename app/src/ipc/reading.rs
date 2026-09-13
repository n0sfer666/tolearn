use super::dto::dto;
use super::shape::Shape;
use super::types::Span;

dto!(LibraryIn {});
dto!(LibraryOut {
    programs: Vec<ShelfView>,
    refused: Vec<RefusedView>,
});
dto!(ShelfView {
    uuid: String,
    title: String,
    goal: String,
    hours: Span,
    children: Vec<RowView>,
});
dto!(RefusedView {
    directory: String,
    code: String,
    message: String,
});
dto!(RowView {
    id: String,
    title: String,
    hours: Span,
    ready: bool,
});

dto!(ImportPackageIn { path: String });
dto!(ImportPackageOut {
    uuid: String,
    title: String,
    copy_of: Option<String>,
});

dto!(NodeIn {
    program: String,
    node: String,
});
dto!(NodeOut {
    program: String,
    uuid: String,
    title: String,
    goal: String,
    level: String,
    hours: Span,
    trail: Vec<CrumbView>,
    stages: Vec<StageRowView>,
    children: Vec<RowView>,
    summary: SummaryView,
});
dto!(StageRowView {
    id: String,
    title: String,
    hours: Span,
    ready: bool,
    status: String,
    pass: Option<String>,
});
dto!(SummaryView {
    passed: u32,
    total: u32,
    skipped: u32,
});
dto!(CrumbView {
    uuid: String,
    title: String,
});

dto!(StageIn {
    program: String,
    node: String,
    stage: String,
});
dto!(StageOut {
    program: String,
    node: String,
    node_title: String,
    id: String,
    title: String,
    blocks: Vec<BlockView>,
    practice: TaskView,
    questions: Vec<AskView>,
});
dto!(BlockView {
    id: String,
    kind: String,
    text: String,
    lang: Option<String>,
    src: Option<String>,
    license: Option<String>,
    attribution: Option<String>,
    source: Option<String>,
});
dto!(TaskView {
    task: Vec<BlockView>,
    deliverable: String,
    constraints: Vec<ClaimView>,
    acceptance: Vec<ClaimView>,
});
dto!(ClaimView {
    id: String,
    claim: String,
    check: Option<String>,
    expect: String,
});
dto!(AskView {
    id: String,
    text: String,
});
dto!(ExportIn {
    program: String,
    node: String,
    folder: String,
});
dto!(ExportOut {
    path: String,
    files: u64,
});

pub fn shapes() -> Vec<Shape> {
    vec![
        LibraryIn::shape(),
        LibraryOut::shape(),
        ShelfView::shape(),
        RefusedView::shape(),
        RowView::shape(),
        ImportPackageIn::shape(),
        ImportPackageOut::shape(),
        NodeIn::shape(),
        NodeOut::shape(),
        StageRowView::shape(),
        SummaryView::shape(),
        CrumbView::shape(),
        StageIn::shape(),
        StageOut::shape(),
        BlockView::shape(),
        TaskView::shape(),
        ClaimView::shape(),
        AskView::shape(),
        ExportIn::shape(),
        ExportOut::shape(),
    ]
}
