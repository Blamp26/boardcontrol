# MSI MS-7E75 MsiHid Native Static Reverse Engineering Notes

## Scope

This document records a static-only native pass on MSI's HID wrapper used below the MS-7E75 MB800 lighting path:

```text
MS-7E75 profile data
  -> EnumChipest.NUC126_MB800
  -> MSI_LED.Class_MB_800
  -> MSI_LED.MSI_800sLed
  -> MsiHid.HID_Basic
  -> Lib\MsiHid.dll
  -> HidD_SetFeature
```

The goal is to document native device discovery, VID/PID/MI/COL filtering, open flags, HID feature wrappers, report-buffer handling, and Linux implications. This pass does not execute MSI software and does not open HID devices.

## Safety Constraints

- Static analysis and documentation only.
- Do not run MSI Center, Mystic Light, `LEDKeeper2.exe`, or MSI binaries.
- Do not open HID devices.
- Do not run doctor.
- Do not run detect-chip, read-reg, write, or apply.
- Do not touch `/dev/port`.
- Do not perform raw SMBus or Super I/O access.
- Do not enable MS-7E75 hardware access.
- Do not claim Linux support is ready unless static evidence is complete.

## Analyzed Binary And Hash

| File | SHA-256 | Size | Timestamp | Static method |
| --- | --- | --- | --- | --- |
| `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib\MsiHid.dll` | `27AE0D6D9BF86FDE47E6309557D7DB8EF9E010538FC793FAA2F293A3982C779C` | 150,600 bytes | 2025-09-26 15:01:42 | PE import/export/resource/string inspection with `pefile`; x86 disassembly with Capstone. |
| `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib\MsiHid_GameSync.dll` | `27AE0D6D9BF86FDE47E6309557D7DB8EF9E010538FC793FAA2F293A3982C779C` | 150,600 bytes | 2025-09-26 15:01:42 | Hash comparison only; same bytes as `MsiHid.dll` in this install. |

PE metadata:

| Property | Value |
| --- | --- |
| Machine | x86 / `0x14c` |
| Image base | `0x10000000` |
| Entry RVA | `0x518b` |
| PDB path string | `E:\bob\workspace\MsiHid\Release\MsiHid.pdb` |

## Native HID Evidence Table

| Evidence | Static details | MS-7E75 relevance |
| --- | --- | --- |
| Exported open helpers | `openMyDevice`, `openMyDevice_Read`, `openMyDevice_Overlapped`, `openMyDeviceByStringID_Read`, and `openMyDeviceByStringID_Overlapped` are exported. | Managed `MsiHid.HID_Basic.openMyDevice_Read(3504, 118, 0, 0, 1)` binds to this native layer. |
| Exported feature helpers | `SetFeature` and `GetFeature` are exported. | Managed `MSI_800sLed` feature-report apply calls bind here. |
| Exported report/string helpers | `GetInputReport`, `SetOutputReport`, `ReadDeviceInput`, `WriteDeviceOutput`, `GetManufacturerString`, `GetProductString`, and `GetSerialNumberString` are exported. | Matches the MB800 managed helper's firmware, switch, serial, and apply support calls. |
| HID imports | Imports `HidD_SetFeature`, `HidD_GetFeature`, `HidD_GetInputReport`, `HidD_SetOutputReport`, `HidD_GetAttributes`, `HidD_GetSerialNumberString`, `HidD_GetManufacturerString`, `HidD_GetProductString`, `HidD_FlushQueue`, and `HidD_GetHidGuid`. | Confirms Windows HID API transport below the managed MB800 helper. |
| SetupAPI / Config Manager imports | Imports `SetupDiGetClassDevsW`, `SetupDiEnumDeviceInterfaces`, `SetupDiGetDeviceInterfaceDetailW`, `SetupDiDestroyDeviceInfoList`, `CM_Get_Device_Interface_List_SizeW`, and `CM_Get_Device_Interface_ListW`. | Confirms native wrapper owns HID interface enumeration and path selection. |
| Kernel32 imports | Imports `CreateFileW`, `CloseHandle`, `ReadFile`, `WriteFile`, `CancelIo`, `CreateEventW`, `WaitForSingleObject`, `ResetEvent`, `Sleep`, and `GetLastError`. | Confirms native wrapper opens HID device paths and implements direct and overlapped I/O helpers. |
| Device-path filter strings | UTF-16 strings include `VID_%04X`, `PID_%04X`, `MI_%02X`, and `Col%02X`. | Matches known MS-7E75 MB800 target parameters: VID `0x0DB0`, PID `0x0076`, MI `0`, COL `0`, device `1`. |
| Registry helper strings | Strings include `SOFTWARE\Wow6432Node\MSI\MsiHid` and `DevicePaths`. | Evidence for exported `WriteAllDevicesToRegistry`; not needed by the MS-7E75 apply path. |

## Device Discovery / Filtering Evidence

