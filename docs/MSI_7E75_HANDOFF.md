# MSI MS-7E75 Current-State Handoff

Status: documentation only. This handoff approves no code and no writes.

## 1. Core Goal

Build a safe Linux replacement path for MSI Center / Mystic Light on MS-7E75,
without risking hardware.

## 2. Current Status

Implemented:

- Phase 0 pure in-memory report builder
- Phase 1 read-only HID inventory
- Phase 2 DMI / HID / serial gate
- Phase 3 dry-run report preview
- external evidence notes and OpenRGB protocol comparison
- passive MSI Center USBPcap capture notes for `MB -> JARGB_V2_1`
- explicit Phase 4 hold notes in the design, risk, and implementation-plan docs

Validated:

- MS-7E75 board identity matched on the target system
- HID candidate matched `VID 0x0DB0` and `PID 0x0076`
- serial prefix gate matched `0x7E75`
- dry-run buffers matched the documented layouts
- no device opens were reported in the validated phases
- no writes were performed
- passive MSI Center traffic for `MB -> JARGB_V2_1` used `0x50`/290, not
  `0x90`/302

Explicitly held:

- Phase 4 HID write work
- any `write-once` path
- any general Linux lighting support beyond inventory, gate, and dry-run

## 3. Known Board Identity

- DMI: MSI `B850 GAMING PLUS WIFI PZ`
- board family: `MS-7E75`
- HID identity: `0x0DB0:0x0076`
- serial gate: first four serial characters parse as board ID `0x7E75`

## 4. Static RE Conclusions

- LEDKeeper2, MBAPI, MB800, and `MsiHid` all point to the same Mystic Light
  control chain for this board family.
- static profile data shows MS-7E75 zone records and board-specific routing
  rather than a generic lighting model.
- the zone mapping documented in the static notes includes `JRGB1`,
  `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, and `EZ Conn`.
- `0x90..0x93` remain static/decompiled evidence only until live traffic
  confirms them.
- The only live MSI Center write path observed so far is `0x50`/290.

## 5. Linux Implementation State

- Phase 0: implemented, in-memory report builder only
- Phase 1: implemented, read-only inventory only
- Phase 2: implemented, DMI / HID / serial gate only
- Phase 3: implemented, dry-run preview only
- Phase 4: on hold

## 6. Safety Boundaries

- Do not implement HID writes.
- Do not open HID devices.
- Do not use SMBus, Super I/O, or `/dev/port` as a fallback for this path.
- Do not add `write-once` unless a later review separately accepts that risk.
- Do not use `JARGB_V2_1 -> 0x90` as a first-write plan.
- Keep inventory, gate, and dry-run paths read-only.

## 7. Key Commands Allowed

```bash
cargo run -- linux hid inventory
cargo run -- linux hid gate
cargo run -- linux hid dry-run --zone JRGB1 --color ff0000
cargo run -- linux hid dry-run --zone JARGB_V2_1 --color ff0000
```

These commands stay read-only. They do not open HID devices or transmit
feature reports.

## 8. Evidence Gaps

- no live confirmation for MB800 `0x90..0x93` reports
- no approved write or recovery path
- OpenRGB provides only a near match, not an exact MS-7E75 MB800 confirmation
- current passive MSI Center capture observed `0x50`/290 for `MB -> JARGB_V2_1`

## 9. Best Next Steps

- decode the passive `0x50`/290 USBPcap payloads offline
- collect more passive MSI Center traffic for the other zones and restore paths
- keep adding safer diagnostics and documentation
- do not add write code yet

## 10. Latest Commit State

- latest local commit should document the MS-7E75 USBPcap capture evidence
