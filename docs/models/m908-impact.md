# Redragon M908 Impact (`04d9:fc4d`)

Status: **untested**. Never checked on the physical mouse. The app and the CLI write directly, with a warning.

## Where each field comes from
| Field | Value | Source |
| --- | --- | --- |
| Family / sensor | 3 / 3327 (`0xCFF`) | official software `RDCfg.exe`, `FUN_00431a30` and `FUN_004314d0` |
| Memory map | same as the M901-1: profile bases, DPI at `base+4+6n`, polling at `0x32`, LED at `0x449`, macros at `0x478+200n` | `mouse_m908` capture of this model (addresses only) |
| Buttons | 20 per profile, same slots and addresses as the M901-1 (the two wheel slots at `0x16` and `0x17`) | `mouse_m908` capture |
| DPI | 93 values: 100 to 6200 in steps of 100, 6400 to 12400 in steps of 200. One code byte plus a flag per axis | official tables (`FUN_00457b30`, sensor `0xCFF`); codes for 200 to 12400 match the `mouse_m908` capture, 100 DPI is only in the official table |
| LED | `r g b mode speed sub`: off `0 0 0`, steady `1 8 2`, breathing `1 v 4`, flash `1 v 0x10` | `sub` values 4, 8, 2, 0 and 1 in `FUN_00456d90` (family 3); the mapping to modes comes from the `mouse_m908` capture |

## Differences from the M901-1
- DPI table and range.
- LED mode bytes (family 3).

## Still unconfirmed
- The official software also writes two bytes at `0x446` (`FUN_00456d90`) that depend on the LED mode group. We do not write them.
- Speed direction for breathing and flash (the M901-1 uses `9 - v`, family 3 was not checked).
- LED modes this model has and we do not offer: rainbow, breathing rainbow, wave, alternating, reactive.
- The official table has 21 more entries after 12400 that we do not understand. We stay at 12400.

## How to help
Open the "Mouse support" issue form with the output of `salamandra show` before and after a change, and what you saw on the mouse (LED, DPI, buttons).
