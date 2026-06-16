# MSI MS-7E75 MB800 HID Static Reverse Engineering Notes

## Scope

This document records a static-only deep pass on the HID layer used by the MS-7E75 MB800 zone path:

```text
MS-7E75 profile data
  -> EnumChipest.NUC126_MB800
  -> MSI_LED.Class_MB_800
  -> MSI_LED.MSI_800sLed
  -> MsiHid.HID_Basic
  -> Lib\MsiHid.dll
```

The focus is HID discovery/open evidence, `SetFeature` evidence, Gen1/Gen2 report buffer layout, zone-to-report mapping, and Linux implications. This is still static analysis only. It does not enable Linux support and does not prove that writes are safe.

Later passive USBPcap evidence is recorded in
[MSI_7E75_USBPCAP_CAPTURE_NOTES.md](MSI_7E75_USBPCAP_CAPTURE_NOTES.md).
That live MSI Center capture for `MB -> JARGB_V2_1` observed `0x50`/290
Feature `SET_REPORT` traffic, not `0x90`/302. Do not use
`JARGB_V2_1 -> 0x90` as a first-write plan. `0x90..0x93` remain
static/decompiled evidence only until live traffic confirms them. The only live
MSI Center write path observed so far is `0x50`/290. This does not approve
Linux writes.

Phase 0 reference implementation: [`src/linux/hid/report.rs`](../src/linux/hid/report.rs)

## Safety Constraints

- Static analysis and documentation only.
- Do not run MSI Center, Mystic Light, `LEDKeeper2.exe`, or MSI binaries.
- Do not run doctor.
- Do not run detect-chip, read-reg, write, or apply.
- Do not touch `/dev/port`.
- Do not perform raw SMBus or Super I/O access.
- Do not open HID devices.
- Do not enable MS-7E75 hardware access.
- Do not claim Linux support is ready unless static evidence is complete.

## Inputs And Evidence Used

| Input | SHA-256 | Static evidence used |
| --- | --- | --- |
| `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\LEDKeeper2.exe` | `990C65F31038AA6DCA39ABBE33735E42424B37696FB56D5B58D6EEA05FBB8159` | Decompiled `MSI_LED.Class_MB_800`, `MSI_LED.MSI_800sLed`, and `MsiHid.HID_Basic`. |
| `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib\MsiHid.dll` | `27AE0D6D9BF86FDE47E6309557D7DB8EF9E010538FC793FAA2F293A3982C779C` | Static binary string/import scan and native disassembly for exported wrapper names, Windows HID APIs, device filtering, and direct `HidD_SetFeature` behavior. See [MSI_7E75_MSIHID_STATIC_RE.md](MSI_7E75_MSIHID_STATIC_RE.md). |
| `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib\MsiHid_GameSync.dll` | `27AE0D6D9BF86FDE47E6309557D7DB8EF9E010538FC793FAA2F293A3982C779C` | Same hash as `MsiHid.dll`; used as corroborating wrapper evidence from `MysticLight_AllDevice.dll`. |
| `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\MysticLight_AllDevice.dll` | `AC35239A4C29DD41736A2162412D525ADA651A2ED5A16CCC9BD50B0C18D0C780` | Parallel `MysticLight_AllDevice.Device.MB_800.MSI_800sLed` and `MysticLight_AllDevice.HID_Basic` evidence. |
| Prior decoded profile data | See [MSI_7E75_PROFILE_DATA_STATIC_RE.md](MSI_7E75_PROFILE_DATA_STATIC_RE.md). | Confirms `MS-7E75_1` zones and `EnumChipest.NUC126_MB800` routing. |

## HID Discovery / Open Evidence

The primary LEDKeeper decompile has `MSI_LED.MSI_800sLed` with `using MsiHid;`, so unqualified `HID_Basic` calls bind to `MsiHid.HID_Basic` from the LEDKeeper assembly. That managed wrapper imports native functions from `Lib\MsiHid.dll`.

