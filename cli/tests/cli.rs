#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

#[allow(
    dead_code,
    reason = "the usage gate takes only its part of the shared support module"
)]
mod support;

use support::{code, scratch, stderr, tolearn};

#[test]
fn a_command_nobody_knows_is_a_usage_error_not_a_panic() {
    let bundle = scratch("unknown-command");

    let out = tolearn(&["fly", bundle.to_str().unwrap()]);

    assert_ne!(code(&out), 0);
    assert!(
        stderr(&out).contains("неизвестная команда `fly`"),
        "{}",
        stderr(&out)
    );
    assert!(!stderr(&out).contains("panicked"), "{}", stderr(&out));
}

#[test]
fn the_v1_bundle_commands_are_gone() {
    let bundle = scratch("v1-commands");
    for name in ["validate", "scan", "progress", "exam", "merge"] {
        let out = tolearn(&[name, bundle.to_str().unwrap()]);

        assert_eq!(code(&out), 2, "{name}");
        assert!(
            stderr(&out).contains(&format!("неизвестная команда `{name}`")),
            "{name}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn a_v1_command_is_unknown_before_its_arguments_are_read() {
    for argv in [
        vec!["validate"],
        vec!["exam", "somewhere", "ownership", "--run-checks"],
        vec!["merge", "somewhere", "--was", "before"],
    ] {
        let out = tolearn(&argv);

        assert!(
            stderr(&out).contains(&format!("неизвестная команда `{}`", argv[0])),
            "{argv:?}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn a_missing_argument_is_named_by_the_command() {
    for (argv, said) in [
        (vec!["export"], "`export` не назвал каталог программы"),
        (vec!["pack"], "`pack` не назвал каталог программы"),
        (vec!["unpack"], "`unpack` не назвал пакет"),
        (
            vec!["unpack", "chiptune.tolearn"],
            "`unpack` не назвал каталог",
        ),
    ] {
        let out = tolearn(&argv);

        assert_eq!(code(&out), 2, "{argv:?}");
        assert!(stderr(&out).contains(said), "{argv:?}: {}", stderr(&out));
    }
}

#[test]
fn the_usage_names_only_export_pack_and_unpack() {
    let out = tolearn(&["fly", "somewhere"]);

    let usage = stderr(&out);
    for kept in ["tolearn export", "tolearn pack", "tolearn unpack"] {
        assert!(usage.contains(kept), "{kept}: {usage}");
    }
    for gone in [
        "validate",
        "scan",
        "progress",
        "exam",
        "merge",
        "run-checks",
    ] {
        assert!(!usage.contains(gone), "{gone}: {usage}");
    }
}
