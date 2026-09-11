#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "provider gate: a panic here is the report"
)]

mod support;

use support::harness;
use tolearn_provider::{PRESETS, ask, preset};

#[test]
fn opencode_и_pi_закрыты_до_конца_v2() {
    let open: Vec<(&str, bool)> = PRESETS
        .iter()
        .map(|preset| (preset.id, preset.available))
        .collect();

    assert_eq!(
        open,
        [
            ("claude", true),
            ("opencode", false),
            ("pi", false),
            ("custom", true)
        ]
    );
}

#[test]
fn пресет_claude_без_чужого_контекста() {
    let claude = preset("claude").expect("пресет claude зарегистрирован");

    assert_eq!(
        claude.advised(),
        vec![
            "-p",
            "--output-format",
            "stream-json",
            "--verbose",
            "--include-partial-messages",
            "--tools",
            "",
            "--system-prompt",
            "Выполни инструкцию из сообщения, ответь только результатом.",
            "--setting-sources",
            "project",
            "--strict-mcp-config",
        ]
    );
}

#[test]
fn промпт_идёт_только_через_stdin_а_не_позиционным_аргументом() {
    let claude = preset("claude").expect("пресет claude зарегистрирован");
    let mut args = vec!["args".to_owned()];
    args.extend(claude.advised());

    let provider = harness(&args, 20);
    let answer =
        ask(&provider, None, "нельзя перепутать с именем инструмента").expect("харнесс отвечает");

    assert_eq!(answer.text, claude.advised().join(" "));
    assert!(!answer.text.contains("нельзя перепутать"));
}