| Evidence | Static details | MS-7E75 relevance |
| --- | --- | --- |
| `MSI_LED.MSI_800sLed` constants | `VID_MSI_Common = 3504` (`0x0DB0`), `PID_MSI_Common = 118` (`0x0076`), general report length `64`, Gen1 length `290`, Gen1 advanced length `727`, Gen2 length `302`, Gen2 advanced length `761`. | MB800 path opens MSI common HID VID/PID, not a board-specific PID. |
| `CheckConnectedDevice(out pid)` | Calls `HID_Basic.openMyDevice_Read(3504, 118, 0, 0, 1)`, reads a 64-byte serial string, parses the first four characters as hex `pid`, then checks firmware with `Get_FwVersion`. | Explains how an MS-7E75-like board ID can be derived from the HID serial prefix if the common device is present. This was not run. |
| `Init(ushort pid)` | Calls `openMyDevice_Read(3504, 118, 0, 0, 1)`, reads serial, parses first four hex characters, and succeeds only if that value equals the requested `pid`. | Static gate that can select board ID `0x7E75` from common HID device serial prefix. |
| `CheckHandle()` | Calls `GetAttributes` and requires VID `3504`, PID `118`. | Confirms the retained handle is checked against MSI common HID VID/PID. |
| `HID_Basic.openMyDevice_Read` signature | Managed P/Invoke: `openMyDevice_Read(ushort VID, ushort PID, ushort MI, ushort COL, ushort deviceNum = 1)`. | LEDKeeper passes `MI=0`, `COL=0`, `deviceNum=1`; exact native interpretation of MI/COL remains in `MsiHid.dll`. |
| Native `MsiHid.dll` strings/imports | Exposes `openMyDevice`, `openMyDevice_Read`, `openMyDevice_Overlapped`, `openMyDeviceByStringID_Read`, `GetAllDevicesID`, and imports SetupAPI/Config Manager HID enumeration helpers. Native disassembly shows path tokens `VID_%04X`, `PID_%04X`, `MI_%02X`, and `Col%02X`. | Strong evidence device discovery/open is inside the native HID wrapper and uses VID/PID/MI/COL path filtering. |
| Usage page / usage | No managed usage-page or usage constants were found for the MB800 path. The native pass did not recover a usage-page or usage predicate beyond HID interface path filtering and `HidD_GetAttributes`. | Usage filtering remains unknown. Do not assume a Linux usage-page selector yet. |

## `SetFeature` Evidence

`MsiHid.HID_Basic` is a thin P/Invoke class:

| Managed method | Native DLL | Static native evidence |
| --- | --- | --- |
| `SetFeature(IntPtr DevHandle, byte[] Data, ulong length)` | `Lib\MsiHid.dll` | Native wrapper guards `INVALID_HANDLE_VALUE`, then directly calls `HidD_SetFeature(handle, Data, length)`; no report-ID rewrite was found. |
| `GetFeature(IntPtr DevHandle, byte[] Data, ulong length)` | `Lib\MsiHid.dll` | Native wrapper guards `INVALID_HANDLE_VALUE`, then directly calls `HidD_GetFeature(handle, Data, length)`. |
| `ReadDeviceInput` / `WriteDeviceOutput` | `Lib\MsiHid.dll` | Native DLL imports `ReadFile`, `WriteFile`, and HID report helpers. |
| `GetInputReport` / `SetOutputReport` | `Lib\MsiHid.dll` | Native DLL imports `HidD_GetInputReport` and `HidD_SetOutputReport`. |
| `GetAttributes` | `Lib\MsiHid.dll` | Native DLL imports `HidD_GetAttributes`. |
| `GetManufacturerString`, `GetProductString`, `GetSerialNumberString` | `Lib\MsiHid.dll` | Native DLL imports matching `HidD_*String` APIs. |
| `openMyDevice*` | `Lib\MsiHid.dll` | Native DLL imports `HidD_GetHidGuid`, `SetupDiGetClassDevsW`, `SetupDiEnumDeviceInterfaces`, `SetupDiGetDeviceInterfaceDetailW`, `CM_Get_Device_Interface_List*`, and `CreateFileW`. |

