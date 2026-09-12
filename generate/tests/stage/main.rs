#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "generate gate: a panic here is the report"
)]

mod answers;
mod forking;
mod metered;
mod starting;
#[allow(dead_code, reason = "the plan helpers are shared with the plan tests")]
#[path = "../support/mod.rs"]
mod support;
mod web;

mod cancel;
mod compose;
mod counted;
mod fork;
mod fork_refused;
mod gather;
mod ledger;
mod ledger_file;
mod mend;
mod next;
mod next_cancel;
mod prompts;
mod refused;
mod regenerate;
mod regenerate_cancel;
mod regenerate_saved;
mod start;
mod text;
