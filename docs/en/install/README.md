# Installation

[Русский](../../ru/install/README.md) · **English**

The app is installed from a ready-made installer for your OS, or built from
source. Install **one** variant — either `tolearn` or `tolearn-with-speech`; the
difference is explained in [release.md](../release.md).

| OS | What to download | Page |
|---|---|---|
| macOS (Apple Silicon) | `.dmg` | [macos.md](macos.md) |
| Windows 10/11, x64 | `.msi` | [windows.md](windows.md) |
| Linux, Debian/Ubuntu, x64 | `.deb` | [linux.md](linux.md) |
| Any of the three | build from source | [source.md](source.md) |

There are no builds for Intel Macs, ARM Linux or 32-bit systems: CI produces
exactly six installers — two variants across three operating systems. On any other
hardware, [building from source](source.md) is the way.

## The installers are unsigned

The project has neither an Apple certificate nor an Authenticode one, so both
macOS and Windows will say the developer could not be verified on first launch.
That is expected and does not mean anything is wrong with the file; how to get
past the warning is on the page for your OS. Bypassing a warning is a deliberate
act: do it only for a file downloaded from this project's
[releases page](https://github.com/n0sfer666/tolearn/releases).

## Where your data lives

The same on all three operating systems — the app does not lay it out differently
per system:

| What | Path |
|---|---|
| Settings, provider, program registry | `~/.config/tolearn` |
| Notes, page archives, unpacked bundles, history | `~/.local/share/tolearn` |

On Windows `~` is `C:\Users\<name>`, i.e. `C:\Users\<name>\.config\tolearn`. The
`XDG_CONFIG_HOME` and `XDG_DATA_HOME` variables are honoured when set to an
absolute path. The learning programs themselves stay wherever you put them: the
app stores the path to them, not the programs.

The data is tied neither to the build variant nor to the way you installed:
switching from `tolearn` to `tolearn-with-speech`, reinstalling, and moving from an
installer to a source build all leave it alone. If you had an early version that
kept everything in the OS application directory (`dev.tolearn.app`), the first
launch of the new one migrates the files itself.

## Uninstalling

Removing the app does not remove the data — that is a separate action:

```sh
rm -rf ~/.config/tolearn ~/.local/share/tolearn
```

Notes in external storage (an Obsidian vault or any other directory you set) live
outside these paths and are yours to delete: the app will not touch them.