This confirms the MB800 helper path uses Windows HID APIs through MSI's native wrapper. It does not reveal all native enumeration predicates or prove a safe Linux write path.

## Report / Buffer Layout Table

### General 64-byte output/input commands

| Method | Direction | Length | Byte layout | Notes |
| --- | --- | --- | --- | --- |
| `Get_FwVersion` | `WriteDeviceOutput`, then `ReadDeviceInput` | 64 | Request: `[0]=0x01`, `[1]=0xB0`; response accepted when `[1]=0x5A`; reads major from `[2]`, extend from `[3]`. | Used by `CheckConnectedDevice`. |
| `Get_GlobalSwitch` | output/input | 64 | Request `[0]=0x01`, `[1]=0xBA`; ACK `[1]=0x5A`; switch at `[6]`. | Helper command, not zone apply. |
| `Set_GlobalSwitch` | output/input | 64 | Request `[0]=0x01`, `[1]=0xBB`, `[6]=0/1`; ACK `[1]=0x5A`, `[6]` echoes requested value. | Helper command, not zone apply. |
| `Gen2_Detect` | output/input | 64 | Request `[0]=0x01`, `[1]=0x82`, `[6]=port`; ACK `[0]=0x01`, `[1]=0x5A`, `[6]=port`; strip count `[7]`, error `[8]`. | Detection only; not executed. |
| `Gen2_SetEnableGen2` | output/input | 64 | Request `[0]=0x01`, `[1]=0x84`, `[6]=port`, `[7]=enable`; ACK `[0]=0x01`, `[1]=0x5A`, `[6]=port`. | Port mode helper; not executed. |
| `Set_Volume` | output only | 64 | Request `[0]=0x01`, `[1]=0xC0`, `[3]=main`, `[4]=left`, `[5]=right`. | Music-style helper. |
| `ResetMCU` | output only | 64 | Request `[0]=0x01`, `[1]=0xD0`; then closes device. | Not part of MS-7E75 support and must not be run. |

### Gen1 apply report

| Field | Offset / formula | Meaning |
| --- | --- | --- |
| Report selector | `[0] = 0x50` | Gen1 board apply / get selector. |
| Area count | 18 records | `Gen1_TotalAreaCount = 18`. |
| Per-area record base | `base = i * 16` for area index `i` | Record bytes occupy `[base+1]..[base+16]`. |
| Lighting mode | `[base+1]` | `Enum_LightingMode`: Off `0`, Wave `1`, Steady `2`, Flame `3`, Breathing `4`, ColorRing `5`, Lightning `6`, Recreation `7`, Meteor `8`, Advanced `9`, GodLike `10`. |
| Color 1 | `[base+2..4]` | RGB order. |
| Color 2 | `[base+5..7]` | RGB order. |
| Color 3 | `[base+8..10]` | RGB order. |
| Color 4 | `[base+11..13]` | RGB order. |
| Option1 | `[base+14]` | Color count: `0..3` for 1..4 colors. |
| Option2 | `[base+15]` | Packed flags: bit 7 select-all, bit 6 direction, bit 5 color selection, bits 2..4 brightness, bits 0..1 speed. |
| Cycle number | `[base+16]` | Per-area cycle count. |
| Store flag | `[289]` | `1` when persistent store requested, otherwise `0`. |
| Transport call | `HID_Basic.SetFeature(_Device, array, 290)` | Native wrapper statically imports `HidD_SetFeature`. |

### Gen2 port apply report

