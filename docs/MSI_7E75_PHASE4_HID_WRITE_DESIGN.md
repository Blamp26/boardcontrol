# MSI MS-7E75 Phase 4 HID Write Design

Status: documentation-only design. This document approves no code and no writes.

## Purpose

The exact purpose of Phase 4 is to design the first reviewed HID `SetFeature`
experiment for the MSI MS-7E75 / B850 GAMING PLUS WIFI PZ MB800 lighting path.

Phase 4 is not Linux HID support, not a general apply path, not an effects
engine, and not a claim that MS-7E75 hardware access is approved. It is only a
proposal for one future, separately reviewed, manually confirmed HID feature
report write after the read-only and dry-run gates have passed.

## Current Evidence

Real-machine validation passed for the read-only and dry-run phases:

- Phase 1 inventory found exactly one MSI MYSTIC LIGHT HID candidate:
  - VID `0x0DB0`
  - PID `0x0076`
  - serial `7E7525011601`
  - serial prefix `0x7E75`
- Phase 2 gate reached `eligible_for_dry_run`.
- Phase 3 dry-run passed for all currently documented zones:
  - `JRGB1` -> report `0x50`, length `290`, area `9`
  - `JARGB_V2_1` -> report `0x90`, length `302`, port `0`
  - `JARGB_V2_2` -> report `0x91`, length `302`, port `1`
  - `JARGB_V2_3` -> report `0x92`, length `302`, port `2`
  - `EZ Conn` -> report `0x93`, length `302`, port `3`
- No device opens were reported.
- No writes were performed.
- Linux HID support remains unsupported and not enabled.

This evidence is enough to design a reviewed first-write experiment. It is not
enough to implement or run that experiment without a later approval step.

## Prerequisites Before Implementation

Before any Phase 4 code is implemented, all of the following must be true:

- A reviewer has accepted this design or a stricter replacement.
- The implementation task explicitly permits HID write code.
- The implementation task explicitly permits opening the target HID device.
- The implementation task explicitly permits the future `SetFeature` call.
- Phase 1, Phase 2, and Phase 3 outputs from the target host are captured and
  attached to the implementation review.
- The Phase 3 dry-run command succeeds immediately before the proposed write
  command on the same booted host.
- The exact report ID, report length, zone, and color are named in the review.
- The rollback and disable plan below has a concrete implementation.

Until those prerequisites are met, the codebase must remain documentation-only
for Phase 4 and must not enable MS-7E75 hardware access.

## Required Safety Gates

A future Phase 4 command must fail closed unless every gate below passes:

- DMI must match `MS-7E75` / `B850 GAMING PLUS WIFI PZ`.
- HID inventory must find exactly the expected VID/PID candidate:
  - VID `0x0DB0`
  - PID `0x0076`
- HID inventory must not find multiple plausible MS-7E75 candidates.
- The HID serial prefix must parse and match expected board ID `0x7E75`.
- Phase 3 dry-run must succeed immediately before the write.
- The user must pass an explicit confirmation flag such as
  `--confirm-hid-write`.
- The command must refuse if any gate is blocked, inconclusive, missing, or
  stale.
- The command must not perform broad or automatic writes.
- The command must not run an effects loop.
- The command must not perform an all-zone write as the first test.

The command must treat ambiguity as a hard refusal, not as a prompt to guess.

## Required CLI Confirmation Flag

The future write command must require an explicit flag named
`--confirm-hid-write` or a stricter equivalent. The flag must be specific to HID
writes and must not be shared with read-only confirmation flags such as
`--confirm-read`.

Without the flag, the command must print the planned report metadata and refuse
before opening a device.

## Proposed Command Shape

Proposed first command:

```bash
cargo run -- linux hid write-once \
  --zone JARGB_V2_1 \
  --color ff0000 \
  --confirm-hid-write
```

Required behavior:

