# Safety Model

`boardcontrol` uses a safety-first model for all hardware-adjacent features.
The project prioritizes traceability, explicit gating, and read-only validation before any future write-capable path can exist.

## Core Rules

- no blind writes
- no arbitrary register access
- no unknown board writes
- no unknown chip writes
- no writes before read-only detection
- no writes without explicit confirmation flag
- no writes outside board profile allowlist
- trace-first, read-only second, write-last
- hardware writes are not implemented yet

## Current Protection Layers

1. `TraceBackend`
   - init/reset sequences can be simulated without hardware access

2. Board profile gate
   - currently only `7A45`

3. DMI preflight
   - Linux host must look like MSI `7A45`

4. `/proc/ioports` check
   - blocks access if `004e-004f` appears busy

5. Chip ID check
   - NCT id high byte must be `0xC5`

6. Allowlist
   - only known `(LDN, REG)` pairs are readable
   - future writes must use the same allowlist model

7. Safe RMW

```text
new_value = (current & and_mask) | or_mask
changed = current ^ new_value

if changed & !allowed_change_mask != 0:
    block
else:
    write
```

## Current Non-Target Test

`Dell Inc. / OptiPlex 5000` was rejected by DMI preflight before `/dev/port` access.

## Before Any Future Hardware Write

- `doctor` must pass
- board must be recognized as MSI `7A45`
- chip must be detected as NCT6779D
- target register must be in allowlist
- operation must be RMW, not blind write
- dry-run trace must be reviewed first
- write command must require explicit confirmation flag

## Not Implemented Yet

- NCT hardware writes
- LED init apply
- LED reset apply
- Renesas SMBus writes
- RGB/effect control
- Windows backend