| Field | Offset / formula | Meaning |
| --- | --- | --- |
| Report selector | `[0] = 0x90 + port` | Port `0..3` gives `0x90`, `0x91`, `0x92`, `0x93`. |
| Initial fill | all bytes set to `0xFF` before writing records | Unused strip slots remain `0xFF`. |
| Max strip records | 6 | `Gen2_PortMaxStripCount = 6`. |
| Per-strip record base | `base = j * 20` for strip index `j` | Record bytes occupy `[base+1]..[base+20]`. |
| Fixed ID | `[base+1..4]` | Little-endian `uint Fixed_ID`. |
| Lighting mode | `[base+5]` | Same `Enum_LightingMode` values as Gen1. |
| Color 1 | `[base+6..8]` | RGB order. |
| Color 2 | `[base+9..11]` | RGB order. |
| Color 3 | `[base+12..14]` | RGB order. |
| Color 4 | `[base+15..17]` | RGB order. |
| Option_1 | `[base+18]` | Color count: `0..3` for 1..4 colors. |
| Option_2 | `[base+19]` | Packed flags: bit 7 select-all, bit 6 direction, bit 5 color selection, bits 2..4 brightness, bits 0..1 speed. |
| LED count | `[base+20]` | Clamped to max `180` when read back. |
| Store flag | `[301]` | `1` when persistent store requested, otherwise `0`. |
| Transport call | `HID_Basic.SetFeature(_Device, array, 302)` | Native wrapper statically imports `HidD_SetFeature`. |

### Advanced reports

| Method | Length | Selector | Layout summary | MS-7E75 relevance |
| --- | --- | --- | --- | --- |
| `Gen1_Adv_ApplyArea` / `_Byte` | 727 | `[0]=0x51`, `[1]=0x09` | Area high/low at `[2..3]`, color count at `[6]`, RGB stream starts at `[7]`. | Static advanced Gen1 path; not proven for default MS-7E75 zones. |
| `Gen2_Adv_ApplyPort` | 761 | `[0]=0xB0 + port` | Repeated variable-size records: fixed ID LE, LED count, RGB stream. | Advanced JARGB V2 path exists; default zone mapping uses basic Gen2 apply unless advanced profile state selects otherwise. |

## Zone-To-Report Mapping

| MS-7E75 zone | Static path | Report selector / area or port | Evidence status |
| --- | --- | --- | --- |
| `MS-7E75_1_JRGB1` | `Class_MB_800.SetStyle` -> `MSI_800sLed.Gen1_SetArea(Enum_LedArea.JRGB1)` -> `Gen1_ApplyBoard` | Gen1 report `0x50`; area index `9`. | Confirmed static call-path and report layout. |
| `MS-7E75_1_JARGB_V2_1` | `UpdateJARGB_V2_Basic(0)` -> `Gen2_SetStrip` -> `Gen2_ApplyPort(Enum_TargetPort.JARGB1)` | Gen2 report `0x90`; port index `0`. | Confirmed static call-path and report layout. |
| `MS-7E75_1_JARGB_V2_2` | `UpdateJARGB_V2_Basic(1)` -> `Gen2_SetStrip` -> `Gen2_ApplyPort(Enum_TargetPort.JARGB2)` | Gen2 report `0x91`; port index `1`. | Confirmed static call-path and report layout. |
| `MS-7E75_1_JARGB_V2_3` | `UpdateJARGB_V2_Basic(2)` -> `Gen2_SetStrip` -> `Gen2_ApplyPort(Enum_TargetPort.JARGB3)` | Gen2 report `0x92`; port index `2`. | Confirmed static call-path and report layout. |
| `MS-7E75_1_EZ Conn` | `UpdateJARGB_V2_Basic(3)` -> `Gen2_SetStrip` -> `Gen2_ApplyPort(Enum_TargetPort.JAF)` | Gen2 report `0x93`; port index `3`. | Confirmed static call-path and report layout. |
| `MS-7E75_1_SELECT ALL` | `Class_MB_800.SetStyle` select-all handling | Aggregate Gen1 and/or per-port Gen2 state depending on profile state. | Confirmed as logical aggregate; no unique report selector. |

Important live-capture qualification: the table above is static/decompiled
evidence. A later passive USBPcap capture of MSI Center applying
`MB -> JARGB_V2_1` observed Feature `SET_REPORT` `0x50` with length `290`, and
did not observe `0x0390` / 302-byte traffic for that UI action.

