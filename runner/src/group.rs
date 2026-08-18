use std::process::Command;

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
