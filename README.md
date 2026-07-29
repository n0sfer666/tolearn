# tolearn

Локальное приложение для обучения по программам, которые генерирует LLM.

LLM строит программу обучения — набор тем с материалами, практикой и вопросами.
`tolearn` показывает эту программу, ведёт прогресс, хранит конспекты и умеет
работать полностью без сети. Экзаменует по-прежнему LLM — но приложение не
требует ни аккаунта, ни ключа, ни подписки: промпт копируется в любой чат,
вердикт вставляется обратно.

**Статус: работает из исходников.** Импорт программы, темы, практика с таймером,
зачёт через копипаст и через провайдера, конспекты с поиском и шифрованием,
офлайн-архив, граф, история версий, экспорт в Markdown, плагин Obsidian — готовы.
Не готово: установщики под ОС (S57), варианты дистрибутива (S58) и голосовой
зачёт (S55–S56). Пока приложение собирается и запускается вручную.

[English below](#installation-from-source) · Руководство пользователя —
[docs/guide.md](docs/guide.md).

## Установка из исходников

Нужны: [Rust](https://rustup.rs) (версия из `rust-toolchain.toml` ставится сама),
[pnpm](https://pnpm.io) и Node 22+. На Linux — системные зависимости
[Tauri 2](https://tauri.app/start/prerequisites/); на macOS достаточно Xcode
Command Line Tools.

```sh
git clone <адрес репозитория> tolearn
cd tolearn
pnpm -C ui install
pnpm -C ui build
cargo run -p tolearn-app --release --bin tolearn-desktop
```

Первая сборка занимает несколько минут, дальше — секунды. Готовый бинарник
остаётся в `target/release/tolearn-desktop`, его можно запускать напрямую.
Страницы зашиваются внутрь бинарника (фича `custom-protocol`, включена по
умолчанию); работать против dev-сервера Astro — `--no-default-features`.

Есть и CLI без окна — проверить бандл, посмотреть темы, выгрузить Markdown:

```sh
cargo run -p tolearn-cli --release -- validate examples/llm-agents-base
```

Плагин Obsidian собирается отдельно: `pnpm -C obsidian install && pnpm -C
obsidian build`, затем каталог `obsidian/` (нужны `main.js` и `manifest.json`)
копируется в `<хранилище>/.obsidian/plugins/tolearn/`.

## Installation from source

You need [Rust](https://rustup.rs) (the toolchain from `rust-toolchain.toml` is
installed automatically), [pnpm](https://pnpm.io) and Node 22+. On Linux, also
the [Tauri 2](https://tauri.app/start/prerequisites/) system dependencies; on
macOS the Xcode Command Line Tools are enough.

```sh
git clone <repository url> tolearn
cd tolearn
pnpm -C ui install
pnpm -C ui build
cargo run -p tolearn-app --release --bin tolearn-desktop
```

The first build takes a few minutes, later ones take seconds. The binary stays
at `target/release/tolearn-desktop` and can be started directly. The pages are
embedded into the binary (the `custom-protocol` feature, on by default); to run
against the Astro dev server, pass `--no-default-features`.

There is a windowless CLI as well — validate a bundle, list topics, export
Markdown:

```sh
cargo run -p tolearn-cli --release -- validate examples/llm-agents-base
```

The Obsidian plugin is built separately: `pnpm -C obsidian install && pnpm -C
obsidian build`, then copy the `obsidian/` directory (`main.js` and
`manifest.json` are required) into `<vault>/.obsidian/plugins/tolearn/`.

Installers for macOS, Windows and Linux are not built yet (S57); until then the
source build above is the way to run the app. The user guide is in Russian —
[docs/guide.md](docs/guide.md).

## Принципы

- **Полностью офлайн.** Сеть нужна один раз — скачать материалы. Дальше приложение работает без неё.
- **Никаких SDK и платных функций.** Интеграция с LLM опциональна и работает через копипаст.
- **Данные — ваши.** Всё лежит в обычных файлах: YAML-программа, Markdown-конспекты. Приложение можно удалить, данные останутся.
- **Легко и быстро.** Rust-ядро, системный WebView, статические HTML-страницы. Без SPA-роутера, без бандла Chromium.
- **Свободно.** GPL-3.0, без телеметрии.

## Стек

| Слой | Решение | Почему |
|---|---|---|
| Ядро | Rust | парсинг, валидация, прогресс, офлайн-загрузка |
| Оболочка | Tauri 2 | системный WebView вместо Chromium |
| Страницы | Astro (статическая MPA) | реальные HTML-страницы, ноль JS там, где он не нужен |
| Интерактив | SolidJS-острова | точечно, только где есть состояние |
| Архив страниц | [monolith](https://github.com/Y2Z/monolith) (CC0) | страница в один самодостаточный HTML |
| Читалка | dom_smoothie | извлечение основного текста |

Подробнее — [docs/architecture.md](docs/architecture.md).

Сборок планируется две: `tolearn` и `tolearn-with-speech`. Вторая принимает
зачёт голосом, распознавая речь локально, и весит заметно больше за счёт модели
([ADR-010](docs/adr/010-speech-distribution.md)).

## Формат программы

Приложение читает бандлы семейства `learning-roadmap`: `roadmap.yaml`
(`learning-roadmap/v1`) + `topics/*.yaml` (`learning-roadmap/topic/v1`) +
`progress.yaml` (`learning-roadmap/progress/v1`) + `examiner.md`. Пример — в
[`examples/llm-agents-base`](examples/llm-agents-base).

Промпт, которым такие бандлы генерируются, поставляется вместе с приложением.

## Документация

- [Руководство пользователя](docs/guide.md)
- [Архитектура: ограничения, бюджеты, структура](docs/architecture.md) · [Решения (ADR)](docs/adr/)
- [Спецификации](docs/specs/README.md)
- [Протокол зачёта](docs/protocol.md)
- [Конспекты](docs/notes.md)
- [Офлайн-режим](docs/offline.md)
- [Дизайн-система](docs/design/visual-system.md) · [Навигация](docs/design/navigation.md)

## Лицензия

[GPL-3.0](LICENSE)