Offline builder comparison support in
[`src/linux/hid/capture_compare.rs`](../src/linux/hid/capture_compare.rs)
parses pasted capture hex fixtures only. Current tests confirm that the USB
setup length and payload report ID match the Gen1 builder's `0x50`/290 shape,
while the available frame prefixes differ in area-0 mode/color bytes from the
current single-zone `JRGB1` builder fixture. The comparison intentionally does
not force the builder to match the capture because the full payload and byte
meanings are not yet proven.

## Confirmed Vs Unknown

Confirmed:

- LEDKeeper's primary MB800 path uses `MSI_LED.MSI_800sLed`, not only the parallel `MysticLight_AllDevice.dll` helper.
- `MSI_LED.MSI_800sLed` uses `MsiHid.HID_Basic` from `LEDKeeper2.exe`.
- `MsiHid.HID_Basic` P/Invokes `Lib\MsiHid.dll`.
- `Lib\MsiHid.dll` and `Lib\MsiHid_GameSync.dll` have the same SHA-256 in this install.
- Native HID wrapper strings/imports include SetupAPI/Config Manager enumeration, `CreateFileW`, `ReadFile`, `WriteFile`, and `HidD_*` APIs including `HidD_SetFeature`.
- Native `MsiHid.dll` filtering uses device-path tokens for `VID_%04X`, `PID_%04X`, `MI_%02X`, and `Col%02X`; `SetFeature` passes caller buffers directly to `HidD_SetFeature`, including report ID byte `Data[0]`.
- MB800 common device open uses VID `0x0DB0`, PID `0x0076`, MI `0`, COL `0`, device number `1`.
- `CheckConnectedDevice` and `Init` derive/validate a board-like PID from the first four hex characters of the HID serial string.
- Gen1 and Gen2 feature report lengths and byte layouts are statically visible.

Unknown:

- The exact optimized branch semantics of native post-open `HidD_GetAttributes` checks.
- HID usage page and usage; the native pass did not recover a usage predicate.
- The actual HID device path on this host; no devices were enumerated or opened.
- Whether Linux `hidraw` will expose the same report lengths and behavior without MSI's native wrapper.
- Whether the MS-7E75 controller accepts all documented report forms safely.
- Whether additional initialization, locking, or service coordination is required before any future non-Windows implementation.
- Whether MBAPI's separate static `7E75` board-list hit is involved before, beside, or independently of this MB800 HID path.

## Linux Implications

This pass improves the static Linux research picture, but it does not make Linux support ready.

Static implications:

- The visible lower transport for the MB800 zone path is HID feature reports, not raw SMBus, EC space, Super I/O GPIO, or `/dev/port`.
- A future Linux prototype would likely need HID enumeration equivalent to the native wrapper's VID/PID/MI/COL selection and serial-prefix validation before any report access is considered.
- `hidraw` could be a candidate interface only if a later safe inventory pass confirms the matching USB HID node and report sizes. That inventory has not been performed here.
- The 290-byte and 302-byte `SetFeature` payloads are larger than the common 64-byte output reports; byte `0` is statically confirmed as the feature report ID at the Windows HID API boundary, but Linux report-size behavior must still be verified safely.
- No Linux write/apply implementation should be added until native device matching, descriptor/report-size behavior, and failure modes are documented and reviewed.
- A separate Linux HID implementation plan is documented in [MSI_7E75_LINUX_HID_IMPLEMENTATION_PLAN.md](MSI_7E75_LINUX_HID_IMPLEMENTATION_PLAN.md); it is documentation-only and does not permit device access.
- A pure in-memory Phase 0 report builder now exists in [`src/linux/hid/report.rs`](../src/linux/hid/report.rs); it assembles documented buffers only and does not enumerate or open HID devices.
- A Phase 1 read-only HID inventory now exists in [`src/linux/hid/inventory.rs`](../src/linux/hid/inventory.rs); it parses metadata only, does not open HID devices, and does not enable writes.
- A Phase 2 read-only board gate now exists in [`src/linux/hid/gate.rs`](../src/linux/hid/gate.rs); it combines DMI plus inventory metadata only and still enables no HID writes.
- A Phase 3 dry-run report preview now exists in [`src/linux/hid/dry_run.rs`](../src/linux/hid/dry_run.rs); it builds MB800 reports in memory only, prints report metadata/hex preview, and still performs no HID writes.

