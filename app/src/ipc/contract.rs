use serde_json::Value;

use super::context::Context;
use super::error::IpcError;
use super::handlers;
use super::shape::Shape;
use super::types;

#[derive(Debug)]
pub struct Descriptor {
    pub name: String,
    pub input: Shape,
    pub output: Shape,
}

macro_rules! commands {
    ($($name:ident ($input:ty) -> $output:ty),* $(,)?) => {
        pub const NAMES: &[&str] = &[$(stringify!($name)),*];

        pub fn descriptors() -> Vec<Descriptor> {
            vec![$(Descriptor {
                name: stringify!($name).to_owned(),
                input: <$input>::shape(),
                output: <$output>::shape(),
            }),*]
        }

        pub fn call(context: &Context, name: &str, payload: &Value) -> Result<Value, IpcError> {
            $(
                if name == stringify!($name) {
                    let input: $input =
                        serde_json::from_value(payload.clone()).map_err(|error| IpcError::payload(&error))?;
                    let output: $output = handlers::$name::run(context, &input)?;
                    return serde_json::to_value(output).map_err(|error| IpcError::payload(&error));
                }
            )*
            Err(IpcError::unknown_command(name))
        }
    };
}

commands! {
    validate(types::ValidateIn) -> types::ValidateOut,
    scan(types::ScanIn) -> types::ScanOut,
    program(types::ProgramIn) -> types::ProgramOut,
    topic(types::TopicIn) -> types::TopicOut,
    run_check(types::RunCheckIn) -> types::RunCheckOut,
    set_status(types::SetStatusIn) -> types::SetStatusOut,
    note(types::NoteIn) -> types::NoteOut,
    save_note(types::SaveNoteIn) -> types::SaveNoteOut,
    parse_verdict(types::ParseVerdictIn) -> types::VerdictView,
    apply_verdict(types::ApplyVerdictIn) -> types::ApplyVerdictOut,
    review(types::ReviewIn) -> types::ReviewOut,
    prompt(types::PromptIn) -> types::PromptOut,
    examine(types::ExamineIn) -> types::ExamineOut,
    practice(types::PracticeIn) -> types::PracticeOut,
    programs(types::ProgramsIn) -> types::ProgramsOut,
    plan(types::PlanIn) -> types::PlanOut,
    stale(types::StaleIn) -> types::StaleOut,
    export(types::ExportIn) -> types::ExportOut,
    graph(types::GraphIn) -> types::GraphOut,
    stats(types::StatsIn) -> types::StatsOut,
    queue(types::QueueIn) -> types::QueueOut,
    repeat(types::RepeatIn) -> types::RepeatOut,
    import(types::ImportIn) -> types::ImportOut,
    history(types::HistoryIn) -> types::HistoryOut,
    history_diff(types::HistoryDiffIn) -> types::HistoryDiffOut,
    settings(types::SettingsIn) -> types::SettingsView,
    search(types::SearchIn) -> types::SearchOut,
    provider(types::ProviderIn) -> types::ProviderOut,
}

#[tauri::command]
pub fn command(app: tauri::AppHandle, name: String, payload: Value) -> Result<Value, IpcError> {
    call(&super::context::of(&app)?, &name, &payload)
}
