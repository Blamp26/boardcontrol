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
- offline `0x50`/290 fixture comparison for the newer USBPcap4 mode captures
- offline-only local generator/check for live `JARGB_V2_1` `0x50`/290 payload
  payloads
- full TEST 2 through TEST 6 USBPcap fixtures with byte-for-byte offline
  equality checks
- exact offline/dry-run CLI output for the checked-in live `JARGB_V2_1`
  `0x50`/290 payloads
- formal first-write checklist and read-only decision-gate command
- design-only first-write implementation plan for a future one-packet path
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
- newer USBPcap4 captures confirmed `0x50`/290 steady, breath, and off mode
  bytes for `JARGB_V2_1`
- analyzed passive captures did not contain `0x90..0x93`/302, `0x51`/727, or
  `0xB0`/761 traffic

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
- The live MSI Center UI path observed for JARGB_V2_1 is 0x50/290.
- 0x90..0x93 are not live-confirmed.
- Live `0x50` mode observations now include steady `0x02`, breath `0x04`, and
  off `0x00`, with RGB prefixes `ff0000`, `00ff00`, and `0000ff`.
- A separate offline-only local generator/check now exists for those
  live-confirmed `JARGB_V2_1` `0x50`/290 payloads.
- Full local byte-for-byte equality now passes for the checked-in TEST 2
  through TEST 6 payload dumps.
- An exact offline/dry-run CLI path now prints those checked-in setup bytes and
  full 290-byte payloads for the supported live-confirmed `JARGB_V2_1` cases.
- A formal read-only first-write checklist now exists in
  `docs/MSI_7E75_FIRST_WRITE_CHECKLIST.md` and via
  `cargo run -- linux hid first-write-checklist`.
- A separate design-only first-write implementation plan now exists in
  `docs/MSI_7E75_FIRST_WRITE_IMPLEMENTATION_PLAN.md`.
- This evidence does not approve Linux HID writes.

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
- Do not use 0x90 as the first-write target.
- Keep inventory, gate, and dry-run paths read-only.

## 7. Key Commands Allowed

```bash
cargo run -- linux hid inventory
cargo run -- linux hid gate
cargo run -- linux hid dry-run --zone JRGB1 --color ff0000
cargo run -- linux hid dry-run --zone JARGB_V2_1 --color ff0000
cargo run -- linux hid exact-live-dry-run --zone JARGB_V2_1 --mode steady --color ff0000
cargo run -- linux hid first-write-checklist
```

These commands stay read-only. They do not open HID devices or transmit
feature reports.

## 8. Evidence Gaps

- no live confirmation for MB800 `0x90..0x93` reports
- no approved write or recovery path
- OpenRGB provides only a near match, not an exact MS-7E75 MB800 confirmation
- current passive MSI Center capture observed `0x50`/290 for `MB -> JARGB_V2_1`
- byte `[289]` has been observed as both `0x00` and `0x01`, so it remains only
  observed metadata
- no live `0x51`/727 or `0xB0`/761 traffic was observed in the analyzed
  captures

## 9. Best Next Steps

- continue decoding the passive `0x50`/290 USBPcap payloads offline
- collect more passive MSI Center traffic for the other zones and restore paths
- keep adding safer diagnostics and documentation
- keep the formal first-write checklist as the required gate before any future
  Phase 4 decision
- keep the new first-write implementation plan design-only until separate
  explicit approval exists
- do not add write code yet

## 10. Latest Commit State

- latest local commit should keep the first-write path in documentation only
  status
