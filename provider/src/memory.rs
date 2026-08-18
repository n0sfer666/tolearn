use std::sync::OnceLock;

const BYTES_IN_GIGABYTE: u64 = 1_073_741_824;

pub fn memory() -> u32 {
    static MEASURED: OnceLock<u32> = OnceLock::new();
    *MEASURED.get_or_init(|| {
        bytes().map_or(0, |bytes| {
            u32::try_from(bytes / BYTES_IN_GIGABYTE).unwrap_or(u32::MAX)
        })
    })
}

#[cfg(target_os = "macos")]
fn bytes() -> Option<u64> {
    said("sysctl", &["-n", "hw.memsize"])?.parse().ok()
}

#[cfg(target_os = "linux")]
fn bytes() -> Option<u64> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let total = meminfo.lines().find(|line| line.starts_with("MemTotal:"))?;
    let kilobytes: u64 = total.split_whitespace().nth(1)?.parse().ok()?;
    Some(kilobytes * 1024)
}

#[cfg(target_os = "windows")]
fn bytes() -> Option<u64> {
    said(
        "powershell",
        &[
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ],
    )?
    .parse()
    .ok()
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn bytes() -> Option<u64> {
    None
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn said(command: &str, args: &[&str]) -> Option<String> {
    let spoken = std::process::Command::new(command)
        .args(args)
        .output()
        .ok()?;
    if !spoken.status.success() {
        return None;
    }
    Some(String::from_utf8(spoken.stdout).ok()?.trim().to_owned())
}
