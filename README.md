# Salamandra

[![CI](https://github.com/polotto/salamandra/actions/workflows/ci.yml/badge.svg)](https://github.com/polotto/salamandra/actions/workflows/ci.yml)

**English** | [Português](README.pt-BR.md)

Open source configurator for gaming mice on Windows, Linux and macOS.

It sets lighting, DPI, polling rate, profiles, buttons and macros straight in
the mouse memory, without the vendor software, which only exists for Windows.

## Status

Test version (pre-release). Download the installer for your system from
[Releases](https://github.com/polotto/salamandra/releases) and see
[How to install](#how-to-install).

## Compatibility

### Mice

| Mouse | USB | Status |
| --- | --- | --- |
| Redragon M901-1 Perdition 2 | `04d9:fc40` | supported and tested |
| Redragon M908 Impact | `04d9:fc4d` | untested: mapped from the official software and public docs ([details](docs/models/m908-impact.md)) |
| Redragon M711 Cobra | `04d9:fc30` | untested: mapped from the official software and public docs ([details](docs/models/m711-cobra.md)) |

**Untested** means the model was never checked on the physical mouse. The app
shows a banner and the command line prints a warning before every write. It
writes anyway, so back up first (`salamandra backup`) and report what happened
with the "Mouse support" issue form. Other models are not recognized yet: each
one must be mapped first, because writing to the wrong address can leave the
mouse behaving unexpectedly. Have another mouse and want
to help? Open an issue asking for support for that model (see
[CONTRIBUTING.md](CONTRIBUTING.md)).

To find the USB ID of your mouse: on Linux, `lsusb`; on macOS, System
Information > USB; on Windows, Device Manager > mouse properties > Details >
Hardware Ids.

### Systems

| System | Configure the mouse | Shortcuts run on the PC | Tested with the mouse |
| --- | --- | --- | --- |
| macOS 13 or newer | yes | yes (asks for Accessibility permission) | yes (Apple Silicon) |
| Windows 10 and 11 | yes | yes | not yet |
| Linux | yes (needs the udev rule) | X11 only; Wayland not yet | not yet |

"Not yet" means it builds and passes the automated tests, but nobody has tested
it with the mouse on that system. If you did, tell us in an issue.

### Vendor software features

| Feature | Status |
| --- | --- |
| 5 profiles and active profile switch | done |
| Polling rate (125, 250, 500, 1000 Hz) and scroll speed | done |
| DPI: 5 levels from 100 to 24000, separate X and Y, active level | done |
| Lighting: static, breathing, blinking, off, color, brightness and speed | done |
| Buttons: clicks, key, key combo, media, macro, rapid fire, DPI, profile, polling, scroll, LED mode, disable | done |
| Shortcuts (copy, paste, browser, lock...), run by the PC | done |
| Macros: 15, up to 63 actions, keyboard recording with real timing, editing | done |
| Save and load profile to a file, backup, restore, factory reset | done |
| Tray icon and on screen notices from the mouse (profile, DPI, polling) | done |
| Pointer speed, acceleration and double click (system settings) | to do |

The full list, with where each feature lives in the mouse, is in
[docs/official-features.md](docs/official-features.md).

## How to install

From the [Releases](https://github.com/polotto/salamandra/releases) page,
download:

| System | File |
| --- | --- |
| Windows | `Salamandra_x.y.z_x64-setup.exe` or `Salamandra_x.y.z_x64_en-US.msi` |
| macOS (Intel and Apple Silicon) | `Salamandra_x.y.z_universal.dmg` |
| Linux (Debian, Ubuntu) | `Salamandra_x.y.z_amd64.deb` |
| Linux (Fedora, openSUSE) | `Salamandra-x.y.z-1.x86_64.rpm` |
| Linux (others) | `Salamandra_x.y.z_amd64.AppImage` |

The installers are not signed, so the system warns you the first time:

- **Windows**: on the SmartScreen warning, click "More info" and "Run anyway".
- **macOS**: drag the app to Applications. The first time, open System
  Settings > Privacy & Security and click "Open Anyway". If macOS says the app
  is damaged, run
  `xattr -dr com.apple.quarantine /Applications/Salamandra.app`.
- **Linux**: the `.deb` and `.rpm` already install the udev rule. With the
  AppImage, install the rule by hand (see below) and make the file executable.

## How to build

You need stable [Rust](https://rustup.rs) and [Node.js](https://nodejs.org) 22
or newer. On Linux, also the Tauri and hidapi libraries:

```sh
sudo apt-get install libudev-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev
```

App:

```sh
cd app
npm install
npm run tauri dev                  # to try it
npm run tauri build                # installer for your system
```

Command line and MCP server, with no Tauri, Node.js, WebKit or libxdo (on
Linux only `libudev-dev`):

```sh
./run.sh cli                       # target/release/salamandra and salamandra-mcp
target/release/salamandra --help
target/release/salamandra show
```

### MCP server

`salamandra-mcp` lets an agent read and set the mouse. It speaks the Model
Context Protocol over stdio, one JSON message per line. Register it in Claude
Code with:

```sh
claude mcp add salamandra -- /full/path/to/salamandra-mcp
```

Tools: `mouse_show`, `mouse_macro_get`, `mouse_backup`, `mouse_set_profile`,
`mouse_set_led`, `mouse_set_dpi`, `mouse_set_active_dpi_level`,
`mouse_set_polling_rate`, `mouse_set_scroll`, `mouse_set_button`,
`mouse_set_macro`, `mouse_reset` and `mouse_restore`. Everything except the two
reads writes to the mouse memory; `mouse_reset` and `mouse_restore` replace the
whole configuration and are marked as destructive, so clients ask before
running them. The server uses the same checks as the CLI, so it also refuses to
leave a profile without a left click. Make a `mouse_backup` first.

### Linux: mouse access without root

Only for the AppImage or a local build (the `.deb` and `.rpm` already do this):

```sh
sudo cp packaging/linux/70-salamandra.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Then unplug the mouse and plug it back in.

### macOS: shortcuts

For the shortcuts (copy, paste, browser) to work, give Salamandra permission in
System Settings > Privacy & Security > Accessibility. Configuring the mouse
needs no permission.

## Before you change anything: back up

From the About screen of the app, or from the command line:

```sh
cargo run -p salamandra-cli -- backup my-mouse.json
cargo run -p salamandra-cli -- restore my-mouse.json
```

## Why "Salamandra"

In European legends, the salamander is the creature that lives in fire. Pliny
the Elder wrote that it walked through flames without burning, and Paracelsus
made it the elemental spirit of fire. A fire being, red as embers, from the
same family of legends as dragons, but small and with no owner.

The name recalls the world of red dragons without using anyone's brand.

## Contributing

Fixes, tests on other systems and support for new mice are welcome. Read
[CONTRIBUTING.md](CONTRIBUTING.md) first.

## Disclaimer

Independent project, not affiliated with Redragon or any manufacturer. Brand
and product names belong to their owners and appear only to say which mice the
project is compatible with.

The configuration protocol was documented by reverse engineering, for
interoperability. This repository has no code, binaries, images or text from
the vendor software.

Use at your own risk: writing a wrong configuration can leave the mouse
behaving unexpectedly. Back up before making changes.

## License

MIT. See [LICENSE](LICENSE).
