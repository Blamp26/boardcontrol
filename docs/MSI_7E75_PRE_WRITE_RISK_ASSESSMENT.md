# MSI MS-7E75 Pre-Write Risk Assessment

Status: documentation only. This document approves no code and no writes.

## Current decision: Phase 4 hold

Phase 4 is on hold. Do not implement HID writes yet. Do not run write-once. Do
not add general Linux lighting support yet. Keep only inventory, gate, and
dry-run paths.

Required unblocker: stronger board-family-exact live evidence for the actual MSI
Center report family, length, initialization flow, and recovery behavior.

## Purpose

This document records the current risk posture for the MSI MS-7E75 / B850
GAMING PLUS WIFI PZ board after the read-only HID phases completed successfully.
It exists to keep Phase 4 deferred until the team has enough external evidence to
justify any later write review.

This is not a recommendation to run `write-once`, `SetFeature`, `GetFeature`, or
any other write-capable path.

## Current Confidence Level

Current confidence is moderate for the read-only identification and dry-run
path, and low for the first HID write path.

Why:

- the candidate VID/PID and serial prefix matched on a real board
- the board gate reached `eligible_for_dry_run`
- the dry-run buffers matched the documented report layouts
- no device opens were reported in the validated phases
- no write behavior has been observed yet
- no recovery behavior has been proven yet
- external evidence confirms the common `0DB0:0076` Mystic Light HID identity,
  but does not confirm the MS-7E75 / MB800 `0x90..0x93` write reports
- passive MSI Center USBPcap evidence for `MB -> JARGB_V2_1` observed
  `0x50`/290, not `0x90`/302

## What Is Known

- The real board enumerated one MSI MYSTIC LIGHT HID candidate.
- The candidate matched VID `0x0DB0` and PID `0x0076`.
- The serial prefix matched expected board ID `0x7E75`.
- The Phase 2 gate reached `eligible_for_dry_run`.
- The Phase 3 dry-run generated the expected reports for:
  - `JRGB1`
  - `JARGB_V2_1`
  - `JARGB_V2_2`
  - `JARGB_V2_3`
  - `EZ Conn`
- The current Linux HID work remains read-only and no writes were performed.
- The target machine still has no validated write path.
- The only live MSI Center write path observed so far is `0x50`/290.
- `0x90..0x93` remain static/decompiled evidence only until live traffic
  confirms them.

## What Is Still Unknown

- Whether the MB800 HID path on this board behaves like other MSI boards that
  are already documented elsewhere.
- Whether the first HID feature report will be accepted unchanged by the
  controller.
- Whether the controller needs state initialization before any write.
- Whether the controller reacts safely to one isolated write.
- Whether the controller can be restored by firmware, BIOS, MSI Center, or a
  power-cycle if a bad state is reached.
- Whether a write would affect only lighting, or would also change controller
  state more broadly.
- Whether Linux behavior matches Windows behavior closely enough to justify a
  first write review.

## Concrete Risks Of First HID SetFeature

- The controller may accept a report layout but interpret the payload
  differently than expected.
- A write may affect more than the target header or port.
- A write may alter controller state in a way that is not immediately visible.
- A write may leave the board in a state that is annoying to recover from even
  if the board remains functional.
- An unexpected response could tempt broader retries, which would increase risk.
- A report that works in dry-run may still fail when actually transmitted.
- A first write to the wrong zone or wrong report could produce side effects not
  covered by the current read-only evidence.
- The lack of a validated recovery flow means the cost of a bad first write is
  still unknown.

## Why Dry-Run Passing Is Not Enough

Dry-run only proves that the in-memory report builder matches the documented
layout. It does not prove:

- that the board accepts the packet
- that the controller applies the packet safely
- that the controller ignores unrelated bits
- that the selected zone is the only affected zone
- that the controller can be restored if the state becomes bad
- that the Windows path and Linux path are behaviorally identical

Dry-run is necessary, but it is not sufficient to justify a first write.

## External Evidence To Collect Before Any Write

