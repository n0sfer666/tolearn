# tolearn

Локальное офлайн-приложение для обучения по программам, которые генерирует LLM.
Rust-ядро + Tauri 2, UI — статическая Astro MPA с островами SolidJS.

## Перед работой

Прочитать [.context/](.context/) — там карта проекта, конвенции, состояние работ.
Реализация идёт строго по спекам из [docs/specs/README.md](docs/specs/README.md),
по критическому пути, по порядку. TDD: тест пишется до кода; спека закрыта, когда
её DoD проверен запуском.

## Команды

```
cargo check --workspace --all-targets
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Полный список и то, что появится позже, — [.context/checks.md](.context/checks.md).

## Что нельзя

- Тащить в `core` знание про Tauri и UI, в `offline` — про Tauri.
- Ходить в сеть в тестах и требовать сеть, аккаунт или ключ для основной работы.
- Ослаблять бюджеты из [docs/architecture.md](docs/architecture.md#бюджеты).
- Выполнять `check`-команды из бандла без явного действия человека.
- Писать в файлы бандла что-либо, кроме `progress.yaml`.
