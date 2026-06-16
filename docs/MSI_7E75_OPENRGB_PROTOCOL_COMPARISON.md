# MSI MS-7E75 OpenRGB Protocol Comparison

Status: documentation only. This document approves no code and no writes.

## Purpose

This note compares the repo's static MSI MS-7E75 / MB800 HID report evidence
against OpenRGB's MSI Mystic Light HID implementation.

Scope of comparison:

- common VID/PID handling for `0DB0:0076`
- `0x50` / 290-byte report behavior
- `0x51` / 761-byte discovery behavior and OpenRGB's later per-zone packets
- `JARGB`, `JRAINBOW`, and `JAF` naming
- any 302-byte / `0x90..0x93` near match
- transport API shape
- safety and recovery implications

No HID devices were opened for this pass. No `SetFeature` or `GetFeature`
calls were made.

## Sources Compared

Repo-side sources:

- [MSI_7E75_HID_MB800_STATIC_RE.md](MSI_7E75_HID_MB800_STATIC_RE.md)
- [MSI_7E75_MSIHID_STATIC_RE.md](MSI_7E75_MSIHID_STATIC_RE.md)
- [MSI_7E75_EXTERNAL_HID_EVIDENCE.md](MSI_7E75_EXTERNAL_HID_EVIDENCE.md)
- [MSI_7E75_PRE_WRITE_RISK_ASSESSMENT.md](MSI_7E75_PRE_WRITE_RISK_ASSESSMENT.md)

OpenRGB source checked:

- local ignored checkout `.scratch/OpenRGB`
- commit `a8354ee0464150df8650f7a11396a607227bb65b`
- commit subject: `Add Gigabyte Aorus Waterforce X II 360 AIO controller
  (Castor3)`
- relevant files:
  - `Controllers/MSIMysticLightController/MSIMysticLightControllerDetect.cpp`
  - `Controllers/MSIMysticLightController/MSIMysticLightCommon.h`
  - `Controllers/MSIMysticLightController/MSIMysticLight761Controller/MSIMysticLight761Controller.cpp`
  - `Controllers/MSIMysticLightController/MSIMysticLight761Controller/RGBController_MSIMysticLight761.cpp`
  - older `112`, `162`, and `185` Mystic Light controller files for context

## High-Level Result

OpenRGB provides strong external confirmation for the common MSI Mystic Light
HID family and partial confirmation for a `0x50` / 290-byte report on newer
boards.

OpenRGB does not confirm the MS-7E75 MB800 basic Gen2 apply reports:

- no `MS-7E75` board entry
- no `NUC126`, `MB800`, or `Class_MB_800` naming
- no 302-byte report path found
- no JARGB/JAF report IDs `0x90`, `0x91`, `0x92`, or `0x93`
- OpenRGB's closest newer path uses `0x51` per-zone/per-LED packets, not
  `0x90 + port`

The comparison keeps Phase 4 deferred.

## Side-By-Side Protocol Shape

