# Custom actions

Change what each of the mouse's built-in shortcuts does on your computer:
key combos, Mission Control, a Hyper Key, open an app or URL, run a command,
or a timed sequence. Inspired by BetterTouchTool.

## How it works

The mouse only tells the PC when one of its 22 built-in shortcuts is pressed
(Copy, Paste, Show Desktop...). Assign one of them to a mouse button in the
**Buttons** tab, then bind it to a custom action in `custom-actions.json`. The
background service runs your action instead of the built-in one.

Run the service with the file:

```
SALAMANDRA_ACTIONS=custom-actions.json ./target/release/salamandra-service
```

A missing file is fine (built-in shortcuts keep working). An invalid file is
refused with the reason, and nothing changes.

## File format

```json
{
  "format": "salamandra-actions",
  "version": 1,
  "bindings": {
    "ShowDesktop": { "type": "Builtin", "name": "MissionControl" },
    "Mail":        { "type": "Hyper" },
    "Find":        { "type": "Keys", "modifiers": ["Cmd", "Shift"], "key": "4" }
  }
}
```

Keys of `bindings` are the shortcut names: `Cut`, `Copy`, `Paste`, `SelectAll`,
`Find`, `New`, `Print`, `Save`, `SwitchWindow`, `CloseWindow`,
`OpenFileManager`, `Run`, `ShowDesktop`, `Lock`, `BrowserHome`, `BrowserBack`,
`BrowserForward`, `BrowserStop`, `BrowserRefresh`, `BrowserSearch`,
`BrowserFavorites`, `Mail`.

## Actions

| `type` | Fields | What it does |
| --- | --- | --- |
| `Keys` | `modifiers` (`Ctrl`, `Shift`, `Alt`, `Cmd`), `key` | Holds the modifiers, taps the key |
| `Builtin` | `name`: `MissionControl`, `Launchpad` (macOS), `ShowDesktop` | Ready-made action, resolved per system |
| `Hyper` | none | Holds Ctrl+Alt+Shift+Cmd while the button is held |
| `OpenApp` | `name` | Opens an app (`open -a` on macOS) |
| `OpenUrl` | `url` | Opens with the default program |
| `Command` | `argv` | Runs a program with arguments, no shell |
| `Wait` | `ms` | Pause, only inside a `Sequence` (10 s total) |
| `Sequence` | `steps` (1 to 64) | Runs the steps in order. No nesting, no `Hyper` |

`key` accepts a single character, `f1` to `f12`, `space`, `tab`, `escape`,
`enter`, `backspace`, `delete`, `left`, `right`, `up`, `down`, `home`, `end`,
`pageup` and `pagedown`.

While `Hyper` is held, any `Keys` action reuses the held modifiers, so Hyper
plus a key from another button works. The keys are released when the button is
released or the mouse disconnects.

## Limits

- Only the 22 built-in shortcuts reach the PC. Plain clicks and key combos
  written to the mouse never do.
- The Hyper release uses the mouse's generic "released" notice. Holding two
  shortcut buttons at once is untested on hardware.
- On Windows, Hyper pressed and released with no other key triggers the Office key shortcut.
- macOS needs the Accessibility permission. Linux works on X11, not Wayland.