Initial external evidence collection is recorded in
[MSI_7E75_EXTERNAL_HID_EVIDENCE.md](MSI_7E75_EXTERNAL_HID_EVIDENCE.md).
The focused OpenRGB protocol comparison is recorded in
[MSI_7E75_OPENRGB_PROTOCOL_COMPARISON.md](MSI_7E75_OPENRGB_PROTOCOL_COMPARISON.md).
Passive MSI Center USBPcap evidence is recorded in
[MSI_7E75_USBPCAP_CAPTURE_NOTES.md](MSI_7E75_USBPCAP_CAPTURE_NOTES.md).
These notes confirm the common VID/PID family and partial `0x50` / `290`
evidence, but do not lower first-write risk enough to change the read-only
recommendation.

Collect the following external evidence before any later Phase 4 review:

- confirm whether OpenRGB already supports the same MSI HID controller VID/PID
  or NUC126/MB800 path
- search existing Linux, OpenRGB, and MSI Mystic Light code for `0DB0:0076`
- compare report IDs `0x90` through `0x93` and `0x50` with known safe
  implementations
- capture Windows behavior passively if possible without writing custom packets
- do not use `JARGB_V2_1 -> 0x90` as a first-write plan
- confirm whether MSI Center can restore default lighting after changes
- identify whether BIOS has LED restore or default options
- confirm the board has a recovery path if the lighting controller state becomes
  bad

## Safe Observation-Only Checks

Safe observation-only checks may include:

- reviewing static code and strings in known tools
- reading existing documentation for the same controller family
- collecting device metadata without opening or writing the device
- comparing report lengths and IDs against published or previously documented
  implementations
- verifying what recovery options are visible in firmware setup screens

These checks must not open HID devices or transmit feature reports.

## Windows-Side Observation Ideas

Observation ideas that do not require reverse-driving hardware:

- inspect installed MSI Center or Mystic Light files for controller names and
  board selectors
- look for logs, config files, or strings that mention `0DB0:0076`,
  `NUC126`, or `MB800`
- observe whether MSI Center exposes a default lighting reset or restore
  control
- capture windowed UI behavior and visible state changes without sending custom
  packets
- review BIOS or UEFI setup screens for lighting defaults or restore entries
- use process monitoring to see which components load when lighting is opened,
  without altering device state

## Linux-Side Observation-Only Checks

Safe Linux-side observation-only checks may include:

- read-only HID inventory metadata
- sysfs or udev metadata that reports VID, PID, serial, and interface details
- comparing the observed HID identity with other documented MSI HID
  implementations
- reading existing documentation or code that handles the same controller family
- confirming that the current commands still refuse to open devices or write

These checks must stay read-only and must not touch `/dev/hidraw*`, `/dev/port`,
SMBus, or Super I/O.

## Decision Criteria

Stay read-only if any of the following remain true:

- the recovery path is unclear
- the controller family is not corroborated by external evidence
- OpenRGB or another known implementation does not match the same controller
  family
- the Windows-side evidence is too thin to explain the write behavior
- the BIOS or MSI Center recovery path cannot be confirmed
- the report IDs or lengths disagree with known safe implementations
- the board-specific consequences of a bad write are still uncertain

Proceed only to a later write review if all of the following are true:

- the board recovery path is documented
- the controller family is corroborated by external evidence
- the observed report IDs and lengths align with known safe implementations
- the write target is still narrowly scoped
- the review can explain exactly why the first report is the right one to test
- the team is comfortable with the remaining uncertainty after read-only
  evidence

## Explicit Recommendation

Do not implement or run Phase 4 yet.

The current evidence supports continuing observation, evidence collection, and
read-only validation only. It does not justify opening the device or sending a
first HID `SetFeature` report.

The OpenRGB protocol comparison and the MSI Center USBPcap capture keep this
recommendation unchanged. The closest OpenRGB JARGB/JAF implementation does not
confirm the MS-7E75 MB800 `0x90..0x93` / `302` report path, and the live MSI
Center `MB -> JARGB_V2_1` path observed so far is `0x50`/290.

Current decision: Phase 4 hold. Keep only inventory, gate, and dry-run paths.
Do not add general Linux lighting support yet. Do not use `JARGB_V2_1 -> 0x90`
as a first-write plan. The unblocker is stronger board-family-exact live
evidence for the actual MSI Center report family, length, initialization flow,
zone mapping, and recovery behavior, or a separately accepted risk decision.

## Explicit Boundary

This document approves no code and no writes. It does not implement HID writes,
does not open HID devices, does not call `SetFeature` or `GetFeature`, does not
touch `/dev/hidraw*`, `/dev/port`, SMBus, or Super I/O, and does not enable
MS-7E75 hardware access.
