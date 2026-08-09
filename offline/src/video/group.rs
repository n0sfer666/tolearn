use std::process::{Command, Stdio};

#[cfg(unix)]
pub fn lead(command: &mut Command) -> &mut Command {
    use std::os::unix::process::CommandExt;

    command.process_group(0)
}

#[cfg(windows)]
pub fn lead(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

    command.creation_flags(CREATE_NEW_PROCESS_GROUP)
}

#[cfg(unix)]
pub fn tree(leader: u32) {
    quietly(Command::new("kill").args(["-9", &format!("-{leader}")]));
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
