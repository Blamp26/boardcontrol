# MSI MS-7E75 USBPcap Capture Notes

Status: documentation only. This document approves no code and no writes.

## Capture Scope

A passive Windows USBPcap/Wireshark capture was taken while MSI Center controlled
the real MSI MS-7E75 / B850 GAMING PLUS WIFI PZ board.

The exact MSI Center UI path used was:

```text
MB -> JARGB_V2_1
```

The capture scope was limited to observing MSI Center traffic while applying
changes through that UI path. No custom packet was sent by this project. This
does not approve Linux writes.

## Observed SET_REPORT Frames

The relevant frames observed for `MB -> JARGB_V2_1` were HID class
`SET_REPORT` transfers with:

- `wValue = 0x0350`
- `ReportID = 0x50`
- `ReportType = Feature`
- `wLength = 290`
- data starts with `0x50`
- observed frames include `4781` and `7757`

No `0x0390` / 302-byte report was observed for this MSI Center UI action.

Observed report starts:

| Frame | Report start | Notes |
| --- | --- | --- |
| `4781` | `50 02 14 ff 09 00 ff ...` | Feature report ID `0x50`, 290-byte transfer. |
| `7757` | `50 03 ff 00 00 ff 64 ...` | Feature report ID `0x50`, 290-byte transfer. |

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

## Key Conclusion

The only live MSI Center write path observed so far is `0x50`/290.

For the real MS-7E75 / B850 GAMING PLUS WIFI PZ board, selecting
`MB -> JARGB_V2_1` in MSI Center and applying changes produced HID Feature
`SET_REPORT` traffic using report ID `0x50` with a 290-byte payload. It did not
produce an observed report ID `0x90` with a 302-byte payload.

Do not use `JARGB_V2_1 -> 0x90` as a first-write plan. `0x90..0x93` remain
static/decompiled evidence only until live traffic confirms them.

Required wording:

- Do not use JARGB_V2_1 -> 0x90 as a first-write plan.
- 0x90..0x93 remain static/decompiled evidence only until live traffic confirms
  them.
- The only live MSI Center write path observed so far is 0x50/290.
- This does not approve Linux writes.

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

## Phase 4 Status

Phase 4 remains on hold.

This capture does not approve Linux writes, HID device opens, `SetFeature`,
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
