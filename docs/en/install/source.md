# Building from source

[Русский](../../ru/install/source.md) · **English**

This is the route for anyone the ready-made installers do not fit: an Intel Mac,
ARM Linux, a non-Debian distribution, or simply the wish to build it yourself. The
result is the same installer that ships in a release — CI builds it with the very
commands on this page.

## What you need

| Tool | Version | What for |
|---|---|---|
| Rust | taken from `rust-toolchain.toml` (1.97.1) | the core and the app |
| Node.js | 24 | building the interface |
| pnpm | 10 | interface dependencies |
| Tauri CLI | 2.11.4 | building the installer |

Install Rust via [rustup](https://rustup.rs) — it picks the right version out of
`rust-toolchain.toml` itself. The Tauri CLI:

```sh
cargo install tauri-cli --version 2.11.4 --locked
```

Plus system dependencies:

- **macOS** — Command Line Tools: `xcode-select --install`.
- **Windows** — Visual Studio Build Tools with the "Desktop development with C++"
  workload, and WebView2 (already present on Windows 11).
- **Linux (Debian/Ubuntu)**:

  ```sh
  sudo apt update
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev patchelf
  ```

  For the speech variant, `libasound2-dev` as well.

## Build

```sh
git clone https://github.com/n0sfer666/tolearn.git
cd tolearn
pnpm -C ui install --frozen-lockfile
cd app
cargo tauri build
```

The interface builds itself: `beforeBuildCommand` in `tauri.conf.json` runs
`pnpm -C ui build` before compilation. The artefacts land in
`target/release/bundle/` at the repository root: `.dmg` on macOS, `.msi` on
Windows, `.deb` on Linux. Install them like any installer, following the page for
your OS ([macOS](macos.md), [Windows](windows.md), [Linux](linux.md)).

## The speech variant

It needs the whisper.cpp submodule and the model weights — the weights file is not
in the repository and is placed by hand:

```sh
git submodule update --init --recursive
# app/models/ggml-small-q5_1.bin — put it there yourself
cd app
cargo tauri build --features speech --config tauri.with-speech.conf.json
```

How the two variants differ and why they are not installed side by side is in
[release.md](../release.md).

## Running without installing

For development, `cargo tauri dev` from `app` (it also starts the interface dev
server). To simply look at the built app without producing an installer:

```sh
pnpm -C ui build
cargo run -p tolearn-app --release --bin tolearn-desktop
```

The interface here is taken from `ui/dist`, so the first command is mandatory:
without it the app compiles but the window stays blank.

Such a run puts its data in the same place as an installed app
([exactly where](README.md#where-your-data-lives)) — it has no sandbox of its own.

The pages are embedded into the binary (the `custom-protocol` feature, on by
default); to run against the Astro dev server, pass `--no-default-features`.

## The CLI and the Obsidian plugin

Neither ships in the installer; both are built separately. The windowless CLI —
validate a bundle, list topics, export Markdown:

```sh
cargo run -p tolearn-cli --release -- validate examples/llm-agents-base
```

The Obsidian plugin:

```sh
pnpm -C obsidian install
pnpm -C obsidian build
```

Then copy the `obsidian/` directory (`main.js` and `manifest.json` are required)
into `<vault>/.obsidian/plugins/tolearn/`. What it does is in
[notes.md](../notes.md#the-obsidian-plugin).
