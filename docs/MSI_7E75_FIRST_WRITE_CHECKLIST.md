# MSI MS-7E75 First-Write Checklist

Status: read-only decision gate only. This document approves no write code and
no hardware writes.

## 1. Purpose

This checklist is the formal pre-write gate for any future MS-7E75 Linux HID
write discussion.

Current conclusion:

- exact offline/dry-run output is approved
- hardware writes are still not approved
- Phase 4 remains on hold until a separate explicit user risk decision

## 2. Already Satisfied Evidence

- USBPcap live MSI Center evidence confirms `SET_REPORT` feature report `0x50`
  with HID payload length `290` for `JARGB_V2_1`.
- Full TEST 2 through TEST 6 setup+payload USBPcap fixtures are checked in for:
  steady red, steady green, steady blue, breath red, and off with retained red.
- The isolated offline `JARGB_V2_1` live payload builder matches the checked-in
  290-byte MSI Center payloads byte-for-byte.
- `exact-live-dry-run` prints the exact setup bytes and full 290-byte payload
  and reports `fixture_match = yes`.
- Unsupported zones, modes, and colors reject clearly with no fallback and no
  guessing.
- The HID safety tripwire is still present.

## 3. Still Required Before First Write

- The user must close MSI Center, OpenRGB, and SignalRGB before any future write
  test.
- The user must confirm that the exact board/profile is `MS-7E75`.
- The first write may only target `JARGB_V2_1`.
- The first write may only use steady `ff0000`.
- The first write may send one command and one packet only, with no loop and no
  retry.
- Any future write path must require a scary explicit flag.
- Any future write path must refuse unsupported zones, modes, and colors.
- Any future write path must print the exact payload before sending.
- Any future write path must print `last chance to abort` immediately before any
  send step.
- Any future write path must not support `0x90..0x93`.

## 4. Explicit User Decision Required

- A separate explicit user risk decision is required before Phase 4 can move
  off HOLD.
- Byte-for-byte offline equality and exact dry-run output do not by themselves
  approve Linux HID writes.
- No HID write implementation, `write-once` path, `SetFeature`/`GetFeature`
  path, or HID device-open path is approved by this checklist.

## 5. Read-Only Checklist Command

The CLI can print this gate in a read-only form:

```bash
cargo run -- linux hid first-write-checklist
```

Expected status lines:

- `status = READ ONLY`
- `devices_opened = no`
- `writes_enabled = no`
- `writes_performed = no`
- `phase4_status = HOLD`
- `first_write_ready = no`

This command performs no hardware access.