- Run or require a fresh Phase 3 dry-run result for the same zone and color.
- Print the exact report family, report ID, report length, port or area, and
  color before the write.
- Open only the single gated target HID device.
- Send exactly one HID feature report.
- Exit immediately after the one attempted report.
- Print whether the one attempted `SetFeature` call returned success or
  failure.

The command name `write-once` is intentional: it should communicate that this is
not a general apply/effects command.

## Proposed Module Layout

The existing read-only and dry-run split should remain intact:

- `src/linux/hid/report.rs`: pure in-memory report builders only.
- `src/linux/hid/inventory.rs`: read-only candidate discovery only.
- `src/linux/hid/gate.rs`: DMI, HID identity, serial-prefix, and freshness
  gates.
- `src/linux/hid/dry_run.rs`: report preview and layout checks only.

Future Phase 4 code, if later approved, should be isolated:

- `src/linux/hid/write_once.rs`: one-report write orchestration.
- `src/linux/hid/device.rs`: minimal HID device open and `SetFeature` wrapper.
- `src/linux/hid/confirm.rs`: explicit write confirmation validation if this
  grows beyond CLI parsing.

The write wrapper must not be reachable from inventory, gate, or dry-run paths.
Tests must prove those paths stay read-only.

## Allowed First Write Scope

The allowed first write candidate is:

- One simple static RGB report only.
- Prefer one Gen2 port.
- Preferred first zone: `JARGB_V2_1`.
- Preferred first report: `0x90`.
- Expected report length: `302`.
- Expected port: `0`.
- One color only, such as `ff0000`.
- One report dispatch only.
- No persistence assumptions.
- No repeated loop.
- No attempt to restore, animate, synchronize, or fan out to other zones.

The first experiment should be treated as an observation of one report result,
not as an attempt to set a stable user-facing lighting state.

## Forbidden Write Scope

The first Phase 4 experiment must not include:

- no all-zone first write
- no `SELECT ALL` behavior
- no effects loops
- no brightness/speed experiments
- no repeated writes
- no automatic retries that broaden scope
- no fallback to SMBus, EC, Super I/O, WMI, ACPI, `/dev/port`, or another
  transport
- no broad HID autodetect write
- no writes if multiple candidates exist
- no writes if serial is missing or inconclusive
- no writes to `JRGB1` as first test unless separately reviewed
- no write to more than one zone
- no mixed Gen1 plus Gen2 write
- no persistence command or assumption that the state survives reboot

## Rollback / Disable Plan

A future implementation must include all of the following controls:

- compile-time feature or explicit command isolation
- no default-enabled write command
- clear removal path for `write_once` and device modules
- CLI must stay absent or refusing unless explicitly enabled
- if first write behaves unexpectedly, stop and remove or disable the Phase 4 path
- keep Phase 4 behind a dedicated code path that can be removed without
  changing report builders, inventory, gate, or dry-run behavior
- keep the command opt-in and hidden from any default flow
- keep Linux HID support marked unsupported/not enabled until a separate support
  decision is made
- add a build-time or runtime kill switch if the implementation introduces a
  feature flag
- make failed gates return before any device open
- make missing confirmation return before any device open
- make write failures stop immediately without retry fan-out
- preserve the previous read-only and dry-run commands as safe diagnostics

Rollback for a bad review or bad result is to delete or disable only the future
write module and command, leaving the validated read-only path intact.

## Test Plan

Required non-hardware tests:

- unit tests for confirmation flag refusal
- gate blocked or inconclusive refusal
- multiple candidate refusal
- stale dry-run refusal
- exact report ID and length assertions
- ensure inventory, gate, and dry-run paths do not link to writer
- mock writer only; no real HID device in tests
- unit tests for the write gate requiring DMI match
- unit tests for the write gate requiring exactly one VID/PID candidate
- unit tests for the serial prefix match to `0x7E75`
- unit tests rejecting missing, malformed, or non-`0x7E75` serial prefixes
- unit tests rejecting blocked or inconclusive gate states
- unit tests rejecting stale or mismatched dry-run evidence
- unit tests requiring `--confirm-hid-write`
- unit tests proving inventory, gate, and dry-run paths do not call write code
- unit tests proving the first write scope is one zone, one color, one report

