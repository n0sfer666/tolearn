use std::path::{Path, PathBuf};

#[cfg(unix)]
const UNDER_HOME: [&str; 1] = [".local/bin"];

#[cfg(unix)]
const ALWAYS: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];

pub fn separator() -> char {
    if cfg!(windows) { ';' } else { ':' }
}

pub fn search() -> String {
    enriched(
        &std::env::var("PATH").unwrap_or_default(),
        home().as_deref(),
    )
}

pub fn candidates(program: &str, search: &str) -> Vec<PathBuf> {
    if Path::new(program).components().count() > 1 {
        return vec![PathBuf::from(program)];
    }
    let file = format!("{program}{}", std::env::consts::EXE_SUFFIX);
    let mut found: Vec<PathBuf> = search
        .split(separator())
        .filter(|part| !part.is_empty())
        .map(|part| Path::new(part).join(&file))
        .filter(|candidate| runnable(candidate))
        .collect();
    found.push(PathBuf::from(program));
    found
}

fn enriched(inherited: &str, home: Option<&str>) -> String {
    let mut places: Vec<String> = inherited
        .split(separator())
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect();
    for place in also(home) {
        if !places.contains(&place) {
            places.push(place);
        }
    }
    places.join(&separator().to_string())
}

#[cfg(unix)]
fn also(home: Option<&str>) -> Vec<String> {
    let under = home.into_iter().flat_map(|home| {
        UNDER_HOME
            .iter()
            .map(move |place| Path::new(home).join(place).to_string_lossy().into_owned())
    });
    under
        .chain(ALWAYS.iter().map(|place| (*place).to_owned()))
        .collect()
}

#[cfg(windows)]
fn also(_home: Option<&str>) -> Vec<String> {
    Vec::new()
}

#[cfg(unix)]
fn home() -> Option<String> {
    std::env::var("HOME").ok()
}

#[cfg(windows)]
fn home() -> Option<String> {
    None
}

#[cfg(unix)]
fn runnable(candidate: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(candidate)
        .is_ok_and(|about| about.is_file() && about.permissions().mode() & 0o111 != 0)
}

#[cfg(windows)]
fn runnable(candidate: &Path) -> bool {
    candidate.is_file()
}

#[cfg(all(test, unix))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "runner gate: a panic here is the report"
    )]

    use super::*;

    const GUI: &str = "/usr/bin:/bin:/usr/sbin:/sbin";

    const HARNESS: &str = "tolearn-fake-harness";

    fn planted(name: &str, mode: u32) -> (PathBuf, PathBuf) {
        use std::os::unix::fs::PermissionsExt;

        let home = std::env::temp_dir().join(format!(
            "tolearn-path-{name}-{}-{mode:o}",
            std::process::id()
        ));
        let bin = home.join(".local").join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let file = bin.join(HARNESS);
        std::fs::write(&file, "#!/bin/sh\necho ready\n").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(mode)).unwrap();
        (home, file)
    }

    fn places(search: &str) -> Vec<&str> {
        search
            .split(separator())
            .filter(|part| !part.is_empty())
            .collect()
    }

    #[test]
    fn a_harness_outside_the_gui_path_is_found_all_the_same() {
        let (home, file) = planted("outside", 0o755);

        assert_eq!(candidates(HARNESS, GUI), vec![PathBuf::from(HARNESS)]);
        assert_eq!(
            candidates(HARNESS, &enriched(GUI, home.to_str())).first(),
            Some(&file)
        );
    }

    #[test]
    fn the_inherited_path_keeps_its_place_at_the_front() {
        let search = enriched(GUI, Some("/home/somebody"));

        assert_eq!(&places(&search)[..4], &places(GUI)[..]);
        assert!(places(&search).len() > 4, "{search}");
    }

    #[test]
    fn a_place_already_inherited_is_not_added_twice() {
        let doubled = enriched(&format!("/usr/local/bin{}{GUI}", separator()), None);
        let seen = places(&doubled)
            .iter()
            .filter(|place| **place == "/usr/local/bin")
            .count();

        assert_eq!(seen, 1, "{doubled}");
    }

    #[test]
    fn an_empty_inherited_path_still_leads_somewhere() {
        let (home, file) = planted("empty", 0o755);

        assert_eq!(
            candidates(HARNESS, &enriched("", home.to_str())).first(),
            Some(&file)
        );
    }

    #[test]
    fn a_command_given_as_a_path_is_taken_as_it_is() {
        let (_, file) = planted("spelled", 0o755);
        let spelled = file.to_str().unwrap();

        assert_eq!(candidates(spelled, GUI), vec![file]);
    }

    #[test]
    fn a_file_that_cannot_be_run_is_not_a_candidate() {
        let (home, file) = planted("unrunnable", 0o644);

        assert!(!candidates(HARNESS, &enriched(GUI, home.to_str())).contains(&file));
    }

    #[test]
    fn a_harness_that_is_nowhere_leaves_the_search_to_the_platform() {
        let home =
            std::env::temp_dir().join(format!("tolearn-path-nowhere-{}", std::process::id()));

        assert_eq!(
            candidates(HARNESS, &enriched(GUI, home.to_str())),
            vec![PathBuf::from(HARNESS)]
        );
    }
}