## Implementation Audit

Static audit date: 2026-06-16.

Audited implementation: [`src/linux/hid/report.rs`](../src/linux/hid/report.rs).

Result: the current in-memory Rust report builder matches the statically decompiled `MSI_LED.MSI_800sLed` basic Gen1 and Gen2 byte layout. No device access was added or used during this audit.

Confirmed matches:

- Gen1 `JRGB1` uses report ID `0x50`, report length `290`, area index `9`, and 16-byte area records.
- Gen1 record bytes match `Gen1_ApplyBoard`: mode at `[base+1]`, RGB colors at `[base+2..13]`, option/color-count byte at `[base+14]`, packed option byte at `[base+15]`, cycle byte at `[base+16]`, and store byte at `[289]`.
- Gen2 `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, and `EZ Conn` use report IDs `0x90`, `0x91`, `0x92`, and `0x93`, matching port indices `0`, `1`, `2`, and `3`.
- Gen2 reports use length `302`, an initial fill of `0xFF`, 20-byte strip records, fixed ID little-endian at `[base+1..4]`, mode at `[base+5]`, RGB colors at `[base+6..17]`, option/color-count byte at `[base+18]`, packed option byte at `[base+19]`, LED count at `[base+20]`, and store byte at `[301]`.
- The builder rejects Gen1/Gen2 zone mismatches instead of silently building a report for the wrong report family.

Tests hardened in this audit:

- Exact Gen2 static red, green, blue, and `EZ Conn` byte-offset tests now assert report ID, report length, fixed ID, mode, RGB bytes, option bytes, LED count, unused `0xFF` bytes, and store byte.
- Existing Gen1 tests continue to assert the `JRGB1` area-9 offsets and store byte.
- Zone/report mismatch tests now assert the exact mismatch error variant.

Unknowns that remain unchanged:

- Linux `hidraw` dispatch behavior, report descriptor behavior, and whether a future Linux transport must include or omit the report ID byte remain untested and unknown.
- The current builder is intentionally in-memory only. It does not open HID devices, call `SetFeature`/`GetFeature`, or touch `/dev/hidraw*`.
- Phase 4 remains on hold; this audit does not approve writes or write-once behavior.

## Real-Machine Validation Result

The read-only and dry-run phases were validated on a real MSI MS-7E75 / B850 GAMING PLUS WIFI PZ board and passed without device opens or writes.

- Phase 1 inventory passed.
- Phase 2 gate reached `eligible_for_dry_run`.
- Phase 3 dry-run passed for `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, and `EZ Conn`.
- No device opens were reported.
- No writes were performed.
- Phase 4 is still not implemented and not approved.
- The next step is a separate reviewed Phase 4 write design, not immediate write code.

## Next Static-Only Targets

- Reconstruct the full optimized native control flow of `Lib\MsiHid.dll` `openMyDevice*`, especially post-open `HidD_GetAttributes` comparison semantics.
- Search for any remaining native usage-page/usage checks or descriptor queries not recovered in the first native pass.
- Statically map `openMyDeviceByStringID_Read` and `GetAllDevicesID` output format without calling either function.
- Cross-reference `Class_MB_800.Initial`, `RGBControlClass.updateSupportedDevice`, and MB800 startup support-list construction to decide whether MBAPI's `7E75` list gates this HID path.
- Keep MS-7E75 Linux support blocked until static evidence and a separately reviewed read-only HID inventory plan exist.

## Explicit Hardware-Access Note

No MSI binaries were executed during this pass. MSI Center, Mystic Light, and `LEDKeeper2.exe` were not launched. No HID devices were opened or enumerated through MSI code. No hardware access was enabled or run: no doctor command, no chip detection, no register read, no write/apply command, no `/dev/port`, no raw SMBus, no raw Super I/O, and no MS-7E75 hardware support code changes.
