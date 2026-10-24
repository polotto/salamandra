# Redragon M908 Impact (`04d9:fc4d`)

Status: **untested**. Never checked on the physical mouse. The app and the CLI write directly, with a warning.

## Where each field comes from
| Field | Value | Source |
| --- | --- | --- |
| Family / sensor | 3 / 3327 (`0xCFF`) | official software `RDCfg.exe`, `FUN_00431a30` and `FUN_004314d0` |
| Memory map | same as the M901-1: profile bases, DPI at `base+4+6n`, polling at `0x32`, LED at `0x449`, macros at `0x478+200n` | `mouse_m908` capture of this model (addresses only) |
| Buttons | 20 per profile, same slots and addresses as the M901-1 (the two wheel slots at `0x16` and `0x17`) | `mouse_m908` capture |
| DPI | 93 values: 100 to 6200 in steps of 100, 6400 to 12400 in steps of 200. One code byte plus a flag per axis | official tables (`FUN_00457b30`, sensor `0xCFF`); codes for 200 to 12400 match the `mouse_m908` capture, 100 DPI is only in the official table |
| LED | `r g b mode speed sub`: off `0 0 0`, steady `1 0 2`, breathing `1 v 4`, rainbow `1 8 8`, wave `2 v 0`, alternating `6 v 0`, reactive `7 v 0`, flash `1 v 0x10`. `v` is the firmware byte, `9 -` the interface value, as in the M901-1 | the eight positions of family 3 in `FUN_00456d90`; the names come from the `mouse_m908` capture, which agrees on every pair |
| LED group | before the LED we write `00 00` at `0x446` | `FUN_00456d90` writes it, with zeros for this model (a non-zero value is only set for another model, "2850"); the `mouse_m908` capture writes the same packet |

## Differences from the M901-1
- DPI table and range.
- LED mode bytes (family 3).

## Still unconfirmed
- Steady: the official software writes speed `0`, the `mouse_m908` capture writes `8`. We follow the official one. The firmware probably ignores it.
- LED modes seen only in the `mouse_m908` capture, not offered by the official software: breathing rainbow (`1 v 1`), button reactive (`3 v 0`) and random (`4 v 0`). We do not offer them.
- The official table has 21 more entries after 12400 that we do not understand. We stay at 12400.

## How to help
Open the "Mouse support" issue form with the output of `salamandra show` before and after a change, and what you saw on the mouse (LED, DPI, buttons).
