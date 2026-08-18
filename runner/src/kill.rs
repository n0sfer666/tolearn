use std::process::{Command, Stdio};

#[cfg(unix)]
pub fn tree(leader: u32) {
    quietly(Command::new("kill").args(["-9", "--", &format!("-{leader}")]));
}

#[cfg(windows)]
pub fn tree(leader: u32) {
    quietly(Command::new("taskkill").args(["/T", "/F", "/PID", &leader.to_string()]));
}

fn quietly(command: &mut Command) {
    let _ = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
