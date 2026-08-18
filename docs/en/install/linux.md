# Installing on Linux

[Русский](../../ru/install/linux.md) · **English**

Only one package is built: a `.deb` for x64 — that is, Debian, Ubuntu and their
derivatives. For other distributions and for ARM there is
[building from source](source.md).

## Dependencies

The interface is drawn by WebKitGTK, so its runtime libraries must be present.
`apt` pulls them in itself when you install the package through it (see below); to
install them beforehand:

```sh
sudo apt update
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0
```

The `tolearn-with-speech` variant additionally needs ALSA (`libasound2`) — present
on any desktop system; if the microphone is not visible, install the package
explicitly.

## How to install

1. Download one file from the
   [releases page](https://github.com/n0sfer666/tolearn/releases):
   `tolearn_<version>_amd64.deb` or `tolearn-with-speech_<version>_amd64.deb`. Do
   not install both — they fight over the `tolearn://` scheme.
2. Install the package together with its dependencies:

   ```sh
   sudo apt install ./tolearn_<version>_amd64.deb
   ```

   `dpkg -i` works too, but then dependencies are on you
   (`sudo apt -f install`).
3. The app appears in the menu as `tolearn`; from a terminal it runs as `tolearn`.

The package is not signed — `apt` does not verify it against a key, you are
installing a file you trust yourself. Download it only from the project's releases
page.

## If the window is blank or does not open

Almost always this is WebKitGTK. Run it from a terminal and look at the output:

```sh
tolearn
```

- an error about `libwebkit2gtk-4.1.so` — the dependency did not get installed, go
  back to the first step;
- a blank white window on Nvidia drivers and in some VMs is cured by turning off
  hardware-accelerated compositing:

  ```sh
  WEBKIT_DISABLE_COMPOSITING_MODE=1 tolearn
  ```

  If that helps, put the variable into your `.desktop` file or into the session
  environment.

## Check that it works

Launch the app: a window with the home screen and the "Choose folder" / "Choose
archive" buttons should appear. From there — the [guide](../guide.md).

To check that the link scheme got registered (the app has to be running):

```sh
xdg-open "tolearn://topic?roadmap=demo&topic=demo"
```

The app comes to the front. That there is no `demo` program is fine — what is
being checked is the handling of the link itself.

## Updating and uninstalling

To update, install the new `.deb` with the same command; it replaces the installed
one. To uninstall:

```sh
sudo apt remove tolearn
```

Your data stays where it was ([exactly where](README.md#where-your-data-lives)).
