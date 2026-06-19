# MSI MS-7E75 USBPcap Capture Notes

Status: documentation only. This document approves no code and no writes.

## Capture Scope

A passive Windows USBPcap/Wireshark capture was taken while MSI Center controlled
the real MSI MS-7E75 / B850 GAMING PLUS WIFI PZ board.

USBPcap evidence was taken from USBPcap4, device `4.4.0`, packet type
`HCI_USB 326 Sent`.

Two passive captures were analyzed offline:

- `msi_7e75_0x50_capture.pcapng`
- `msi_7e75_jarbg_v2_1_capture.pcapng`

The exact MSI Center UI path used was:

```text
MB -> JARGB_V2_1
```

The capture scope was limited to observing MSI Center traffic while applying
changes through that UI path. No custom packet was sent by this project. This
evidence does not approve Linux HID writes.

## Observed SET_REPORT Frames

The relevant frames observed for `MB -> JARGB_V2_1` were HID class
`SET_REPORT` transfers with:

- `wValue = 0x0350`
- `ReportID = 0x50`
- `ReportType = Feature`
- `wLength = 290`
- data starts with `0x50`
- observed frames include `4781` and `7757`

No live MSI Center traffic in the analyzed captures contained:

- `0x90` / 302
- `0x91` / 302
- `0x92` / 302
- `0x93` / 302
- `0x51` / 727
- `0xB0` / 761

No `0x0390` / 302-byte report was observed for this MSI Center UI action.

Confirmed live tests for `MB -> JARGB_V2_1`:

- steady / `R108 G225 B40` / LED changed yes
- steady / `R255 G0 B0` / LED changed yes
- steady / `R0 G255 B0` / LED changed yes
- steady / `R0 G0 B255` / LED changed yes
- breath / `R255 G0 B0` / LED changed yes
- off / LED changed yes

Observed report starts and store bytes from
`msi_7e75_jarbg_v2_1_capture.pcapng` and the newer USBPcap4 tests:

| Frame | Report start | Store byte `[289]` | Notes |
| --- | --- | --- | --- |
| `4781` | `50 02 14 ff 09 00 ff 00 00 00 ff ff ff ff 00 35 1e ...` | `0x00` | Feature report ID `0x50`, 290-byte transfer. |
| `7757` | `50 03 ff 00 00 ff 64 00 00 00 ff ff ff ff 01 35 1e ...` | `0x01` | Feature report ID `0x50`, 290-byte transfer. |
| USBPcap4 steady red | `50 02 ff 00 00 ...` | `0x01` | Mode `0x02`, RGB begins `ff 00 00`. |
| USBPcap4 steady green | `50 02 00 ff 00 ...` | `0x01` | Mode `0x02`, RGB begins `00 ff 00`. |
| USBPcap4 steady blue | `50 02 00 00 ff ...` | `0x01` | Mode `0x02`, RGB begins `00 00 ff`. |
| USBPcap4 breath red | `50 04 ff 00 00 ...` | `0x01` | Mode `0x04`, RGB begins `ff 00 00`. |
| USBPcap4 off | `50 00 ff 00 00 ...` | `0x01` | Mode `0x00`; RGB remained red, so off is mode-driven. |

## USB Setup Byte Decoding

USB setup bytes observed at offset `0x1c`:

```text
21 09 50 03 00 00 22 01
```

Decoded:

| Bytes | Field | Value | Interpretation |
| --- | --- | --- | --- |
| `21` | `bmRequestType` | `0x21` | Host-to-device, class, interface request. |
| `09` | `bRequest` | `0x09` | HID `SET_REPORT`. |
| `50 03` | `wValue` | `0x0350` | Feature report, report ID `0x50`. |
| `00 00` | `wIndex` | `0x0000` | Interface index observed in setup packet. |
| `22 01` | `wLength` | `0x0122` | 290 bytes. |

## Report Summary

