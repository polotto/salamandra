# Vendor software features

**English** | [Português](official-features.pt-BR.md)

Definition of "100%" for v1. Source: user interface text of the vendor
installer (screens and options), and the code of `hid.exe` (communication) and
`ldcfg.exe` (user interface), read by reverse engineering. Each row says where
the feature lives.

Where: **mouse** = mouse memory (`docs/protocol.md`); **PC** = run by the
program on the computer; **OS** = operating system setting.

| Screen | Feature | Where | Status in Salamandra |
| --- | --- | --- | --- |
| Profiles | 5 profiles, switch the active one | mouse `0x2C` | core done |
| Profiles | save profile to a file, load a file | PC | done (own JSON format, validated on load) |
| Profiles | restore current profile to factory | mouse | done (app: loads into the editor and writes) |
| Profiles | reset all profiles | mouse | done (app, with confirmation) |
| General | polling rate 125/250/500/1000 Hz | mouse `0x32 + 2p` | core done |
| General | scroll speed (lines per step) | mouse `0x20 + 2p` | core done |
| General | pointer speed | OS | to do (Windows, macOS, Linux) |
| General | acceleration | OS | to do |
| General | double click speed | OS | to do |
| DPI | 5 levels, enable and disable each one | mouse `base + 4 + 6i` | core done |
| DPI | separate X and Y ("link XY") | mouse | core done |
| DPI | active level | mouse `base + 2` | core done |
| Lighting | static, breathing, blinking | mouse `0x449 + 8p` | core done |
| Lighting | palette color and custom color | mouse | core done |
| Lighting | brightness low, medium, high | mouse `0x44F + 8p` | core done |
| Lighting | breathing and blinking speed (low, medium, high) | mouse | core done |
| Buttons | left, right, middle click, forward, back | mouse | core done |
| Buttons | single key | mouse | core done |
| Buttons | key combo (Win, Ctrl, Shift, Alt + key) | mouse | core done |
| Buttons | basic: cut, copy, paste, select all, find, new, print, save | PC (mouse notifies, PC runs it) | service done (`salamandra-service`); tested on macOS |
| Buttons | advanced: switch window, close window, open file manager, run, show desktop, lock, browser (home, back, forward, stop, refresh, search, favorites), e-mail | PC | service done; Linux X11 only |
| Buttons | media: play/pause, stop, previous, next, volume +/-, mute, player | mouse | core done ("player" still to map) |
| Buttons | macro, with repeat count or repeat while held | mouse | core done |
| Buttons | rapid fire: L/R/M button or key, times, delay | mouse | core done |
| Buttons | DPI: cycle, +, - | mouse | core done |
| Buttons | switch profile | mouse | core done |
| Buttons | polling +, - | mouse | core done |
| Buttons | scroll up, scroll down, switch LED mode | mouse | core done |
| Buttons | disable | mouse | core done |
| Buttons | rule: at least one left click | PC (validation) | core done |
| Macros | up to 15 macros, 63 actions each | mouse `0x478 + 200i` | core done |
| Macros | record keyboard with real timing between actions | PC (recording) + mouse | done (app) |
| Macros | insert default delay, or record without delay | PC + mouse | done (app) |
| Macros | mouse actions: L, R, M (press and release) | mouse | core done |
| Macros | edit: new, delete, record later, move, copy, cut, paste | PC | done (app) |
| Macros | macro name | PC (computer only) | done (app) |
| Tray | tray icon, open settings, quit | PC | done |
| Tray | show on screen what changes from the mouse buttons (profile, DPI, polling, LED) | PC (report `0x08`) | done (profile, DPI and polling; LED notice only) |
| Info | software version, model | PC | done (About screen) |

Outside the vendor software and outside v1: firmware update, firmware version
(the vendor software only shows the program version).
