# MSI MS-7E75 External HID Evidence Notes

Status: documentation only. This document approves no code and no writes.

## Purpose

This note records an external/open-source evidence pass for the MSI MS-7E75 /
B850 GAMING PLUS WIFI PZ HID lighting path.

The target internal hypothesis remains:

- USB VID `0x0DB0`
- USB PID `0x0076`
- product name `MSI MYSTIC LIGHT`
- MSI Center static route through `NUC126_MB800` / `MB800`
- Gen1 report `0x50`
- Gen2 reports `0x90`, `0x91`, `0x92`, `0x93`
- lighting zones `JRGB`, `JRAINBOW`, `JARGB_V2`

No HID devices were opened for this pass. No `SetFeature` or `GetFeature`
calls were made.

## Sources Checked

External sources checked on 2026-06-16:

- OpenRGB source mirror, local shallow clone at commit
  `a8354ee0464150df8650f7a11396a607227bb65b`
  - `Controllers/MSIMysticLightController/MSIMysticLightControllerDetect.cpp`
  - `Controllers/MSIMysticLightController/MSIMysticLightCommon.h`
  - `Controllers/MSIMysticLightController/MSIMysticLight761Controller/*`
  - `Controllers/MSIMysticLightController/MSIMysticLight185Controller/*`
- OpenRGB supported-device page for `0076`:
  <https://openrgb.org/devices.html?search=0076>
- OpenRGB GitLab issues / work items:
  - <https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/4359>
  - <https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/4645>
  - <https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/4800>
  - <https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/4900>
  - <https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/4910>
  - <https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/4944>
  - <https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/5429>
- liquidctl source, local shallow clone at commit
  `6227bbfdfe33db2a8e12361b9a96495c00676275`
  - `liquidctl/driver/msi.py`
  - `docs/developer/protocol/coreliquid.md`
- Linux kernel upstream files from `torvalds/linux` `master`:
  - `drivers/hid/hid-ids.h`
  - `drivers/hid/hid-quirks.c`
  - `drivers/hid/Kconfig`
  - `drivers/hid/Makefile`
  - `drivers/hid/hid-gt683r.c`
- LKML patch result for `drivers/hid/hid-msi.c` RGB support:
  <https://lkml.iu.edu/2605.2/12442.html>
- MSI public Mystic Light page / SDK page:
  <https://www.msi.com/Landing/mystic-light-rgb-gaming-pc/>
- SignalRGB public MSI troubleshooting page:
  <https://docs.signalrgb.com/troubleshooting/brand-specific/msi/>
- `garashchenko/mystic-why` issue context for older Mystic Light experiments:
  <https://github.com/garashchenko/mystic-why/issues/2>
- OpenRGB bricking-risk reports:
  - <https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/389>
  - <https://www.reddit.com/r/MSI_Gaming/comments/m38e0e/mystic_light_support_enabled_in_openrgb/>

A focused protocol comparison with OpenRGB's MSI Mystic Light implementation is
recorded in
[MSI_7E75_OPENRGB_PROTOCOL_COMPARISON.md](MSI_7E75_OPENRGB_PROTOCOL_COMPARISON.md).
It confirms the common HID identity and `0x50` / `290` family evidence, but
does not confirm the MS-7E75 MB800 `0x90..0x93` / `302` reports.

## Exact Matches Found

### `0DB0:0076`

External evidence strongly confirms that recent MSI Mystic Light motherboard
controllers can enumerate as USB VID `0x0DB0`, PID `0x0076`.

OpenRGB source has:

- `MSI_USB_VID_COMMON 0x0DB0`
- `MSI_USB_PID_COMMON 0x0076`
- a detector named `MSI Mystic Light Common`
- a detector named `MSI Mystic Light X870`
- both common detectors route to `DetectMSIMysticLightControllers`

OpenRGB issues contain repeated user-provided logs for `0DB0:0076` devices
reported as `MSI - MYSTIC LIGHT` or `Micro Star International MYSTIC LIGHT`.
Examples include:

