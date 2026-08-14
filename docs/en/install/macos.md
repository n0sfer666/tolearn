# Installing on macOS

[Русский](../../ru/install/macos.md) · **English**

You need an Apple Silicon Mac (M1 or newer): only the `aarch64` build is produced,
there is no Intel version. Minimum system is macOS 10.13; for the
`tolearn-with-speech` variant it is 10.15 (the bar is raised by whisper.cpp, not
by the app).

## How to install

1. Download one file from the
   [releases page](https://github.com/n0sfer666/tolearn/releases):
   `tolearn_<version>_aarch64.dmg` or
   `tolearn-with-speech_<version>_aarch64.dmg`. Do not install both — they fight
   over the `tolearn://` scheme.
2. Open the `.dmg` with a double click and drag `tolearn` into Applications.
3. The image can be ejected and deleted — the app is already copied.

## First launch: "cannot verify the developer"

The installer is neither signed with an Apple certificate nor notarised, so
Gatekeeper will refuse to open it on a double click. To get past it:

- **right-click** `tolearn` in Applications → "Open" → "Open" once more in the
  dialog. The system remembers the choice, and afterwards the app launches the
  usual way;
- if the dialog has no "Open" item — System Settings → Privacy & Security, near
  the bottom there will be a line about `tolearn` being blocked and an "Open
  Anyway" button.

Do this only for a file downloaded from the project's releases page.

## Check that it works

Launch the app: a window with the home screen and the "Choose folder" / "Choose
archive" buttons should appear. From there — the [guide](../guide.md).

To check that the link scheme got registered (the app has to be running):

```sh
open "tolearn://topic?roadmap=demo&topic=demo"
```

The app comes to the front. That there is no `demo` program is fine — what is
being checked is the handling of the link itself.

## Updating and uninstalling

Updating works the same way: download the new `.dmg` and replace the app in
Applications. To uninstall, drag `tolearn` to the Trash; your data stays where it
was ([exactly where](README.md#where-your-data-lives)).
