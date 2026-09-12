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