| Evidence source | UI path | Report ID | Report type | Length | Status |
| --- | --- | --- | --- | --- | --- |
| USBPcap frame `4781` | `MB -> JARGB_V2_1` | `0x50` | Feature | `290` | Live MSI Center traffic observed. |
| USBPcap frame `7757` | `MB -> JARGB_V2_1` | `0x50` | Feature | `290` | Live MSI Center traffic observed. |
| Prior static/decompiled notes | `JARGB_V2_1` candidate path | `0x90` | Feature | `302` | Static/decompiled evidence only; not observed in this capture. |

## Confirmed Payload Meanings

Confirmed from the newer USBPcap4 captures:

- `payload[0] = 0x50`
- `payload[1] = mode`
- steady mode observed as `0x02`
- breath mode observed as `0x04`
- off mode observed as `0x00`
- `payload[2..5]` begins with the first RGB triplet for steady and breath
- red begins `ff 00 00`
- green begins `00 ff 00`
- blue begins `00 00 ff`
- in the off capture, RGB remained `ff 00 00` while mode changed to `0x00`
- `payload[289]` was `0x01` for the clean red/green/blue/breath/off tests

Important limit:

- an earlier random-color capture had `payload[289] = 0x00`
- therefore byte `[289]` should be documented only as observed store/apply
  metadata, not as a fully understood flag yet

## Key Conclusion

The only live MSI Center write path observed so far is `0x50`/290.

For the real MS-7E75 / B850 GAMING PLUS WIFI PZ board, selecting
`MB -> JARGB_V2_1` in MSI Center and applying changes produced HID Feature
`SET_REPORT` traffic using report ID `0x50` with a 290-byte payload. It did not
produce an observed report ID `0x90` with a 302-byte payload.

Do not use `JARGB_V2_1 -> 0x90` as a first-write plan. `0x90..0x93` remain
static/decompiled evidence only until live traffic confirms them.

The live MSI Center UI path observed for JARGB_V2_1 is 0x50/290.
0x90..0x93 are not live-confirmed.
Do not use 0x90 as the first-write target.
This evidence does not approve Linux HID writes.

Required wording:

- Do not use JARGB_V2_1 -> 0x90 as a first-write plan.
- 0x90..0x93 remain static/decompiled evidence only until live traffic confirms
  them.
- The only live MSI Center write path observed so far is 0x50/290.
- This does not approve Linux writes.
- The live MSI Center UI path observed for JARGB_V2_1 is 0x50/290.
- 0x90..0x93 are not live-confirmed.
- Do not use 0x90 as the first-write target.
- This evidence does not approve Linux HID writes.

## Impact On Previous Assumptions

Earlier static notes identified a decompiled MB800 Gen2 helper shape where
`JARGB_V2_1` mapped to report `0x90`, length `302`. The passive USBPcap capture
does not validate that shape for the observed MSI Center UI action.

This changes the risk posture:

- the live UI behavior must take precedence over the static first-write
  assumption
- any previous `JARGB_V2_1 -> 0x90` first-write candidate is now explicitly
  blocked
- `0x50`/290 is the only observed live MSI Center write family for this UI path
  so far
- the observed `0x50`/290 traffic still does not prove a safe Linux write path
  or a complete protocol

## Builder Comparison

Offline comparison support was added in
[`src/linux/hid/capture_compare.rs`](../src/linux/hid/capture_compare.rs). It
only parses pasted hex fixtures and compares byte slices against the existing
in-memory report builder. It has no CLI entry point and no device access.

That module now also includes a separate offline-only local generator for the
live-confirmed `JARGB_V2_1` `0x50`/290 payload prefixes observed in the USBPcap
captures. It is intentionally separate from the existing broad Gen1/Gen2 report
builders because the older static builder model does not by itself explain the
live `JARGB_V2_1` `0x50` path.

The current embedded fixtures use the documented frame starts:

| Frame | Embedded fixture bytes | Store byte metadata | Source form |
| --- | --- | --- | --- |
| `4781` | `50 02 14 ff 09 00 ff 00 00 00 ff ff ff ff 00 35 1e` | `[289] = 0x00` | HID report payload prefix plus pcap-derived store byte metadata. |
| `4781` | `21 09 50 03 00 00 22 01 50 02 14 ff 09 00 ff 00 00 00 ff ff ff ff 00 35 1e` | `[289] = 0x00` | USB setup bytes followed by HID report payload prefix. |
| `7757` | `50 03 ff 00 00 ff 64 00 00 00 ff ff ff ff 01 35 1e` | `[289] = 0x01` | HID report payload prefix plus pcap-derived store byte metadata. |

