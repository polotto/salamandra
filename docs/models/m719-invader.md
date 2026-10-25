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
- The official macro writer (`FUN_00459cf0`) has no family or model branch, so the layout is the M901-1 one. The 1 slot limit comes only from the `mouse_m908` author, so only slot 1 is offered until someone tests more.
- The official software maps its button indexes 3, 4 and 5 to memory slots 5, 3 and 4 for this model (`FUN_0045c570`). We number by slot, so writing is unaffected, but the names of those three buttons on the shell may be swapped.
- The `mouse_m908` capture uses a different LED coding for this model than the official software. We follow the official mode, with the `sub` byte the M901-1 was tested with (the multi-model official program (`RDCfg.exe`) writes `1` there for families 0 to 2; see the reverse engineering notes).
- Button names on the shell.

## How to help
Open the "Mouse support" issue form with the output of `salamandra show` before and after a change, and what you saw on the mouse.
