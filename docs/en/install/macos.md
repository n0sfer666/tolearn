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

## First launch: "Apple could not verify the app"

The app is signed ad-hoc — that is, without an Apple certificate and without
notarisation. The first launch of a downloaded file is stopped by Gatekeeper with
a dialog saying «"tolearn" Not Opened — Apple could not verify "tolearn" is free
of malware», offering "Move to Trash" and "Done". The way through:

1. Press **"Done"** in the dialog — not "Move to Trash".
2. Open System Settings → Privacy & Security and scroll down to the "Security"
   section: a line about `tolearn` being blocked and an **"Open Anyway"** button
   will be waiting there.
3. Confirm with your password or Touch ID, then press "Open" in the next dialog.

After that the app launches on a plain double click — the decision is remembered.
Right-click → "Open" no longer gets you past this on recent macOS versions; go
through the settings.

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
