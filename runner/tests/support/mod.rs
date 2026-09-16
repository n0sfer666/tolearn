#[path = "../../../tests-support/scratch.rs"]
mod scratch;

use std::path::PathBuf;
use std::time::Duration;

use tolearn_runner::Limits;

#[cfg(unix)]
pub const SHELL: (&str, &str) = ("sh", "-c");
#[cfg(windows)]
pub const SHELL: (&str, &str) = ("cmd", "/C");

#[cfg(unix)]
pub const SLEEP: &str = "sleep 30";
#[cfg(windows)]
pub const SLEEP: &str = "ping -n 31 127.0.0.1 >nul";

pub fn scratch(name: &str) -> PathBuf {
    scratch::made(&format!("spawn-{name}"))
}

pub fn args(script: &str) -> Vec<String> {
    vec![SHELL.1.to_owned(), script.to_owned()]
}

pub fn limits(millis: u64, bytes: usize) -> Limits {
    Limits {
        timeout: Duration::from_millis(millis),
        silence: None,
        output_bytes: bytes,
    }
}
