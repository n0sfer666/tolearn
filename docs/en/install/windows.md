# Installing on Windows

[Русский](../../ru/install/windows.md) · **English**

You need Windows 10 or 11, x64. The app draws its interface in the system WebView2:
Windows 11 always has it, Windows 10 usually does — and if it does not, the
installer installs it.

## How to install

1. Download one file from the
   [releases page](https://github.com/n0sfer666/tolearn/releases):
   `tolearn_<version>_x64_en-US.msi` or
   `tolearn-with-speech_<version>_x64_en-US.msi`. Do not install both — they fight
   over the `tolearn://` scheme.
2. Run the `.msi` with a double click and go through the wizard. Administrator
   rights are not needed if you install for the current user.
3. The app appears in the Start menu as `tolearn`.

## First launch: SmartScreen

The installer is not signed with an Authenticode certificate, so SmartScreen shows
its blue "Windows protected your PC" window. To get past it: "More info" → "Run
anyway". Do this only for a file downloaded from the project's releases page.

Antivirus software may additionally hold the `.msi` for scanning — expected for an
unsigned installer.

## Check that it works

Start `tolearn` from the Start menu: a window with the home screen and the "Choose
folder" / "Choose archive" buttons should appear. From there — the
[guide](../guide.md).

To check that the link scheme got registered (the app has to be running), in
PowerShell:

```powershell
Start-Process "tolearn://topic?roadmap=demo&topic=demo"
```

The app comes to the front. That there is no `demo` program is fine — what is
being checked is the handling of the link itself.

## Updating and uninstalling

To update, run the new `.msi`; it replaces the installed version. To uninstall:
Settings → Apps → `tolearn` → Uninstall; your data stays where it was
([exactly where](README.md#where-your-data-lives)).
