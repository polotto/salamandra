# Redragon M719 Invader (`04d9:fc4f`)

Status: **untested**. Never checked on the physical mouse. The app and the CLI write directly, with a warning.

## Where each field comes from
| Field | Value | Source |
| --- | --- | --- |
| Family / sensor | 2 / 3325 (`0xCFD`) | official software `RDCfg.exe`, `FUN_00431a30` and `FUN_004314d0` |
| Memory map | same as the M901-1 | `mouse_m908` capture of this model (addresses only) |
| Buttons | 10 per profile, slots 1 to 8, 11 and 12, as on the M711 Cobra | `mouse_m908` capture |
| DPI | same 75 values as the M711 Cobra (100 to 5000 by 100, 5200 to 10000 by 200) | official table for sensor `0xCFD`; all 75 codes in the `mouse_m908` capture of this model match it |
| LED | same coding as the M901-1 (family 0) | official software: families 0 to 2 share it |
| Macros | 1 slot | the `mouse_m908` author wrote that this model "only appears to support a single macro" |

## Still unconfirmed
- The official macro layout for family 2 (`FUN_00458f90`) was not read. Only slot 1 is offered until it is.
- The `mouse_m908` capture uses a different LED coding for this model than the official software. We follow the official one.
- Button names on the shell.

## How to help
Open the "Mouse support" issue form with the output of `salamandra show` before and after a change, and what you saw on the mouse.
