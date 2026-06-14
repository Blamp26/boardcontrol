# Controlled Write Design

This document describes the planned design for future controlled NCT6779D hardware writes.
The current project does not implement LED write/apply commands yet.

## Write Pipeline

doctor preflight
→ board profile check
→ DMI MSI 7A45 check
→ /proc/ioports conflict check
→ /dev/port open
→ NCT6779D chip ID check
→ read current register value
→ calculate RMW new_value
→ verify changed bits against allowlist
→ trace planned operation
→ require explicit confirmation
→ perform write
→ read back register
→ report before/after

## Required Gates

- board must be `7A45`
- DMI must look like MSI `7A45`
- Super I/O ports `004e-004f` must be available
- chip id high byte must be `0xC5`
- target register must exist in allowlist
- operation must be RMW
- changed bits must be inside `allowed_change_mask`
- command must require explicit write confirmation
- dry-run output must be available before write

## Future CLI Shape

These commands are proposed for a later stage and are not implemented yet.

```bash
boardcontrol nct init-7a45 --dry-run
boardcontrol nct init-7a45 --apply --confirm-write 7A45-NCT-RMW
boardcontrol nct reset-led --dry-run
boardcontrol nct reset-led --apply --confirm-write 7A45-NCT-RMW
```

## Write Report Format

```text
RMW LDN=0x09 REG=0xE0
  current: 0x00
  and:     0x7F
  or:      0x80
  new:     0x80
  changed: 0x80
  allowed: 0x80
  status:  allowed
```

After write:

```text
write complete
readback: 0x80
```

## Rollback Policy

- automatic rollback is not guaranteed
- only readback verification is planned
- reset-led may be used later as a recovery helper only after it is implemented safely
- power cycle may be required if external LED controller state hangs

## Non-Goals

- no arbitrary register writer
- no raw port write CLI
- no write support for unknown boards
- no write support for unknown chips
- no Renesas write design in this document
- no Windows write backend in this document
