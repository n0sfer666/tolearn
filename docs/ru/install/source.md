# Сборка из исходников

**Русский** · [English](../../en/install/source.md)

Это способ для тех, кому не подходит готовый установщик: Intel-Mac, ARM-Linux,
дистрибутив не из семейства Debian, или просто желание собрать самому. Получится
тот же установщик, что выкладывается в релизе, — командой из этой страницы его
собирает и CI.

## Что нужно поставить

| Инструмент | Версия | Зачем |
|---|---|---|
| Rust | берётся из `rust-toolchain.toml` (1.97.1) | ядро и приложение |
| Node.js | 24 | сборка интерфейса |
| pnpm | 10 | зависимости интерфейса |
| Tauri CLI | 2.11.4 | сборка установщика |

Rust ставится через [rustup](https://rustup.rs) — нужную версию он подхватит из
`rust-toolchain.toml` сам. Tauri CLI:

```sh
cargo install tauri-cli --version 2.11.4 --locked
```

Плюс системные зависимости:

- **macOS** — Command Line Tools: `xcode-select --install`.
- **Windows** — Visual Studio Build Tools с рабочей нагрузкой «Разработка
  классических приложений на C++» и WebView2 (в Windows 11 уже есть).
- **Linux (Debian/Ubuntu)**:

  ```sh
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev patchelf
  ```

  Для варианта с речью — ещё `libasound2-dev`.

## Собрать

```sh
git clone https://github.com/n0sfer666/tolearn.git
cd tolearn
pnpm -C ui install --frozen-lockfile
cd app
cargo tauri build
```

Интерфейс собирается сам: `beforeBuildCommand` в `tauri.conf.json` вызывает
`pnpm -C ui build` до компиляции. Готовые артефакты — в
`target/release/bundle/` в корне репозитория: `.dmg` на macOS, `.msi` на
Windows, `.deb` на Linux. Поставить их — как обычный установщик, по странице
своей ОС ([macOS](macos.md), [Windows](windows.md), [Linux](linux.md)).

На macOS два последних шага делает одна цель из корня репозитория:

```sh
make install
```

Она собирает тот же `.dmg`, монтирует его и копирует `tolearn.app` в
`/Applications` — ровно то же, что вы сделали бы руками в Finder. Другой каталог
назначения — вторым аргументом скрипта:
`sh scripts/install-macos.sh base ~/Applications`. Запущенное приложение цель не
трогает: попросит закрыть его и выйдет. Вариант с речью — `make install-speech`
(ему нужны сабмодуль и веса, см. ниже).

## Вариант с распознаванием речи

Ему нужны сабмодуль whisper.cpp и веса модели — файла с весами в репозитории нет,
его кладут рядом руками:

```sh
git submodule update --init --recursive
# app/models/ggml-small-q5_1.bin — положить самому
cd app
cargo tauri build --features speech --config tauri.with-speech.conf.json
```

Чем два варианта различаются и почему их не ставят вместе — в
[release.md](../release.md).

## Запустить без установки

Разработке — `cargo tauri dev` из `app` (поднимет и dev-сервер интерфейса).
Просто посмотреть на собранное приложение, не делая установщик:

```sh
pnpm -C ui build
cargo run -p tolearn-app --release --bin tolearn-desktop
```

Интерфейс здесь берётся из `ui/dist`, поэтому первая команда обязательна: без
неё приложение соберётся, но окно будет пустым.

Данные такой запуск кладёт туда же, куда и установленное приложение
([где именно](README.md#где-лежат-ваши-данные)), — отдельной песочницы у него нет.

Страницы зашиваются внутрь бинарника (фича `custom-protocol`, включена по
умолчанию); работать против dev-сервера Astro — `--no-default-features`.

## CLI и плагин Obsidian

В установщик они не входят, собираются отдельно. CLI без окна — проверить бандл,
посмотреть темы, выгрузить Markdown:

```sh
cargo run -p tolearn-cli --release -- validate examples/llm-agents-base
```

Плагин Obsidian:

```sh
pnpm -C obsidian install
pnpm -C obsidian build
```

Затем скопировать каталог `obsidian/` (нужны `main.js` и `manifest.json`) в
`<хранилище>/.obsidian/plugins/tolearn/`. Что он умеет — в
[notes.md](../notes.md#плагин-obsidian).