Native enumeration starts by calling `HidD_GetHidGuid`, then obtains a HID interface multi-string through Config Manager list-size/list APIs. The wrapper stores or reuses a global device-path list before filtering candidates.

The internal open path used by `openMyDevice*` builds formatted filter tokens from caller arguments:

| Caller argument | Format token | Known MS-7E75 MB800 value |
| --- | --- | --- |
| VID | `VID_%04X` | `VID_0DB0` |
| PID | `PID_%04X` | `PID_0076` |
| MI | `MI_%02X` | `MI_00` |
| COL | `Col%02X` | `Col00` |
| device number | match counter | `1` |

Static disassembly shows candidate paths are filtered with case-insensitive substring checks for VID and PID tokens, then additional MI/COL token checks with delimiter sanity around `&`-separated device-path components. The wrapper counts matching paths and returns the requested `deviceNum`th match; for LEDKeeper's `deviceNum=1`, this means the first matching path.

On a selected candidate, the wrapper calls:

```text
CreateFileW(path, desiredAccess, 3, NULL, 3, flags, NULL)
```

where share mode `3` is read/write sharing and creation disposition `3` is `OPEN_EXISTING`.

Observed public wrapper access/flag choices:

| Export | Desired access | Flags | Static meaning |
| --- | --- | --- | --- |
| `openMyDevice` | `0x40000000` | `0` | Generic write open. |
| `openMyDevice_Read` | `0xC0000000` | `0` | Generic read/write open. This is the managed MB800 path used by `MSI_800sLed`. |
| `openMyDevice_Overlapped` | `0xC0000000` | `0x40000080` | Generic read/write open with overlapped/normal file flags. |
| `openMyDeviceByStringID_Read` | `0xC0000000` | `0` | Direct open by supplied device path string. |
| `openMyDeviceByStringID_Overlapped` | `0xC0000000` | `0x40000080` | Direct overlapped open by supplied device path string. |

After opening, the wrapper calls `HidD_GetAttributes`. Static disassembly shows comparisons against requested VID/PID values after path filtering. The exact boolean semantics of the attribute checks should be verified in a deeper native control-flow pass because the path-token filter already enforces VID/PID and the optimized branch structure is not yet fully reconstructed.

No HID usage-page or usage descriptor filtering was recovered in this pass. The visible selector is HID interface path filtering by VID/PID/MI/COL plus `HidD_GetAttributes`.

## SetFeature / GetFeature Evidence

The public feature wrappers are thin direct calls:

| Export | Static behavior |
| --- | --- |
| `GetFeature(handle, data, length)` | Returns `0` if `handle == INVALID_HANDLE_VALUE`; otherwise calls `HidD_GetFeature(handle, data, length)` and returns the API Boolean as `0/1`. |
| `SetFeature(handle, data, length)` | Returns `0` if `handle == INVALID_HANDLE_VALUE`; otherwise calls `HidD_SetFeature(handle, data, length)` and returns the API Boolean as `0/1`. |

No static evidence was found in `SetFeature` for rewriting the buffer, stripping the first byte, adding a report ID, retrying, or changing the length. Therefore, for the MB800 path, the managed caller's `data[0]` is passed to `HidD_SetFeature` as the HID feature report ID byte.

This confirms the previously documented report selectors remain in byte 0 at the native HID API boundary:

| MS-7E75 zone | Managed report | Native call |
| --- | --- | --- |
| `MS-7E75_1_JRGB1` | `data[0] = 0x50`, length `290` | `HidD_SetFeature(handle, data, 290)` |
| `MS-7E75_1_JARGB_V2_1` | `data[0] = 0x90`, length `302` | `HidD_SetFeature(handle, data, 302)` |
| `MS-7E75_1_JARGB_V2_2` | `data[0] = 0x91`, length `302` | `HidD_SetFeature(handle, data, 302)` |
| `MS-7E75_1_JARGB_V2_3` | `data[0] = 0x92`, length `302` | `HidD_SetFeature(handle, data, 302)` |
| `MS-7E75_1_EZ Conn` | `data[0] = 0x93`, length `302` | `HidD_SetFeature(handle, data, 302)` |

## ReadFile / WriteFile / Close Evidence

| Export | Static behavior |
| --- | --- |
| `ReadDeviceInput` | Guards `INVALID_HANDLE_VALUE`, then calls `ReadFile(handle, data, length, &bytesRead, NULL)`. |
| `WriteDeviceOutput` | Guards `INVALID_HANDLE_VALUE`, then calls `WriteFile(handle, data, length, &bytesWritten, NULL)`. |
| `ReadDeviceInput_Overlapped` | Creates a manual-reset event, calls `ReadFile` with an `OVERLAPPED`, waits up to `100` ms, and on timeout/error calls `CancelIo` and `HidD_FlushQueue`. |
| `WriteDeviceOutput_Overlapped` | Creates a manual-reset event, calls `WriteFile` with an `OVERLAPPED`, waits up to `100` ms, and on timeout/error calls `CancelIo` and `HidD_FlushQueue`. |
| `CloseDevice` | Calls `CancelIo`, `HidD_FlushQueue`, `CloseHandle`, then `Sleep(20)` when the handle is valid. |