| Topic | MS-7E75 MB800 static evidence | OpenRGB evidence | Match assessment |
| --- | --- | --- | --- |
| VID/PID | Opens common VID `0x0DB0`, PID `0x0076`. | Defines `MSI_USB_VID_COMMON 0x0DB0` and `MSI_USB_PID_COMMON 0x0076`; registers common and X870 detectors. | Strong family match. |
| Board selection | Parses HID serial prefix and expects board ID `0x7E75`; static profile route is `MS-7E75` -> `NUC126_MB800`. | Uses DMI board name for the `761` board config table; no `MS-7E75` entry. | Partial mechanism match, no board match. |
| Gen1/basic setup | `0x50`, length `290`, 18 records, JRGB1 at area index `9`, store flag at byte `289`. | `initial_setup_array` begins with `0x50`; `SETUP_ARRAY_SIZE` is `290`; rows include JARGB, JAF, JPIPE, JRGB, onboard, and select-all-like entries. | Strong report ID/length family match, layout differs. |
| Gen2 basic ports | `0x90 + port`; `0x90..0x93`; length `302`; six 20-byte strip records; store flag at byte `301`. | No `0x90..0x93` JARGB/JAF path found in the Mystic Light 761 controller. | Mismatch / not found. |
| Advanced/per-LED | MB800 static notes include Gen1 advanced `0x51` length `727`, and Gen2 advanced `0xB0 + port` length `761`. | Detection sends `0x51` with a length argument of `761`; later updates send `FeaturePacket_PerLED_761`, whose byte fields total `727` bytes. | Near match around `0x51`/727, mismatch for MB800 Gen2 advanced `0xB0 + port`. |
| Zones | MS-7E75 uses `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, `EZ Conn`. | 761 controller exposes `JAF`, `JARGB 1`, `JARGB 2`, `JARGB 3`; common enum also contains `J_RGB_*` and `J_RAINBOW_*`. | Naming/family match, not exact profile mapping. |
| Transport | MSI `MsiHid.dll` wraps Windows HID APIs; MB800 `SetFeature` calls end in `HidD_SetFeature(handle, data, length)`. | OpenRGB controller code calls hidapi `hid_get_feature_report` and `hid_send_feature_report`. | Same HID feature-report concept; backend details are platform-dependent. |
| Safety posture | Repo blocks Phase 4; no device opens/writes approved. | OpenRGB source warns Mystic Light had past bricking risk and untested boards are at user risk. | External safety signal supports caution. |

## `0DB0:0076` Handling

### MS-7E75 MB800

The static MSI Center path uses:

- `VID_MSI_Common = 3504` (`0x0DB0`)
- `PID_MSI_Common = 118` (`0x0076`)
- `openMyDevice_Read(3504, 118, 0, 0, 1)`
- serial-prefix parsing to derive or validate the board-like ID

The validated real-machine read-only path observed one MSI MYSTIC LIGHT
candidate with VID `0x0DB0`, PID `0x0076`, and serial prefix `7E75`.

### OpenRGB

OpenRGB's Mystic Light detector defines a common MSI VID `0x0DB0` and a common
PID `0x0076`. It registers:

- `MSI Mystic Light Common`
- `MSI Mystic Light X870`

Both route to the same Mystic Light detector function. The X870/common detector
uses HID usage page / usage filters in the detector macro, while MB800 static
evidence currently only proves MSI's native path token filtering and attribute
checks.

### Assessment

This is the strongest exact match. OpenRGB corroborates that `0DB0:0076` is a
real Mystic Light common-controller identity, including for recent MSI
motherboards. It does not prove that all `0DB0:0076` boards share the same
report semantics.

## `0x50` / 290-Byte Report

### MS-7E75 MB800

The MS-7E75 static notes show Gen1 board apply:

- report selector byte `0x50`
- total length `290`
- 18 area records
- `JRGB1` maps to area index `9`
- byte `289` is the store flag
- transport call is `HID_Basic.SetFeature(_Device, array, 290)`

### OpenRGB

OpenRGB's 761 path has:

- `SETUP_ARRAY_SIZE 290`
- `initial_setup_array[]` beginning with `0x50`
- comments for `JARGB 1`, `JARGB 2`, `JARGB 3`, `JAF`, `JPIPE`, `JRGB`, and
  onboard rows
- constructor code that sends the configuration with
  `hid_send_feature_report(dev, conf_arr, SETUP_ARRAY_SIZE)`

OpenRGB's detector also uses report byte `0x50` with a length argument of `290`
in a `hid_get_feature_report` call before trying the 761-style path.

### Assessment

This is a meaningful near match. It suggests newer OpenRGB Mystic Light support
also treats `0x50` / `290` as a setup/configuration report. The contents and
flow are not identical enough to validate a future MS-7E75 write, especially
because OpenRGB uses it as part of a broader initialization/configuration
sequence and not as proof of the repo's exact JRGB1-only payload.

## `761`-Byte Behavior

### MS-7E75 MB800

The MB800 static notes contain two advanced paths:

- Gen1 advanced: selector `0x51`, length `727`
- Gen2 advanced: selector `0xB0 + port`, length `761`

The current Phase 4 candidate is not either advanced path. It is the basic
Gen2 port report `0x90`, length `302`.

### OpenRGB

OpenRGB's detection fallback:

- allocates a `761`-byte buffer
- sets byte `0` to `0x51`
- sends it via `hid_send_feature_report(dev, second_buffer, 761)`
- constructs `MSIMysticLight761Controller` if that send returns success

OpenRGB's controller update path later sends four zone packets:

- `data->jaf.packet`
- `data->jargb1.packet`
- `data->jargb2.packet`
- `data->jargb3.packet`

Those packets use `FeaturePacket_PerLED_761`, whose visible fields are:

- report ID `0x51`
- fixed byte `0x09`
- `hdr0`
- `hdr1`
- two fixed zero bytes
- `hdr2`
- `720` color bytes

That visible structure totals `727` bytes, even though the controller family is
named `761` and the detection fallback sends `761` bytes.

OpenRGB zone headers:

- `JAF`: `hdr0 = 0x08`, `hdr1 = 0x00`
- `JARGB 1`: `hdr0 = 0x04`, `hdr1 = 0x00`
- `JARGB 2`: `hdr0 = 0x04`, `hdr1 = 0x01`
- `JARGB 3`: `hdr0 = 0x04`, `hdr1 = 0x02`
- all initialized with fixed byte `0x09` and LED count/header byte `240`

### Assessment

OpenRGB's `0x51` path is closer to the repo's MB800 Gen1 advanced `0x51` /
`727` evidence than to the MB800 Gen2 advanced `0xB0 + port` / `761` evidence.
It is not a match for the proposed basic Gen2 `0x90..0x93` / `302` path.

This matters because a "newer MSI common controller" match could point toward a
different protocol generation than the one statically decoded from MSI Center's
MB800 basic path.

## Zone Naming Comparison

### MS-7E75 MB800

The repo's decoded MS-7E75 profile exposes:

- `JRGB1`
- `JARGB_V2_1`
- `JARGB_V2_2`
- `JARGB_V2_3`
- `EZ Conn`

The MB800 path maps them as:

- `JRGB1` -> Gen1 report `0x50`, area `9`
- `JARGB_V2_1` -> Gen2 report `0x90`, port `0`
- `JARGB_V2_2` -> Gen2 report `0x91`, port `1`
- `JARGB_V2_3` -> Gen2 report `0x92`, port `2`
- `EZ Conn` -> Gen2 report `0x93`, port `3`

### OpenRGB

OpenRGB common enums include older zone names:

- `MSI_ZONE_J_RGB_1`
- `MSI_ZONE_J_RGB_2`
- `MSI_ZONE_J_RAINBOW_1`
- `MSI_ZONE_J_RAINBOW_2`
- `MSI_ZONE_J_RAINBOW_3`

OpenRGB's 761 controller exposes:

- `JAF`
- `JARGB 1`
- `JARGB 2`
- `JARGB 3`

The code comments describe `JAF` as the connector where fans go with the
"weird connector", which is a plausible near match for MSI Center's `EZ Conn`
concept.

### Assessment

Zone naming is a useful family-level match:

- `JRGB` and `JRAINBOW` are present in older OpenRGB Mystic Light code.
- `JARGB 1/2/3` and `JAF` are present in the newer OpenRGB 761 path.
- `JAF` is plausibly related to the repo's `EZ Conn`.

The mapping does not validate the report IDs. OpenRGB maps the newer JARGB/JAF
zones into `0x51` per-zone/per-LED packets, not `0x90..0x93`.

## 302-Byte / `0x90..0x93` Search Result

No OpenRGB Mystic Light source hit was found for a 302-byte JARGB/JAF report
path or for report IDs `0x90`, `0x91`, `0x92`, `0x93` in the MSI Mystic Light
761 controller.

The only relevant OpenRGB `0x90..0x93` hits in broader searches were from other
devices or unrelated protocols, not MSI Mystic Light JARGB/JAF motherboard
feature reports.

This is the decisive mismatch for Phase 4 risk. The current proposed first HID
write candidate remains:

- zone `JARGB_V2_1`
- report `0x90`
- length `302`
- one Gen2 port

OpenRGB does not independently confirm that report.

## Feature Report Transport

### MS-7E75 MB800

MSI's native `MsiHid.dll` wrapper statically imports Windows HID APIs. The
managed MB800 caller passes full buffers to:

- `HidD_SetFeature(handle, data, length)`
- `HidD_GetFeature(handle, data, length)`

Static notes found no wrapper evidence for stripping byte `0`, rewriting report
IDs, adding checksums, or changing lengths.

### OpenRGB

OpenRGB's controller source calls hidapi:

- `hid_get_feature_report`
- `hid_send_feature_report`

The controller code itself does not call Linux `hidraw` directly. The lower
backend is hidapi/platform dependent. Conceptually, however, OpenRGB is using
HID feature reports rather than SMBus, Super I/O, WMI, or a raw port backend
for this path.

### Assessment

Transport concept matches: both paths are HID feature-report based.

Transport concept is not sufficient for write approval. A HID feature report
with the wrong report ID, length, initialization state, or zone mapping could
still have unsafe effects.

## Safety / Recovery Notes

OpenRGB's Mystic Light detector contains a source warning that MSI Mystic Light
controllers had a bricking risk in the past, says the issue was fixed for a few
tested boards, and says untested boards are at user risk.

External issue history also contains:

- older Mystic Light bricking reports
- newer `0DB0:0076` B850/X870 reports where OpenRGB detects the device but does
  not initialize a controllable RGB device
- reports of descriptor/read failures or `packet length = -1`

No OpenRGB source checked provides a board-specific MS-7E75 recovery path. No
source checked proves that MSI Center, BIOS, firmware, or power-cycle recovery
can restore an MS-7E75 controller after a bad custom feature report.

## What This Comparison Confirms

- `0DB0:0076` is a real MSI Mystic Light common HID identity.
- OpenRGB handles that common VID/PID for at least some recent MSI boards.
- OpenRGB uses HID feature-report APIs for Mystic Light motherboard paths.
- OpenRGB's newer 761 path has JARGB/JAF-style zones.
- OpenRGB's newer path includes a `0x50` / `290` setup/config report.
- OpenRGB's newer path uses `0x51` per-zone/per-LED packets after setup.
- OpenRGB source and issue history support a conservative safety posture.

## What This Comparison Does Not Confirm

- It does not confirm `MS-7E75` support in OpenRGB.
- It does not confirm `NUC126` or `MB800`.
- It does not confirm `JARGB_V2` naming.
- It does not confirm report `0x90`, `0x91`, `0x92`, or `0x93`.
- It does not confirm 302-byte Gen2 basic reports.
- It does not confirm a one-report first-write path.
- It does not confirm a recovery path.
- It does not prove Linux behavior matches MSI's Windows wrapper behavior.

## Risk Conclusion

This comparison does not lower write risk enough to proceed.

It strengthens family confidence around the common HID identity and around
`0x50` / `290` setup-style traffic. It also shows that OpenRGB's closest newer
implementation uses a different JARGB/JAF report family than the repo's current
MB800 basic Gen2 `0x90..0x93` reports.

Recommendation: keep Phase 4 deferred and stay read-only. Do not implement or
run HID writes for MS-7E75 unless a later review obtains stronger evidence for
the exact MB800 report IDs, lengths, initialization flow, zone mapping, and
recovery behavior.

## Explicit Boundary

This document approves no code and no writes. It does not implement HID writes,
does not open HID devices, does not call `SetFeature` or `GetFeature`, does not
touch `/dev/hidraw*`, `/dev/port`, SMBus, or Super I/O, and does not enable
MS-7E75 hardware access.
