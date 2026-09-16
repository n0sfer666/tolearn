# Чужое в этом репозитории

Сам tolearn — GPL-3.0-or-later. Ниже то, что лежит в дереве или скачивается
рядом с ним и живёт по своей лицензии. Обычные зависимости с crates.io и npm
сюда не переписываются: их лицензии перечисляют `Cargo.lock` и `pnpm-lock.yaml`.

## whisper.cpp

Сабмодуль `speech/vendor/whisper.cpp`, тег v1.9.2, коммит `306c88f4d1`.
Лицензия — MIT, © 2023–2026 The ggml authors, текст лежит рядом с исходниками
в `speech/vendor/whisper.cpp/LICENSE`. Собирается в статические библиотеки и
линкуется в вариант `-with-speech`; в базовый вариант не попадает.

## Веса распознавания

`ggml-small-q5_1.bin` — модель Whisper small (OpenAI, MIT), сконвертированная в
формат GGML и квантованная в q5_1; берётся из
[huggingface.co/ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp),
лицензия репозитория — MIT. В git весов нет: путь к файлу задаёт
`TOLEARN_WHISPER_MODEL`, в дистрибутив `-with-speech` они кладутся при упаковке.

## Аудиофикстуры

`fixtures/audio/` — восемь записей, переведённых в 16 кГц моно PCM; текст рядом
в `*.txt` — эталон из того же корпуса. Правки в записи не вносились, кроме
пересэмплирования и сведения в моно.

| Префикс | Источник | Лицензия |
|---|---|---|
| `librispeech-` | [LibriSpeech ASR](https://huggingface.co/datasets/openslr/librispeech_asr), срез `clean/test`, начитка книг из Project Gutenberg | CC BY 4.0 |
| `rudevices-` | [SOVA RuDevices](https://huggingface.co/datasets/bond005/sova_rudevices), срез `test`, живая русская речь | CC BY 4.0 |

Новый префикс в `fixtures/audio/` без строки в этой таблице роняет
`speech/tests/attribution.rs`.

## Mermaid

`app/vendor/mermaid/mermaid.tiny.js` — файл `dist/mermaid.tiny.js` из пакета npm
`@mermaid-js/tiny` 12.0.0, sha256
`9f2807e402479d2864bfd9a95052076fc69d0ff45a0b25148b84b70fc33425b9`. Лицензия —
MIT, © 2014–2022 Knut Sveidqvist, текст рядом в `app/vendor/mermaid/LICENSE`.
Вшивается в бинарник `app` и рисует схемы этапов в скрытом окне во время
генерации (S117). На страницы урока не попадает: там лежит готовый SVG. Файл
сверяется с этой записью в `app/tests/mermaid.rs`, и другая сборка без правки
записи роняет тест.

Взята облегчённая сборка, а не `mermaid`: полная весит 1.52 МиБ в gzip, а у
`.deb` до потолка 14 МиБ оставалось около 1.2 МиБ. В облегчённой нет mindmap,
architecture и формул KaTeX. Такая схема не рисуется и остаётся в уроке
исходником в блоке `code`.

## Корзина ОС

Крейт `trash` 5.2.9 с crates.io: MIT, © 2019 Artúr Barnabás Kovács, текст в
`LICENSE.txt` внутри пакета. Правило «обычные зависимости сюда не
переписываются» для него не работает: он уносит данные пользователя за пределы
каталога приложения, и способ, которым он это делает, — решение, а не деталь
сборки.

На macOS у крейта два пути. По умолчанию — `DeleteMethod::Finder`: `osascript`
просит Finder убрать файл, и первый же такой вызов поднимает запрос прав
автоматизации. `app/src/discard/bin.rs` ставит `DeleteMethod::NsFileManager` —
`trashItemAtURL:` из `NSFileManager`, вызов внутри процесса, без запроса прав и
без звука Finder; цена — macOS не заполняет «Положить обратно», и вернуть
программу из корзины можно перетаскиванием. Способ пришпилен тестом
`корзина_не_просит_прав_на_управление_finder` в `app/src/discard/bin.rs`. На
Windows и Linux путь один, выбирать нечего.

В `Cargo.lock` крейт привёл `urlencoding` и `windows-*` под свои платформы;
`objc2` и `objc2-foundation` там уже лежали от Tauri. Установщик от этого не
потяжелел: `tolearn-desktop` release весит 16 874 176 байт и с крейтом, и со
снятой зависимостью (замер 2026-09-16, aarch64), `.dmg` — 7.0 МБ при потолке
12 МБ.

## Шрифты

`ui/src/fonts/` — вариативные woff2 с осью `wght` из пакетов npm
`@fontsource-variable/literata` 5.3.0 и `@fontsource-variable/golos-text`
5.3.0, файлы из их каталога `files/` без правок: подмножества латиницы и
кириллицы, у Literata — прямой и курсив. Лицензия — SIL Open Font License 1.1:
Literata — © 2017 The Literata Project Authors, Golos Text — © 2019 The Golos
Text Project Authors; тексты рядом, в `ui/src/fonts/OFL-literata.txt` и
`ui/src/fonts/OFL-golos-text.txt`. Шрифты попадают в `ui/dist`, а с ним — в
бинарник `app` (ADR-019, S147).

| Файл | sha256 |
|---|---|
| `literata-latin-wght-normal.woff2` | `9adbeac5b167fe5ad6c49d9e29aa0c76e2f1bb3b46bf4ebf12a9eca7d3525384` |
| `literata-cyrillic-wght-normal.woff2` | `df20f1a8ca3c15497861c6dbf36c0c59507ee351301527ec89543747067ad4d7` |
| `literata-latin-wght-italic.woff2` | `ab198d6616c7cc966f26a4a5b28a3977dc47439640f09d9b3361226bd465c404` |
| `literata-cyrillic-wght-italic.woff2` | `62e8c9a48487878d0027e5946773f029f28ec6be99f1f20eb26b6aa401395c29` |
| `golos-text-latin-wght-normal.woff2` | `9a69d0aa4734c4022224c002a3d944a702e0204972a49d892789f5668b922c2a` |
| `golos-text-cyrillic-wght-normal.woff2` | `17d048ca05cb1218af3c0d6dcdf882989e6d1cc5dcb598ea50eaf54850ff7229` |

Файл в `ui/src/fonts/` без строки в этой таблице, с другим sha256 или строка
без файла роняют `ui/tests/fonts.test.mjs`.