The managed MS-7E75 MB800 apply path uses `SetFeature` for the 290-byte and 302-byte zone reports. The 64-byte `ReadFile`/`WriteFile` helpers are still relevant to firmware version, global switch, detect, and other helper commands documented in the MB800 HID notes.

## MS-7E75 MB800 Path Relevance

This native pass strengthens the static chain from decoded MS-7E75 profile zones to Windows HID feature reports:

- `MSI_LED.MSI_800sLed` opens VID `0x0DB0`, PID `0x0076`, MI `0`, COL `0`, device `1`.
- `MsiHid.dll` implements those parameters with HID interface-path filtering tokens.
- `MSI_LED.MSI_800sLed` validates the first four HID serial characters as a hex board ID before accepting a requested `pid`.
- `MsiHid.dll` passes feature report buffers directly into `HidD_SetFeature`.
- The report ID byte is included in `data[0]` at the native API boundary.

This is not a raw register map. It is static evidence for an MSI HID feature-report backend used by the MB800 path.

## Linux Hidraw Implications

Static implications:

- A future Linux implementation would likely need to find the same HID interface selected by MSI's VID/PID/MI/COL path filtering and then validate the HID serial prefix before any report access.
- Linux `hidraw` is a plausible transport candidate only because the Windows path terminates in `HidD_SetFeature`; it is not yet a supported backend.
- Any future `hidraw` prototype would need to include the report ID byte in byte 0 and pass full feature-report lengths of `290` or `302` bytes for the documented zone reports, subject to independent safe validation.
- The visible native wrapper does not show extra checksums, encryption, or buffer transformations around `HidD_SetFeature`; the managed `MSI_800sLed` buffer layout appears to be the transport payload.

Blocked / unknown:

- No Linux HID node has been identified or opened.
- No feature report descriptor, report size, usage page, or usage has been collected from Linux.
- The exact native attribute-check branch semantics need another static pass before cloning the selector.
- No evidence proves that writing these reports from Linux is safe, complete, or sufficient.
- Linux support must remain disabled until a separate reviewed plan covers safe HID inventory, report descriptor inspection, and strict gating.

## Confirmed Vs Unknown

Confirmed:

- `MsiHid.dll` and `MsiHid_GameSync.dll` are byte-identical in this install.
- `MsiHid.dll` is a native x86 PE exporting MSI HID open, feature, input/output, string, and registry helper functions.
- The native wrapper imports Windows HID APIs and SetupAPI/Config Manager device-enumeration APIs.
- Device filtering uses formatted `VID_%04X`, `PID_%04X`, `MI_%02X`, and `Col%02X` path tokens.
- `openMyDevice_Read` opens matching HID paths with `GENERIC_READ | GENERIC_WRITE`, read/write sharing, and `OPEN_EXISTING`.
- `SetFeature` directly calls `HidD_SetFeature(handle, data, length)` after an invalid-handle guard.
- The MB800 report selector byte is included in `data[0]` when passed to `HidD_SetFeature`.
- No MSI binaries were executed and no HID devices were opened.

Unknown:

- Exact HID usage page / usage matching, if any, beyond path tokens and attributes.
- Exact optimized control-flow semantics of the post-open `HidD_GetAttributes` comparisons.
- Actual device path, descriptor, and report-size behavior on Linux.
- Whether all MS-7E75 zones require only the documented Gen1/Gen2 feature reports in live MSI Center operation.
- Whether MSI services, firmware state, or prior initialization affect HID report acceptance.

## Next Static-Only Targets

- Reconstruct the full native control flow of the internal `openMyDevice` implementation, especially the `HidD_GetAttributes` comparison branches.
- Recover or name internal helper functions used for case-insensitive path matching and multi-string iteration.
- Statically inspect all callers of `SetFeature`, `GetFeature`, `ReadDeviceInput`, and `WriteDeviceOutput` in managed LEDKeeper and `MysticLight_AllDevice.dll`.
- Cross-reference MB800 firmware/version gates against report layout changes.
- Continue MBAPI `7E75` board-list consumer analysis to determine whether MBAPI gates, duplicates, or bypasses the MB800 HID path.

## Explicit Hardware-Access Note

No MSI binaries were executed during this pass. MSI Center, Mystic Light, `LEDKeeper2.exe`, `MsiHid.dll`, and `MsiHid_GameSync.dll` were not launched or loaded as executable code. No HID devices were opened. No hardware access was enabled or run: no doctor command, no chip detection, no register read, no write/apply command, no `/dev/port`, no raw SMBus, no raw Super I/O, and no MS-7E75 hardware support code changes.