- MSI MAG B850M MORTAR WIFI `MS-7E61`, serial prefix `7E61`
- MSI MAG B850 TOMAHAWK / TOMAHAWK MAX WIFI `MS-7E62`, serial prefix `7E62`
- MSI MPG X870E CARBON WIFI `MS-7E49`
- MSI MAG X870E TOMAHAWK WIFI `MS-7E59`
- MSI PRO X870-P WIFI `MS-7E47`
- MSI PRO Z890-S WIFI PZ `MS-7E58`

This is a strong match for the target VID/PID/product identity, but not a
board-exact match for `MS-7E75`.

### JARGB-style zones in newer OpenRGB code

OpenRGB's newer `MSIMysticLight761Controller` includes zones named:

- `JAF`
- `JARGB 1`
- `JARGB 2`
- `JARGB 3`

That is a near-exact naming match to the MSI Center `JARGB_V2` / EZ Connector
concept in this repo, but OpenRGB does not use the exact `JARGB_V2` string.

### Report `0x50`

OpenRGB's `MSIMysticLight761Controller` has an `initial_setup_array` beginning
with report byte `0x50` and sized as `290` bytes by `SETUP_ARRAY_SIZE`.

This externally corroborates that a `0x50` / `290`-byte setup-style report is
used in at least one newer Mystic Light common-controller implementation.

## Near Matches Found

### OpenRGB `761-byte` controller

The closest external source is OpenRGB's `MSIMysticLight761Controller`, added
for newer X870/B850/Z890-era boards. It is close because it:

- is in the MSI Mystic Light motherboard controller family
- is reached from the common `0DB0:0076` detector
- has a board table containing multiple B850/X870 boards
- supports `JARGB 1`, `JARGB 2`, `JARGB 3`, and `JAF`
- sends a `0x50` setup/configuration report of length `290`
- uses HID feature reports through hidapi

It is not an exact match because it:

- does not list `MS-7E75`
- does not name `NUC126`, `MB800`, or `Class_MB_800`
- does not implement the repo's current basic Gen2 report IDs
  `0x90..0x93`
- appears to use report `0x51` per-zone/per-LED packets for the newer direct
  path, not `0x90 + port`
- sends broader startup/configuration traffic than the repo's proposed one-zone
  first-write design

This source is useful family evidence, but it is not a safe one-to-one protocol
confirmation for the proposed MB800 basic-port report path.

### OpenRGB older `112` / `162` / `185` Mystic Light controllers

OpenRGB's older Mystic Light controllers are useful for context:

- they are MSI Mystic Light motherboard HID implementations
- they include `JRGB` / `JRAINBOW` zones
- the `185` controller has a common-PID board entry for `0x0076`
- external reports discuss an older bricking issue around the `185`-byte path

They are not exact matches:

- their primary report ID is `0x52`, not `0x50` or `0x90..0x93`
- their zone layouts are older `JRGB` / `JRAINBOW` style, not MB800
  `JARGB_V2`
- they do not name `MS-7E75`, `NUC126`, or `MB800`

### liquidctl MSI driver

liquidctl has MSI support, but it appears to target CoreLiquid devices rather
than Mystic Light motherboard headers:

- it matches MSI VID `0x0DB0`
- it matches PIDs `0xB130`, `0xCA00`, and `0xCA02`
- it contains `JRAINBOW` and `JRGB` constants
- it has OLED/user-message paths using `0x90..0x93` in comments or unrelated
  display-message helpers

This is not a match for `0DB0:0076`, MB800, NUC126, or motherboard JARGB_V2
lighting writes.

### Linux kernel HID sources

Mainline Linux HID sources checked do not show a Mystic Light motherboard
driver or quirk for `0DB0:0076`.

The kernel has MSI HID references for the older GT683R LED panel using VID
`0x1770`, PID `0xff00`. A searched LKML patch for `hid-msi.c` is about MSI
Claw-style device RGB/control support, not Mystic Light motherboard HID.

This is useful negative evidence: there is no mainline kernel quirk or driver
confirming the target board-lighting protocol.

### SignalRGB

OpenRGB's `MSIMysticLight761Controller` comments say its direct-mode
functionality was implemented based on SignalRGB, but public SignalRGB docs
checked did not expose protocol details for `0DB0:0076`, MB800, NUC126, or
`0x90..0x93`.