The offline extractor treats `21 09 50 03 00 00 22 01` as an 8-byte USB setup
packet and extracts the HID report payload beginning at the following `0x50`.
If a fixture already begins with `0x50`, it is treated as a direct HID report
payload prefix.

Known matches:

| Offset / field | Evidence |
| --- | --- |
| USB setup `wValue` | `0x0350`, report type `0x03`, report ID `0x50`. |
| USB setup `wLength` | `0x0122`, 290 bytes, matching `GEN1_REPORT_LENGTH`. |
| HID payload byte `[0]` | `0x50`, matching the Gen1 report builder report ID. |
| Store byte `[289]` | Frame `4781` has `0x00`; frame `7757` has `0x01`, matching the documented Gen1 store-byte offset. |
| HID payload byte `[1]` | Observed live mode byte: steady `0x02`, breath `0x04`, off `0x00`. |
| HID payload bytes `[2..4]` | Observed live RGB prefix: red `ff0000`, green `00ff00`, blue `0000ff`. |
| Offline local generator/check | Builds the observed `JARGB_V2_1` `0x50`/290 payload prefix for steady red/green/blue, breath red, and off with retained red. |

Known differences in the available prefixes:

| Frame | Differing prefix offsets versus current Gen1 `JRGB1` static-red builder fixture | Gen1 layout interpretation |
| --- | --- | --- |
| `4781` | `[1]`, `[2]`, `[3]`, `[4]`, `[6]`, `[10]`, `[11]`, `[12]`, `[13]`, `[15]`, `[16]` | Area 0 mode/color and option/cycle-like bytes differ from the builder fixture, which leaves area 0 zeroed when building only `JRGB1` area 9. |
| `7757` | `[1]`, `[2]`, `[5]`, `[6]`, `[10]`, `[11]`, `[12]`, `[13]`, `[14]`, `[15]`, `[16]` | Area 0 mode/color and option/cycle-like bytes differ from the builder fixture, which leaves area 0 zeroed when building only `JRGB1` area 9. |

Unknowns:

- The embedded byte strings still include only the first payload bytes, not all
  290 payload bytes.
- Store byte `[289]` is represented as observed metadata in tests and docs; the
  middle bytes between the prefix and store byte are not embedded here, and the
  exact meaning of byte `[289]` is still not proven.
- The visible differences may represent MSI Center's full-board `0x50` state,
  selected mode, colors, brightness/options, persistence, or unrelated populated
  areas. The current evidence is not enough to assign those meanings safely.
- The builder must not be changed to match these prefixes unless static or
  live evidence proves the byte meanings.
- `0x90..0x93` are not live-confirmed and remain static/decompiled evidence
  only until live traffic confirms them.

## Phase 4 Status

Phase 4 remains on hold.

This evidence does not approve Linux HID writes, HID device opens, `SetFeature`,
`GetFeature`, `write-once`, `/dev/hidraw*`, `/dev/port`, SMBus, or Super I/O
access.

## Next Safe Analysis Steps

- Decode more of the observed `0x50`/290 payloads offline from the passive
  capture.
- Compare the observed payload bytes against the decompiled Gen1 `0x50` layout
  and OpenRGB's `0x50`/290 setup-style evidence.
- Capture additional passive MSI Center UI actions for `JRGB1`,
  `JARGB_V2_2`, `JARGB_V2_3`, `EZ Conn`, and select-all behavior.
- Check whether MSI Center emits any separate initialization, restore, or
  persistence traffic before or after the visible UI apply.
- Preserve the rule that live traffic confirmation is required before any
  report family becomes a write candidate.
- Keep all repo-side analysis documentation-only unless a later task explicitly
  authorizes a new reviewed implementation phase.
