# MSI MS-7E75 First-Write Implementation Plan

Status: design only. This document approves no code, no HID device opens, and
no hardware writes.

## 1. Purpose

This document describes the narrowest future first-write path that could be
implemented later if a separate explicit risk decision approves Phase 4 work.

Current conclusion:

- this is design-only
- no writes are implemented
- Phase 4 remains HOLD
- `first_write_ready` remains `no` until separate explicit approval

## 2. Fixed Scope For The First Future Write

If a future write-once path is ever approved, the first allowed target must be
exactly:

- board/profile `MS-7E75`
- zone `JARGB_V2_1`
- mode `steady`
- color `ff0000`
- report `0x50`
- payload length `290`

Additional fixed limits:

- do not use `0x90..0x93`
- send at most one `SET_REPORT` packet
- no loop
- no retry
- no background daemon
- no automatic discovery-to-write behavior

## 3. Proposed Future Command Shape

This command is not implemented now. It is design-only.

Possible future command:

```bash
cargo run -- linux hid first-write-once \
  --zone JARGB_V2_1 \
  --mode steady \
  --color ff0000 \
  --i-understand-this-may-affect-hardware \
  --confirm-ms-7e75 \
  --confirm-jargb-v2-1 \
  --confirm-one-packet
```

Required explicit flags:

- `--i-understand-this-may-affect-hardware`
- `--confirm-ms-7e75`
- `--confirm-jargb-v2-1`
- `--confirm-one-packet`

Possible optional second noninteractive confirmation flag:

- `--yes-send-exact-one-packet`

The final flag naming may change in a future reviewed implementation, but the
scary explicit-confirmation model must not be weakened.

## 4. Required Future Validation Flow

Before any future HID device open or send step:

1. Confirm board/profile is exactly `MS-7E75`.
2. Refuse any zone other than `JARGB_V2_1`.
3. Refuse any mode other than `steady`.
4. Refuse any color other than `ff0000`.
5. Refuse any report shape other than `0x50` length `290`.
6. Refuse if the generated payload does not match the checked-in fixture
   byte-for-byte.
7. Refuse if the checked result is not `fixture_match = yes`.
8. Refuse if the command would send more than one packet.
9. Refuse any fallback to `0x90..0x93`.

This flow must not attempt broader discovery-write behavior. Unsupported input
must return a clear error instead of guessing.

## 5. Required Future Output Before Send

Before the final future send step, the command must print:

- board/profile `MS-7E75`
- zone `JARGB_V2_1`
- mode `steady`
- color `ff0000`
- exact setup bytes `21 09 50 03 00 00 22 01`
- full 290-byte payload hex dump
- `fixture_match = yes`
- `devices_opened = no` during the pre-send phase
- `writes_performed = no` before send
- clear statement that `devices_opened` will become `yes` only at the final
  future step

The future implementation must not open a HID device before printing this
information.

## 6. Last-Chance Abort Requirement

The future command must include a last-chance abort prompt unless a second
explicit noninteractive confirmation flag is supplied.

Required behavior:

- default interactive mode prints `last chance to abort`
- interactive mode must stop if the user declines
- noninteractive send mode requires an additional explicit confirmation flag
- absence of the second confirmation flag must prevent send

## 7. Prepare-Only Design

The future implementation should ideally support a prepare-only mode.

Example design intent:

```bash
cargo run -- linux hid first-write-once --prepare-only ...
```

Prepare-only mode requirements:

- open no devices
- send no packets
- print the exact setup bytes
- print the full 290-byte payload
- print `fixture_match = yes` only when equality really passes
- keep `devices_opened = no`
- keep `writes_performed = no`

This mode would exist only to verify the final future send candidate before any
device access.

## 8. Tripwire Update Rules For The Future Reviewed Commit

The HID safety tripwire must remain active until the same future reviewed write
implementation commit intentionally updates it.

Required rule:

- do not weaken the tripwire earlier
- do not remove tripwire checks in a preparatory commit
- do not add broad exemptions
- update the tripwire only in the same reviewed commit that adds the tightly
  scoped first-write path
- keep the tripwire hostile to any broader write path than the approved
  `MS-7E75` / `JARGB_V2_1` / `steady` / `ff0000` / `0x50` / `290` / one-packet
  design

The future tripwire update should narrow the allowed write markers to the
reviewed first-write implementation only, rather than generally enabling Linux
HID writes.

## 9. Explicit Non-Goals

- no implementation in this commit
- no HID device opens
- no feature-report send path
- no support for other zones
- no support for other modes
- no support for other colors
- no support for `0x90..0x93`
- no loop, retry, service, or daemon behavior
- no general Linux lighting support claim

## 10. Decision Gate Reminder

This plan is not approval.

Before any future implementation work begins, the separate checklist and risk
decision still apply:

- [MSI MS-7E75 First-Write Checklist](MSI_7E75_FIRST_WRITE_CHECKLIST.md)
- [MSI MS-7E75 Current-State Handoff](MSI_7E75_HANDOFF.md)
- [MSI MS-7E75 Pre-Write Risk Assessment](MSI_7E75_PRE_WRITE_RISK_ASSESSMENT.md)

Until a separate explicit approval says otherwise:

- Phase 4 remains HOLD
- `first_write_ready` remains `no`
- no write path is implemented
