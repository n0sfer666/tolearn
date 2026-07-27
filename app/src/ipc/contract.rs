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
    prompt(types::PromptIn) -> types::PromptOut,
    programs(types::ProgramsIn) -> types::ProgramsOut,
    import(types::ImportIn) -> types::ImportOut,
}