Required review-time checks:

- `cargo fmt --all -- --check`
- `cargo check`
- `cargo test`
- `cargo clippy -- -D warnings`

Required real-machine pre-write checks, if a later task permits writes:

- Re-run Phase 1 inventory on the target host.
- Re-run Phase 2 gate on the target host.
- Re-run Phase 3 dry-run for the exact zone and color.
- Confirm the output still reports no device opens and no writes before Phase 4.
- Save the full terminal log before attempting the write.

## Logging / Output Requirements

A future Phase 4 command must print:

- `phase = 4`
- print DMI match
- print VID/PID/serial gate
- print zone/report ID/report length/port/color
- print one-attempt-only wording
- print `SetFeature` success/failure if implemented later
- print `confirm_hid_write = true` only when the flag is present
- `writes_planned = 1`
- `writes_attempted = 0` before the write attempt
- `writes_attempted = 1` only after the one attempted `SetFeature`
- final result: success, failed, or refused
- never print `supported` or `enabled` as general Linux support

The output must not hide refusal reasons. If the command refuses, it must state
which gate failed and must leave `writes_attempted = 0`.

## Review Checklist Before Implementation

Before implementation starts, reviewers should confirm:

- design accepted
- real-machine validation attached
- exact command approved
- exact zone/color/report approved
- write code diff reviewed
- tests pass
- no unrelated hardware paths changed
- the task explicitly requests code changes, not documentation only
- the task explicitly permits HID device opening
- the task explicitly permits one `SetFeature` call
- the DMI gate remains exact for `MS-7E75` / `B850 GAMING PLUS WIFI PZ`
- the HID VID/PID gate remains exact for `0x0DB0` / `0x0076`
- the serial prefix gate remains exact for `0x7E75`
- the command refuses multiple candidates
- the command refuses inconclusive metadata
- the command requires `--confirm-hid-write`
- the first write is limited to `JARGB_V2_1`, report `0x90`, length `302`,
  port `0`, one color, one report
- no effects loop, all-zone write, broad autodetect write, fallback transport,
  or persistence behavior has been added
- read-only and dry-run commands remain unable to open HID devices or write

## Stop Conditions

Stop before implementation if:

- unexpected candidate count
- serial mismatch or missing serial
- gate not eligible
- dry-run mismatch
- write attempt would use different report ID or length
- any request to loop, apply all, or run effects
- any HID error or unexpected behavior after the first write
- the requested work is documentation-only
- the request does not explicitly permit HID writes
- the request does not explicitly permit opening the HID device
- a reviewer has not approved this design or a stricter replacement

Stop before a write attempt if:

- DMI does not exactly match `MS-7E75` / `B850 GAMING PLUS WIFI PZ`
- HID inventory does not find exactly one expected VID/PID candidate
- the serial prefix is missing, malformed, or not `0x7E75`
- Phase 3 dry-run did not succeed immediately before the write
- the user did not pass `--confirm-hid-write`
- the selected zone is not the reviewed first-write zone
- the report ID, length, port, or color differs from the reviewed values
- any command attempts to broaden scope, retry into another zone, or enter a
  loop

Stop after the first attempted report regardless of success or failure.

## Explicit Boundary

This document approves no code and no writes. It does not implement HID writes,
does not add hidapi writes, does not open HID devices, does not call
`SetFeature` or `GetFeature`, does not touch `/dev/hidraw*`, `/dev/port`, SMBus,
or Super I/O, and does not enable MS-7E75 hardware access.

Phase 4 remains unimplemented and unapproved until a later task explicitly
authorizes implementation and review of the first HID `SetFeature` experiment.
