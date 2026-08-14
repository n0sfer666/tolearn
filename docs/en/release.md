# Releases: which build to download

[Русский](../ru/release.md) · **English**

The app ships in two variants. They are the same source tree built with one
differing flag ([ADR-010](../adr/010-speech-distribution.md), in Russian): speech
recognition needs a local model, that model weighs twenty times more than
everything else in the app, and only people who want voice should pay for it.

| Variant | What's inside | Who it's for |
|---|---|---|
| `tolearn` | everything except speech recognition | the default — take this one |
| `tolearn-with-speech` | plus whisper.cpp and the `ggml-small-q5_1` weights | those who take the exam out loud |

Functionally the two differ by exactly one screen. In the base variant the voice
exam is **visible and disabled**: the button is there, and next to it a line says
the other variant is required. Everything else — programs, topics, practice,
notes, the offline archive, the graph, export — is identical.

## Where to download

The installers live on the
[releases page](https://github.com/n0sfer666/tolearn/releases): six files per
release, one per variant and OS.

| Variant | OS | File |
|---|---|---|
| `tolearn` | macOS arm64 | `tolearn_<version>_aarch64.dmg` |
| `tolearn` | Windows x64 | `tolearn_<version>_x64_en-US.msi` |
| `tolearn` | Linux x64 | `tolearn_<version>_amd64.deb` |
| `tolearn-with-speech` | macOS arm64 | `tolearn-with-speech_<version>_aarch64.dmg` |
| `tolearn-with-speech` | Windows x64 | `tolearn-with-speech_<version>_x64_en-US.msi` |
| `tolearn-with-speech` | Linux x64 | `tolearn-with-speech_<version>_amd64.deb` |

They are not built by hand: a `v<version>` tag starts the `release` workflow, which
runs the same build matrix as CI and puts all six installers into a draft release.
A tag on a commit whose CI is not green, or one that disagrees with `version` in
`app/tauri.conf.json`, is rejected before the build starts. A human publishes the
draft — until then the links above lead nowhere.

## How much they weigh

The numbers are a measurement, not an estimate: `scripts/weigh.sh` prints them in
the `package` job next to the artifact it just built, for every variant and every
OS.

| Variant | OS | Format | Weight | Ceiling |
|---|---|---|---|---|
| `tolearn` | macOS arm64 | `.dmg` | 8.6 MB | 12 MB |
| `tolearn` | Windows x64 | `.msi` | 9.5 MB | 14 MB |
| `tolearn` | Linux x64 | `.deb` | 12.8 MB | 14 MB |
| `tolearn-with-speech` | macOS arm64 | `.dmg` | 186.5 MB | no ceiling |
| `tolearn-with-speech` | Windows x64 | `.msi` | 185.8 MB | no ceiling |
| `tolearn-with-speech` | Linux x64 | `.deb` | 189.0 MB | no ceiling |

All six numbers are from 2026-08-09, version 0.1.0, all from the `package` job of
a single green run. When the weight changes, so does this table: it holds the
latest measurement, not the first one.

The base variant's ceilings are [budgets](../architecture.md#бюджеты) (in
Russian), and going over one fails the build. `-with-speech` has no installer
ceiling: its weight is set by the model weights, which have a budget of their own
(≤ 192 MB). The difference between the variants *is* that model: 190,085,487
bytes, inside the installer, with nothing to download afterwards.

## Which one to install, and how to switch

Both variants are one application with one identifier, `dev.tolearn.app`, and so:

- **install exactly one.** Two installed variants fight over the `tolearn://`
  scheme and over a single window; the OS picks the winner itself, and it will not
  be the choice you were asked about.
- **the data is shared and survives the switch.** Programs, progress and notes
  live outside the bundle
  ([ADR-009](../adr/009-user-data-outside-bundle.md), in Russian): remove one
  variant, install the other, nothing is lost.

On macOS `-with-speech` requires 10.15 or newer, while the base variant makes do
with 10.13. The bar is raised by whisper.cpp, not by the app: it is built against
`std::filesystem`, which libc++ lacks before 10.15.

The installer registers the `tolearn://` scheme itself: a link like
`tolearn://topic?roadmap=<program>&topic=<topic>` opens that topic in the already
running app.

## Building the variants yourself

One tree, one differing flag:

```sh
cd app
cargo tauri build                                                   # tolearn
cargo tauri build --features speech --config tauri.with-speech.conf.json
```

The second command needs the `speech/vendor/whisper.cpp` submodule and the
`app/models/ggml-small-q5_1.bin` file — the weights that go into the installer as
a resource. To weigh what you built:
`sh scripts/weigh.sh target/release/bundle base` (or `with-speech`). Full details
are in [install/source.md](install/source.md).
