#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

mod answers;
mod metered;
mod starting;
#[allow(dead_code, reason = "the plan helpers are shared with the plan tests")]
#[path = "../support/mod.rs"]
mod support;
mod web;

mod cancel;
mod compose;
mod counted;
mod gather;
mod ledger;
mod ledger_file;
mod mend;
mod prompts;
mod refused;
mod start;
mod text;
