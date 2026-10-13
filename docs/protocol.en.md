# M901-1 Perdition 2 protocol (`04d9:fc40`)

**English** | [Português](protocol.md)

Source: reverse engineering, for interoperability, of the vendor software:
`hid.exe` (talks to the mouse) and `ldcfg.exe` (user interface), with REA 6.4
and Ghidra 12.1.4. Each row cites the source function. Checked by reading the
real mouse on 2026-10-11. The same firmware (Holtek) shows up in
[mouse_m908](https://github.com/dokutan/mouse_m908), which was the second source
for the polling code.

## Device

- USB `04d9:fc40`, product `USB Gaming Mouse` (`FUN_00410fe0`)
- Configuration interface: usage page `0xFFA0`, usage `0x01` (3rd HID interface)
- Feature Reports only. On macOS the buffer includes the report ID in byte 0
  (`IOHIDDeviceSetReport`/`GetReport`, `kIOHIDReportTypeFeature`)
- Report `0x02` = 16 bytes; report `0x03` = 64 bytes (from the HID descriptor)
- No Input Monitoring permission needed

## Packet

```
byte  0     1    2        3        4    5..7  8..
      rid   cmd  addr_lo  addr_hi  len  0     data
```

| cmd | use | source |
| --- | --- | --- |
| `F3` | writes `len` bytes at `addr` | `FUN_0040e9f0` and others |
| `F2` | requests a read of `len` bytes; the answer comes in the GetFeature of the same report: `rid 08 addr_lo 60 len 00 FA FA data...` | `FUN_0040f510` |
| `F5 00` / `F5 01` | opens and closes the write session | `FUN_0040f1f0`, called by `FUN_00412e90` |
| `F1 02 <m>` | applies an area: `01` profile, `02` DPI, `04` buttons, `08` polling, `10` DPI level | `FUN_004102c0`, `FUN_0040ff40`, `FUN_004103c0`, `FUN_004100a0`, `FUN_00412e90` |

Full write by the vendor software (`FUN_00412e90`, "SendAll2Mouse"): `F5 00`,
writes, `F1 02 04`, `F1 02 01`, `F1 02 02`, `F1 02 08`, `F5 01`.

Pitfall: after writing the active profile (`0x2C`), the mouse uses the buttons
of the new profile right away, but the LED only changes on the next `F5`
session. Checked on hardware: profile switch and restore run one extra session
because of this.

## Memory map

Profiles `p = 0..4`, bases `0x040 0x100 0x1B0 0x260 0x310` (table at `0x47E35C`).

| Address | Size | Content | Source |
| --- | --- | --- | --- |
| `0x002C` | 2 | active profile (0..4), `00` | `FUN_004102c0` |
| `0x0020 + 2p` | 1 (+0) | scroll lines (`WheelScrollLines`), report 3 | `FUN_004101e0` |
| `0x0032 + 2p` | 1 (+0) | polling: `08`=125 `04`=250 `02`=500 `01`=1000 Hz | `FUN_004103c0`, mouse_m908 |
| `0x003E` | 2 | always `00 00` | `FUN_0040e960` |
| `0x0446` | 2 | always `00 00` | `FUN_0040e9f0` |
| `base + 2` | 1 (+0) | active DPI level (0..4) | `FUN_004100a0`, `FUN_0040fc50` |
| `base + 4 + 6i` | 5 (+0) | level `i`: `enabled, x, xf, y, yf` | `FUN_0040ff40` |
| `base + 0x42 + 4b` | 4 | action of button `b` (slots `0x12`, `0x13` go to `0x16`, `0x17`) | `FUN_00412980` |
| `0x0449 + 8p` | 6 | LED: `R G B mode speed sub` | `FUN_0040e9f0`, `FUN_0040fa00` |
| `0x044F + 8p` | 1 | LED brightness (1..3) | `FUN_0040e9f0` |

### DPI

`index = DPI / 100` (100 to 24000). Encoding (`FUN_0040ff40`):
`index <= 0x79`: `byte = index - 1`, `flag = 0`. Otherwise `byte = index - 0x3D`,
`flag = 1`. Factory default (`RSProfile1.pfd`): 1000, 2000, 4000, 8000, 12000.

### LED (UI mode -> firmware)

| UI | mode | speed | sub |
| --- | --- | --- | --- |
| static | `01` | 0 | 0 |
| breathing | `02` | 1..8 | 0 |
| blinking | `04` | 1..8 | 0 |
| off | `00` | 0 | 0 |
| `4` (no name in the UI) | `03` | 0 | `03` |

### Buttons (`base + 0x42 + 4b`, table `0x47D3D0`)

| Action | Bytes |
| --- | --- |
| left / right / middle click | `81` / `82` / `83` |
| back / forward | `84` / `85` |
| DPI and profile (several) | `88`, `89`, `8A`, `8B`, `8C`, `8D` |
| key | `90 00 <HID usage> 00` |
| key combo | `8F <mods> <HID usage> 00`, mods `ctrl=1 shift=2 alt=4 win=8` |
| media | `8E 01 <consumer usage>` (`CD` play, `B5`/`B6` track, `E9`/`EA` volume, `E2` mute, `B7` stop) |
| built-in shortcut | `8E 01 FF <id>` |
| rapid fire | `99 <target> <times> <delay/10 ms>`; target `81`/`82`/`84` = L/R/M, any other value = HID usage of the key (`FUN_0040f5e0`) |
| macro | `91 <index, bit7 = repeat> <count_lo> <count_hi>` |
| disabled | `00 00 00 00` |

## Macros (`0x478 + 200i`, 15 slots)

Written with report `0x04` (256 bytes): `04 F3 <addr> C8 00 00 00 <200 bytes>`,
between `F5 00` and `F5 01`, without `F1` (`FUN_004125b0`). Up to 63 events of
3 bytes (the vendor editor stores 189 bytes); the rest is zero.

| Event | Bytes | Source |
| --- | --- | --- |
| key down / up | `84 <HID usage> 00` / `04 <HID usage> 00` | `ldcfg.exe` `FUN_00427030` |
| mouse button down / up | `81 <b> 00` / `01 <b> 00`, b: `1` L, `2` R, `4` M | `ldcfg.exe` `FUN_0042a600` |
| delay | `06 <lo> <hi>`, time in 10 ms units (16 bits), minimum 10 ms | `ldcfg.exe` `FUN_00427030` |

Checked against macros written to the real mouse (Shift twice; Win+V).

## Events the mouse sends to the PC

`hid.exe` listens to the input reports of interface `0xFFA0` (`FUN_00410be0`).

| Report | Meaning | Vendor software does |
| --- | --- | --- |
| `08 01 <p>` | active profile changed by the mouse button | switches the profile on screen |
| `08 02 <n>` | DPI level changed (n from 0 to 4; checked on the mouse: `08 02 04`, `08 02 03`) | shows the level |
| `08 06 <code>` | polling changed (`1/2/4/8`) | updates the screen |
| `08 09 ...` | LED mode changed | updates the screen |
| `01 FF <id>` then `01 00 00` | button with a built-in shortcut (`8E 01 FF <id>`) pressed and released; comes in the consumer report (ID 1). Checked on the mouse: `01 ff 12` for copy | runs it on the PC (table below) |

Shortcuts run by the PC (`hid.exe` `FUN_00401050` onwards): `0x11` Ctrl+X,
`0x12` Ctrl+C, `0x13` Ctrl+V, `0x14` Ctrl+A, `0x15` Ctrl+F, `0x16` Ctrl+N,
`0x17` Ctrl+P, `0x18` Ctrl+S, `0x19` Alt+Tab, `0x1A` Alt+F4, `0x1B` open file
manager, `0x1C` run, `0x1D` show desktop, `0x1E` lock, `0x1F` browser home
page, `0x20`..`0x25` browser keys (back, forward, stop, refresh, search,
favorites), `0x26` e-mail client.

## Factory defaults

From the installer profiles, checked against the mouse: 500 Hz, 1 scroll line,
DPI 1000/2000/4000/8000/12000 all enabled with the active level of profile `p`
equal to `p`, static LED brightness 2, colors red/blue/green/magenta/yellow.
Buttons: L, R, M, rapid fire L 3x, DPI+, DPI-, keys 1..0, numeric keypad `+`
and `-`, scroll up and down. See `crates/salamandra-core/src/defaults.rs`.
