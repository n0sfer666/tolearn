#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "core gate: a panic here is the report"
)]

mod support;

use std::fs;

use support::search::{fresh, home, shelf};

#[test]
fn hidden_files_inside_a_program_do_not_force_a_reread() {
    let (data, library) = shelf("walk-hidden");
    let mut index = fresh(&library);

    fs::write(home(&data).join(".DS_Store"), "finder").unwrap();
    fs::write(home(&data).join("stages/.voices.yaml.swp"), "vim").unwrap();
    let report = index.refresh(&library).unwrap();

    assert_eq!((report.indexed, report.kept), (0, 2));
}

#[cfg(unix)]
mod unix {
    use std::fs;
    use std::os::unix::fs::symlink;

    use super::support::programs::chmod;
    use super::support::search::{fresh, home, retitle, shelf};
    use tolearn_core::search::Index;

    #[test]
    fn a_program_that_cannot_be_listed_is_left_out_and_read_later() {
        let (data, library) = shelf("walk-locked");
        let stages = home(&data).join("stages");
        let mut index = Index::default();

        chmod(&stages, 0o000);
        let locked = index.refresh(&library);
        let hidden = index.find("голоса чипа", 10).is_empty();
        let others = !index.find("компоновщика", 10).is_empty();
        chmod(&stages, 0o755);
        let report = index.refresh(&library).unwrap();

        assert!(locked.is_ok(), "{locked:?}");
        assert!(hidden && others);
        assert_eq!((report.indexed, report.kept), (1, 1));
        assert!(!index.find("голоса чипа", 10).is_empty());
    }

    #[test]
    fn a_stage_that_cannot_be_read_is_not_remembered_as_empty() {
        let (data, library) = shelf("walk-unreadable");
        let voices = home(&data).join("stages/voices.yaml");
        let mut index = Index::default();

        chmod(&voices, 0o000);
        let locked = index.refresh(&library);
        let hidden = index.find("голоса чипа", 10).is_empty();
        chmod(&voices, 0o644);
        let report = index.refresh(&library).unwrap();

        assert!(locked.is_ok(), "{locked:?}");
        assert!(hidden);
        assert_eq!(report.indexed, 1);
        assert!(!index.find("голоса чипа", 10).is_empty());
    }

    #[test]
    fn a_linked_stage_is_watched_through_its_link() {
        let (data, library) = shelf("walk-linked");
        let voices = home(&data).join("stages/voices.yaml");
        let outside = data.join("voices.yaml");
        fs::rename(&voices, &outside).unwrap();
        symlink(&outside, &voices).unwrap();
        symlink(".", home(&data).join("loop")).unwrap();
        let mut index = fresh(&library);
        assert!(!index.find("голоса чипа", 10).is_empty());

        retitle(&data, "Тарабарщина");
        let report = index.refresh(&library).unwrap();

        assert_eq!((report.indexed, report.kept), (1, 1));
        assert!(!index.find("тарабарщина", 10).is_empty());
    }
}