SignalRGB remains a possible practical implementation source, but no
open-source packet evidence was found in this pass.

## Mismatches / Not Found

No external source checked confirmed all of the following together:

- board `MS-7E75`
- `0DB0:0076`
- `NUC126` or `NUC126_MB800`
- `MB800` or `Class_MB_800`
- Gen2 feature reports `0x90`, `0x91`, `0x92`, `0x93`
- report length `302`
- ports corresponding to `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, and
  `EZ Conn`

Specific non-finds:

- No OpenRGB source hit for `MS-7E75`.
- No OpenRGB source hit for `NUC126`, `MB800`, or `MB_800`.
- No OpenRGB `MSIMysticLight761Controller` hit for `0x90`, `0x91`, `0x92`,
  or `0x93` as JARGB report IDs.
- No liquidctl match for `0DB0:0076`.
- No Linux kernel HID quirk for `0DB0:0076`.
- No Linux kernel Mystic Light motherboard HID driver matching this path.
- No public MSI SDK protocol details found for the specific HID reports.

## Answers To The Evidence Questions

### Does any source confirm `0DB0:0076`?

Yes.

OpenRGB source and multiple OpenRGB issues confirm `0DB0:0076` as a common MSI
Mystic Light HID identity on newer MSI motherboards. This lowers uncertainty
about device identity, but only at the VID/PID/product-family level.

### Does any source confirm report IDs `0x90..0x93`?

No strong external match was found for `0x90..0x93` as MB800 / NUC126 /
JARGB_V2 motherboard feature reports.

The repo's current `0x90..0x93` evidence still comes from internal static MSI
Center / LEDKeeper analysis, not from an independent open-source
implementation.

### Does any source confirm report ID `0x50`?

Partially.

OpenRGB's newer `761-byte` path uses a `0x50` setup/configuration report of
length `290`, which aligns with the repo's Gen1/basic setup-size evidence.
However, OpenRGB uses it in a different newer-controller setup flow, so it does
not by itself confirm that MS-7E75 should receive the repo's exact `0x50`
payload or any later `0x90..0x93` payload.

### Does external evidence lower write risk?

Only slightly, and only for identity/family confidence.

External evidence lowers the chance that `0DB0:0076` is an unrelated device.
It does not materially lower first-write risk for the proposed MS-7E75 MB800
basic-port reports because the decisive reports `0x90..0x93` were not confirmed
by an independent open-source implementation.

External evidence also keeps the risk picture serious:

- OpenRGB source carries an MSI Mystic Light bricking-risk warning.
- OpenRGB issue history includes older Mystic Light controller bricking
  reports.
- Newer OpenRGB issues show `0DB0:0076` B850/X870 boards that are detected but
  not always initialized successfully.
- Some issue reports show descriptor/read failures or `packet length = -1`.
- The closest OpenRGB newer-controller path sends setup and per-zone/per-LED
  reports that do not match the repo's planned one-report `0x90` first write.

## Recommendation

Stay read-only.

The external evidence is strong enough to document that `0DB0:0076` is a real
MSI Mystic Light common HID identity on recent boards. It is not strong enough
to approve any HID open, `GetFeature`, `SetFeature`, or write path for
MS-7E75.

The OpenRGB protocol comparison does not change this recommendation. OpenRGB's
closest newer JARGB/JAF path uses `0x51` per-zone/per-LED packets rather than
the repo's proposed MB800 `0x90..0x93` / `302` basic Gen2 reports.

Do not implement or run Phase 4 unless a later evidence pass finds a
board-family-exact source confirming the MB800/NUC126 report IDs, lengths, zone
mapping, and recovery behavior, or unless a separate review explicitly accepts
the remaining uncertainty.

## Explicit Boundary

This document approves no code and no writes. It does not implement HID writes,
does not open HID devices, does not call `SetFeature` or `GetFeature`, does not
touch `/dev/hidraw*`, `/dev/port`, SMBus, or Super I/O, and does not enable
MS-7E75 hardware access.
