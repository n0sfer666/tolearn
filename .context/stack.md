# Стек

**ДОМЕН: systems** (Rust-десктоп) + **web** (Astro/Solid UI внутри WebView).

Смешанный стек, поэтому справочники открываются оба, по месту работы:

| Работаю в | Справочник правил | Инструкция по тестам |
|---|---|---|
| `core/`, `runner/`, `offline/`, `cli/`, `app/` (Rust) | `~/.claude/rules/domains/systems.md` | `~/wiki/knowledge/testing/systems.md` |
| `ui/` (Astro, Solid, CSS) | `~/.claude/rules/domains/web.md`, `~/.claude/rules/topics/styles.md`, `topics/typescript.md` | `~/wiki/knowledge/testing/web.md` |
| `cli/` как пользовательский интерфейс | — | `~/wiki/knowledge/testing/cli.md` |
| GitHub Actions | `~/.claude/rules/topics/ci.md` | — |
| разбор бандла, запуск чужих команд, age, распаковка архивов | `~/.claude/rules/topics/security.md` | — |

Справочники не автозагружаются. Задача трогает тему — открыть файл самому, до кода.

## Инструменты

| | Версия на машине | Примечание |
|---|---|---|
| rustc / cargo | 1.97.1 | версия пиннится `rust-toolchain.toml` (S01) |
| clippy, rustfmt | из того же toolchain | `clippy -D warnings` — гейт CI |
| node | 24.x | нужен только для `ui/` (S26+) |
| pnpm | есть | менеджер пакетов для `ui/` |

## Зависимости, выбранные заранее (ADR)

| Что | Крейт / пакет | Лицензия | Где | ADR |
|---|---|---|---|---|
| оболочка | `tauri` 2 | MIT/Apache-2.0 | `app/` | [001](../docs/adr/001-shell-tauri.md) |
| страницы | `astro` (static MPA) | MIT | `ui/` | [002](../docs/adr/002-rendering-astro-mpa.md) |
| острова | `solid-js` | MIT | `ui/` | [002](../docs/adr/002-rendering-astro-mpa.md) |
| архивация страниц | `monolith` (как библиотека) | CC0-1.0 | `offline/` | [003](../docs/adr/003-offline-monolith.md) |
| читалка | `dom_smoothie` | — | `offline/` | [003](../docs/adr/003-offline-monolith.md) |
| git-клон материалов | `gix` | — | `offline/` | [003](../docs/adr/003-offline-monolith.md) |
| индекс кэша | SQLite | — | `offline/` | [005](../docs/adr/005-cache-outside-bundle.md) |
| шифрование конспектов | `rage` (age) | MIT/Apache-2.0 | S53 | — |
| распознавание речи | whisper.cpp | MIT | S55, отдельная сборка | [010](../docs/adr/010-speech-distribution.md) |

Лицензия проекта — GPL-3.0 ([ADR-008](../docs/adr/008-license-gpl.md)). Любая
новая зависимость обязана быть с ней совместима; несовместимую не тащим, а
меняем решение.

**Не бандлим:** `yt-dlp`, `ffmpeg` — ищутся в `PATH`
([ADR-004](../docs/adr/004-video-external.md)). Chromium — не нужен, пререндер
делает системный WebView.

Зависимость добавляется, только когда её требует текущая спека. Из внешнего в
воркспейсе пока ровно одно — `toml` в dev-зависимостях `cli`: им читает
манифесты архитектурный гейт `cli/tests/layering.rs`.
