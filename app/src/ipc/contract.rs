use serde_json::Value;

use super::clarified;
use super::context::Context;
use super::error::IpcError;
use super::examined;
use super::forked;
use super::handlers;
use super::planned;
use super::practiced;
use super::reading;
use super::regenerated;
use super::shape::Shape;
use super::started;
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
    export(reading::ExportIn) -> reading::ExportOut,
    settings(types::SettingsIn) -> types::SettingsView,
    search(types::SearchIn) -> types::SearchOut,
    provider(types::ProviderIn) -> types::ProviderOut,
    llm_log(types::LlmLogIn) -> types::LlmLogOut,
    speech_state(types::SpeechStateIn) -> types::SpeechStateOut,
    speech_start(types::SpeechStateIn) -> types::SpeechStateOut,
    speech_stop(types::SpeechStopIn) -> types::SpeechStopOut,
    library(reading::LibraryIn) -> reading::LibraryOut,
    import_package(reading::ImportPackageIn) -> reading::ImportPackageOut,
    delete_program(reading::DeleteProgramIn) -> reading::DeleteProgramOut,
    node(reading::NodeIn) -> reading::NodeOut,
    stage(reading::StageIn) -> reading::StageOut,
    plan_program(planned::PlanProgramIn) -> planned::PlanOut,
    revise_plan(planned::RevisePlanIn) -> planned::PlanOut,
    start_program(started::StartProgramIn) -> started::StartProgramOut,
    cancel_generation(started::CancelGenerationIn) -> started::CancelGenerationOut,
    fork(forked::ForkIn) -> forked::ForkOut,
    take_next(forked::TakeNextIn) -> forked::TakeNextOut,
    regenerate_stage(regenerated::RegenerateStageIn) -> regenerated::RegenerateStageOut,
    tick(practiced::TickIn) -> practiced::TickOut,
    workdir(practiced::WorkdirIn) -> practiced::WorkdirOut,
    check_claim(practiced::CheckClaimIn) -> practiced::CheckClaimOut,
    answer(examined::AnswerIn) -> examined::AnswerOut,
    exam(examined::ExamIn) -> examined::ExamOut,
    exam_prompt(examined::ExamIn) -> examined::ExamPromptOut,
    exam_paste(examined::ExamPasteIn) -> examined::ExamOut,
    skip(reading::StageIn) -> examined::SkipOut,
    clarify(clarified::ClarifyIn) -> clarified::ClarificationsOut,
    understood(clarified::ChainIn) -> clarified::ClarificationsOut,
    unclarify(clarified::ChainIn) -> clarified::ClarificationsOut,
}

#[tauri::command]
pub async fn command(
    app: tauri::AppHandle,
    name: String,
    payload: Value,
) -> Result<Value, IpcError> {
    tauri::async_runtime::spawn_blocking(move || call(&super::context::of(&app)?, &name, &payload))
        .await
        .map_err(|error| IpcError::new("ipc.crashed", error.to_string()))?
}
