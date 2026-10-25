# Redragon M711 Cobra (`04d9:fc30`)

Status: **untested**. Never checked on the physical mouse. Writing works the
same way as on the M901-1, so the app and the CLI write directly, with a warning.

## Where each field comes from
| Field | Value | Source |
| --- | --- | --- |
| Family / sensor | 0 / 3325 (`0xCFD`) | official software `RDCfg.exe`, `FUN_00431a30` and `FUN_004314d0` |
| Memory map | same as the M901-1: profile bases, DPI at `base+4+6n`, buttons at `base+0x42+4n`, LED at `0x449`, polling at `0x32`, macros at `0x478+200n` | the `mouse_m908` capture of this model (addresses only) and the official software, which shares the code for family 0 |
| DPI | 75 values: 100 to 5000 in steps of 100, 5200 to 10000 in steps of 200. One code byte plus a flag per axis | official table (`FUN_00457b30`, sensor `0xCFD`); all 75 codes match the `mouse_m908` capture |
| Buttons | 10 per profile, slots 1 to 8, 11 and 12 (the capture has no slots 9 and 10) | `mouse_m908` capture |
| Macros | 15 slots of 200 bytes, same addresses | `mouse_m908` capture |

## Differences from the M901-1
- DPI table and range (above).
- Only 10 buttons.

## Still unconfirmed
- What each of the 10 buttons is called on the shell.
- LED modes: the app shows the M901-1 modes. Family 0 is the same layout in the official code, but this was not checked on the mouse.
- The `mouse_m908` capture uses a different LED coding for this model than the official software. We follow the official mode (family 0) with the `sub` byte tested on the M901-1; the multi-model official program (`RDCfg.exe`) writes `1` there, an open doubt.
- The official software writes 5 bytes per DPI level, the old capture 4. We write 5, like the current official software.
- Device version: the same PID `fc30` also covers other models with other sensors (M711-FPS-1, M711-FPS-2, 2710-H). This spec matches the official list only for versions 1 to 767. `salamandra show` prints the version; please include it in your report.

## How to help
Open the "Mouse support" issue form with the output of `salamandra show` before and after a change, and what you saw on the mouse (LED, DPI, buttons).
