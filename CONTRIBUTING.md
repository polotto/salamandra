# How to contribute

**English** | [Português](CONTRIBUTING.pt-BR.md)

Thanks for your interest. This guide covers how to set up the environment, what
to check before opening a pull request and how to ask for or add support for a
mouse. Issues and pull requests in English or Portuguese are welcome.

## Ways to help

- **Test on Windows or Linux** with a supported mouse and report the result in
  an issue: what worked, what did not, system and version.
- **Report a bug**: steps to reproduce, system, mouse model and, if possible,
  the output of `salamandra show`.
- **Ask for support for a mouse**: open an issue with the model, the USB ID
  (`vendor:product`) and a link to the vendor software.
- **Code**: pick an open task in `docs/work/v1-multiplataforma/TASKS.md` or an
  issue, and comment before starting something big.
- **Translation**: the texts live in `app/src/lib/i18n.svelte.ts`.

## Environment

You need stable Rust and Node.js 22 or newer. On Linux, the libraries listed in
the [README](README.md#how-to-build).

```sh
cargo test --workspace                                   # core tests
cargo clippy --workspace --all-targets -- -D warnings    # lint
cargo fmt --all                                          # formatting
cd app && npm install && npm run check                   # front end
node app/scripts/check-i18n.mjs                          # missing translations
cd app && npm run tauri dev                              # app
```

CI runs all of this on Windows, Linux and macOS on every push and pull request.

## Code layout

| Folder | Contents |
| --- | --- |
| `crates/salamandra-core` | protocol, memory map, models and the `Mouse` API. No I/O; tested without a mouse |
| `crates/salamandra-hid` | USB HID transport over `hidapi` |
| `crates/salamandra-service` | service that listens to the mouse and runs the shortcuts on the PC |
| `crates/salamandra-cli` | `salamandra` command line |
| `app` | Tauri 2 app with Svelte 5 |
| `docs/protocol.md` | M901-1 protocol, each field with its source |
| `docs/official-features.md` | vendor software features and the status of each one |

## Important rules

**Mouse safety.** Writing to the wrong address can leave the mouse behaving
unexpectedly.

- Only write addresses and sizes documented in `docs/protocol.md`.
- Never leave a profile without a button set to left click (the core already
  refuses it).
- Every packet builder is a pure function in `salamandra-core` and has a test.
- Before testing a write on a real mouse, back up:
  `salamandra backup my-mouse.json`.

**Nothing from the vendor software goes into the repository.** No decompiled
code, binaries, installers, images, icons, `.ini` files or copied text. Facts
about the protocol (addresses, command codes, factory defaults) go in as our
own data, described in our own words and with the source cited. For the same
reason the project does not use brand names in code, packages or binaries: the
brand only appears to say which mice it is compatible with.

**User interface text** always goes through `t("text in pt-BR")`, with the
English translation in the dictionary. The i18n check fails if one is missing.

**App that does not feel like a web page**: no text selection outside input
fields, no Cmd/Ctrl+A, no right click menu and no browser shortcuts
(`app/src/lib/nativeFeel.ts`).

## Adding support for a mouse

1. **Identify the mouse**: USB ID and HID interfaces. The configuration one
   usually has a vendor usage page (`0xFF00` to `0xFFFF`).
2. **Look for existing work**: open projects for the same chip often have the
   protocol, or part of it.
3. **Study the vendor software** by reverse engineering, only for
   interoperability and only on your machine. Note the source of each field.
4. **Start by reading only**: read the same addresses the vendor software reads
   and compare with what it shows on screen.
5. **Document** the protocol in `docs/`, like `docs/protocol.md`.
6. **Implement** the memory map and the tests in `salamandra-core`, with real
   bytes read from the mouse.
7. **Test writing** with a backup made, one field at a time, checking the
   physical effect (LED, pointer, key), and restore at the end.

None of these steps may bring files from the vendor software into the repo.

## Promoting a model to tested

A model marked `untested` becomes `tested` when a report (issue template
"I tested a model") has all of:

- the output of `salamandra show` before and after a write;
- the physical effect confirmed for LED, DPI and one remapped button;
- a backup restored at the end, with `show` matching the first output.

The change is one line in `crates/salamandra-core/src/spec.rs`
(`Status::Tested`) and the status column in both READMEs and in
`docs/models/<model>.md`, in the same commit. A failed or partial report
does not promote the model: open an issue with what failed instead.

## Commits and pull requests

- Commits in English, following Conventional Commits, without scope:
  `feat: ...`, `fix: ...`, `docs: ...`, `test: ...`, `chore: ...`, `ci: ...`.
- One subject per pull request, with a short description: what changes, why
  and how to test (with or without a mouse).
- CI must pass.
- A change that writes something new to the mouse must say which mouse and
  system it was tested on.
- Documentation changes go in both languages: each `.md` has a `.pt-BR.md`
  counterpart.

## License

By contributing, you agree that your contribution will be published under the
project's MIT license.
