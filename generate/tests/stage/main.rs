#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

mod answers;
#[allow(dead_code, reason = "the plan helpers are shared with the plan tests")]
#[path = "../support/mod.rs"]
mod support;
mod web;

mod compose;
mod gather;
mod mend;
mod prompts;
mod text;
