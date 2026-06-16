# MSI MS-7E75 Linux HID Validation Checklist

Status: documentation only. This checklist is for safe read-only and dry-run validation on a real machine before any future Phase 4 write path is even discussed.

## Real-machine validation result

Validation on the real MSI MS-7E75 / B850 GAMING PLUS WIFI PZ board passed for the already-implemented read-only and dry-run phases.

- Phase 1 inventory passed.
- Phase 2 gate reached `eligible_for_dry_run`.
- Phase 3 dry-run passed for `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, and `EZ Conn`.
- No device opens were reported.
- No writes were performed.
- Phase 4 is still not implemented and not approved.
- The next step is a separate reviewed Phase 4 write design, not immediate write code.

Observed safe command outputs:

- `cargo run -- linux hid inventory`
  - `candidates = 1`
  - `syspath = /sys/bus/hid/devices/0003:0DB0:0076.000B`
  - `vid = 0x0DB0`
  - `pid = 0x0076`
  - `mi = unknown`
  - `col = unknown`
  - `product = MSI MYSTIC LIGHT`
  - `serial = 7E7525011601`
  - `serial_prefix = expected board id 0x7E75`
  - `plausible_ms7e75 = true`
  - `devices_opened = no`
  - `writes_enabled = no`
  - `support = unsupported/not enabled`

- `cargo run -- linux hid gate`
  - `dmi_match = true`
  - `dmi matched MSI MS-7E75 board identity: vendor=Micro-Star International Co., Ltd. board=B850 GAMING PLUS WIFI PZ (MS-7E75) product=MS-7E75`
  - `hid_inventory_match = true`
  - `hid_candidates = 1`
  - `serial_gate = matched expected board id 0x7E75`
  - `final_status = eligible_for_dry_run`
  - `writes_enabled = no`
  - `support = unsupported/not enabled`

- `cargo run -- linux hid dry-run --zone JRGB1 --color ff0000`
  - `report_family = Gen1`
  - `report_id = 0x50`
  - `report_length = 290`
  - `area_index = 9`
  - `status = DRY RUN ONLY`
  - `devices_opened = no`
  - `writes_performed = no`

- `cargo run -- linux hid dry-run --zone JARGB_V2_1 --color ff0000`
  - `report_family = Gen2`
  - `report_id = 0x90`
  - `report_length = 302`
  - `port_index = 0`
  - `status = DRY RUN ONLY`
  - `devices_opened = no`
  - `writes_performed = no`

- `cargo run -- linux hid dry-run --zone JARGB_V2_2 --color ff0000`
  - `report_family = Gen2`
  - `report_id = 0x91`
  - `report_length = 302`
  - `port_index = 1`
  - `status = DRY RUN ONLY`
  - `devices_opened = no`
  - `writes_performed = no`

- `cargo run -- linux hid dry-run --zone JARGB_V2_3 --color ff0000`
  - `report_family = Gen2`
  - `report_id = 0x92`
  - `report_length = 302`
  - `port_index = 2`
  - `status = DRY RUN ONLY`
  - `devices_opened = no`
  - `writes_performed = no`

- `cargo run -- linux hid dry-run --zone "EZ Conn" --color ff0000`
  - `report_family = Gen2`
  - `report_id = 0x93`
  - `report_length = 302`
  - `port_index = 3`
  - `status = DRY RUN ONLY`
  - `devices_opened = no`
  - `writes_performed = no`

## Scope

Use this checklist only for the already-implemented read-only phases:

- Phase 1 HID inventory
- Phase 2 board gate
- Phase 3 dry-run report preview

Do not use this checklist to justify writes. No write/apply command is approved here.

## Safe Commands

Use only these commands:

```bash
cargo run -- linux hid inventory
cargo run -- linux hid gate
cargo run -- linux hid dry-run --zone JRGB1 --color ff0000
cargo run -- linux hid dry-run --zone JARGB_V2_1 --color ff0000
cargo run -- linux hid dry-run --zone JARGB_V2_2 --color ff0000
cargo run -- linux hid dry-run --zone JARGB_V2_3 --color ff0000
cargo run -- linux hid dry-run --zone EZ Conn --color ff0000
```

## Expected Safe Behavior

- `inventory` should only scan metadata and print candidate summary text.
- `gate` should only combine DMI plus inventory metadata and print a final gate status.
- `dry-run` should only build reports in memory and print report metadata plus a hex preview.
- None of the commands should open HID devices.
- None of the commands should call `SetFeature` or `GetFeature`.
- None of the commands should touch `/dev/hidraw*`, `/dev/port`, SMBus, or Super I/O.
- None of the commands should enable MS-7E75 hardware access.

## What To Capture

Capture the full terminal output for each command, including:

- the command that was run
- the reported board identity or inventory metadata
- the final status line
- the hex preview from dry-run output
- any refusal reason if the gate is not ready

Save the output exactly as shown on the target machine. If possible, capture the host DMI summary too, because it helps explain why a gate was eligible, blocked, or inconclusive.

## How To Interpret Results

### Inventory

- `supported/not enabled` or similar wording is expected.
- A candidate with `VID 0x0DB0 / PID 0x0076` is the only inventory shape that matters for the MS-7E75 MB800 path.
- If serial is missing, inventory should say that the serial prefix is unknown or unreadable.

### Gate

- `eligible_for_dry_run` means the Phase 2 board gate matched and dry-run may proceed.
- `blocked` means a required condition failed.
- `inconclusive` means the gate could not prove the serial prefix, so dry-run must not proceed by default.

### Dry Run

- `DRY RUN ONLY` means the report was built in memory and no write occurred.
- `report_family`, `report_id`, and `report_length` should match the documented zone.
- `JRGB1` should report Gen1 with area index `9`.
- `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, and `EZ Conn` should report Gen2 with ports `0`, `1`, `2`, and `3`.

## Stop Conditions

Stop immediately if any of the following happen:

- a command tries to open a HID device
- a command offers to write, apply, or set a report
- output mentions `SetFeature`, `GetFeature`, or a device handle
- output differs from the expected read-only or dry-run wording
- the gate is `blocked` or `inconclusive` and the command tries to continue anyway
- any command asks for `/dev/hidraw*`, `/dev/port`, SMBus, or Super I/O access

If any stop condition appears, do not continue to a later phase.

## Phase 4 Readiness Bar

Phase 4 can only be considered after all of the following are true:

- the checklist has been run on a real MS-7E75 machine
- `inventory` consistently shows the expected MSI common HID identity
- `gate` reaches `eligible_for_dry_run` on the real machine
- `dry-run` succeeds for `JRGB1` and all four Gen2 zones
- the captured outputs show no device opens and no writes
- the results are stable across repeated runs on the same host

Even then, Phase 4 still requires a separate reviewed write plan. This checklist does not approve any write/apply command.

## Explicit Boundary

Do not run any write/apply command because none is approved yet. This checklist is read-only and dry-run only.
