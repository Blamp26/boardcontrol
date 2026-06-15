# MSI MS-7E75 Static Reverse Engineering Overview

Status: consolidated static analysis only, hardware access disabled.

## Scope

This document is the top-level static reverse-engineering overview for MSI MS-7E75 / B850 GAMING PLUS WIFI PZ research. It consolidates findings from the MBAPI boundary pass and direct static Ghidra/headless passes on:

- `MBAPI_x86.dll` boundary evidence
- `Driver_Engine.dll`
- `SMBus_Engine.dll`
- `rtk_bridge.dll`
- `CPU_Engine.dll`

The goal is to summarize what is confirmed, which modules look relevant, which modules look low-relevance for motherboard Mystic Light, what remains unknown, and why the current evidence still does not establish an MS-7E75 LED register map.

## Safety Constraints

- Documentation only.
- Static analysis only.
- MSI Center was not run.
- Mystic Light was not run.
- `cargo run -- doctor` was not run.
- `detect-chip`, `read-reg`, `write`, and `apply` were not run.
- `/dev/port` was not touched.
- No raw SMBus access was performed.
- No raw Super I/O access was performed.
- MS-7E75 hardware access was not enabled.
- No MS-7E75 register map was inferred from 7A45.
- The 7A45 NCT register map was not reused for MS-7E75.
- No analyzed path is claimed as the MS-7E75 path unless board-specific evidence exists.

## Related Detailed Notes

| Note | Purpose |
| --- | --- |
| [MSI_7E75_PROFILE_SELECTION_STATIC_RE.md](MSI_7E75_PROFILE_SELECTION_STATIC_RE.md) | Static search for MSI Center / Mystic Light board-profile, LED-zone, and route-selection evidence. |
| [MSI_7E75_LEDKEEPER_STATIC_RE.md](MSI_7E75_LEDKEEPER_STATIC_RE.md) | Direct `LEDKeeper2.exe` static metadata, strings, resources, MBAPI P/Invoke boundary, profile/zone evidence, and dispatch-candidate notes. |
| [MSI_7E75_PROFILE_DATA_STATIC_RE.md](MSI_7E75_PROFILE_DATA_STATIC_RE.md) | Static decode of `Mystic Light Online Data.dat` and inspection of profile data files containing MS-7E75 zone records. |
| [MSI_7E75_ZONE_CALLPATH_STATIC_RE.md](MSI_7E75_ZONE_CALLPATH_STATIC_RE.md) | Static path from decoded MS-7E75 zones through `CLEDParser`, `Class_MB_800`, and `MSI_800sLed` helper calls. |
| [MSI_7E75_HID_MB800_STATIC_RE.md](MSI_7E75_HID_MB800_STATIC_RE.md) | Static deep pass on the MB800 HID helper path, `MsiHid.dll` wrapper evidence, and Gen1/Gen2 feature-report layouts. |
| [MSI_7E75_MSIHID_STATIC_RE.md](MSI_7E75_MSIHID_STATIC_RE.md) | Static native pass on `Lib\MsiHid.dll` device filtering, open flags, direct `HidD_SetFeature` wrapper behavior, and Linux `hidraw` implications. |
| [MSI_7E75_DRIVER_ENGINE_STATIC_RE.md](MSI_7E75_DRIVER_ENGINE_STATIC_RE.md) | Direct `Driver_Engine.dll` transport, service, device, and IOCTL evidence. |
| [MSI_7E75_SMBUS_ENGINE_STATIC_RE.md](MSI_7E75_SMBUS_ENGINE_STATIC_RE.md) | Direct `SMBus_Engine.dll` SMBus transaction, controller-selection, and Renesas-adjacent evidence. |
| [MSI_7E75_RTK_BRIDGE_STATIC_RE.md](MSI_7E75_RTK_BRIDGE_STATIC_RE.md) | Direct `rtk_bridge.dll` Realtek bridge/device-handle evidence and relevance assessment. |
| [MSI_7E75_CPU_ENGINE_STATIC_RE.md](MSI_7E75_CPU_ENGINE_STATIC_RE.md) | Direct `CPU_Engine.dll` CPU telemetry/tuning evidence and relevance assessment. |

## Confirmed Static Evidence

- The MBAPI-like layer contains Mystic Light and LED entry points such as `LEDMysticControl`, `LEDMysticControlV2`, `LEDControl`, `SupportLED`, `SetMysticLEDColor*`, `SetMysticRainbowMode*`, `ResetLED`, and `ControlFANLED`.
- The MBAPI-like layer dynamically references companion modules including `\SMBus_Engine.dll`, `\Driver_Engine.dll`, `\CPU_Engine.dll`, and `\rtk_bridge.dll`.
- The MBAPI-like layer contains `NTIOLib_MysticLight` and resolves `Driver_Engine.dll` exports for port I/O, CMOS, MSR, physical memory, and PCI config access.
- The MBAPI-like layer contains `7E75` in a broad static board-ID list, but the direct consumer and dispatch effect of that entry are not yet mapped.
- `LEDKeeper2.exe` is a managed x86 .NET Framework 4.8 Mystic Light orchestration executable whose managed `MSI_LED.MB` class P/Invokes `Lib\MBAPI_x86.dll` for motherboard LED/support calls, Renesas helpers, EC/SIO helpers, and `SMBus_Initial`.
- `LEDKeeper2.exe` contains log templates matching existing runtime logs, including `Support list : `, `ResetItem : `, and `[RGBControlClass] mbID `, plus generic `JRGB1`, `JRGB2`, `JRAINBOW1`, `JRAINBOW2`, `JARGB_V2_1`, `JARGB_V2_2`, and `JARGB_V2_3` strings.
- `LEDKeeper2.exe` contains board support enums and candidate dispatch classes such as `RGBControlClass`, `Class_Fun_MB`, and `MSI_7B10Led`, but no cleartext `7E75`, `MS-7E75`, `MS-7E75_1`, or `B850` was found in that executable.
- `LEDKeeper2.exe` decompilation shows `MS-7E75_1` can be generated indirectly from runtime board identity as `MB_Info.Product + "_" + MB_Info.Version.Substring(0, 1)`, while `App.mbID` can be generated from the four hex digits after `MS-`.
- `LEDKeeper2.exe` decompilation shows `Class_ParseCfg.ParseCfgFile` selects and decodes `Mystic Light Online Data.dat`, extracts `[SyncData]`, and `Class_Fun_MB.Compare_Support_MB` checks `[SyncData]` against board product/version or market strings.
- Static decoding of both installed `Mystic Light Online Data.dat` copies confirms `[SyncData]` records for `MS-7E75_1` and `MS-7E75_2` with `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, `EZ Conn`, and `SELECT ALL`.
- A follow-up zone call-path pass resolves the MS-7E75 tuple chipset byte `69` to `EnumChipest.NUC126_MB800`, maps the decoded zones into `Class_MB_800`, and finds `MSI_800sLed` HID feature-buffer helper calls below that path.
- HID static analysis confirms the primary LEDKeeper MB800 helper path uses `MsiHid.HID_Basic` P/Invokes into `Lib\MsiHid.dll`; the native wrapper imports Windows HID APIs, filters paths with VID/PID/MI/COL tokens, and directly passes caller buffers to `HidD_SetFeature`.
- No direct static call path was found from these decoded MS-7E75 MB800 zones to `MSI_LED.MB` P/Invokes such as `RenesasLEDControlV3`, `LEDControl`, `LEDMysticControl`, `SMBusControl`, `SetECSpace`, or `SetSIOGPIO`.
- Installed runtime log artifacts show `MS-7E75_1`, `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, and `JARGB_V2_3` for this host, matching the decoded static profile data.
- The actual `Driver_Engine.dll` imports and uses `CreateFileW`, `DeviceIoControl`, and Service Control Manager APIs.
- The actual `Driver_Engine.dll` embeds `NTIOLib.sys` and `NTIOLib_X64.sys`, constructs `\\.\<caller-provided-name>` device paths, and delegates privileged operations to a kernel driver through visible `0xc350....` IOCTL constants.
- The actual `SMBus_Engine.dll` exports SMBus byte/block/check/SPD wrappers and initializes from a Driver Engine-like object supplied by its caller.
- `SMBus_Engine.dll` uses Driver Engine PCI config function-table offsets to select an `IntelSMBus` or `ATISMBus` backend and discover SMBus controller/base information.
- `SMBus_Engine.dll` contains Renesas-labeled synchronization/log strings, but no hard-coded MS-7E75, JRGB, JRAINBOW, or Renesas LED address proof was found in that DLL.
- `rtk_bridge.dll` is a Realtek bridge helper with storage-device scan paths, `DeviceIoControl` bridge commands, and bridge-attached LED helper exports.
- `CPU_Engine.dll` is a CPU telemetry/tuning helper with AMD SMU, Intel mailbox, temperature, ratio, power, and current-limit exports.
- No analyzed static source maps MS-7E75 motherboard headers to a raw SMBus address, EC/SIO register, command payload, or register map.
- The MB800 path provides static HID feature-buffer helper evidence for decoded MS-7E75 zones, but report buffers are not a register map and were not executed.

## Summary Table

| Module | Confirmed role | Relevance to MS-7E75 Mystic Light | Board-specific proof found? | Current assessment |
| --- | --- | --- | --- | --- |
| MBAPI-like boundary | Mystic Light API layer; loads companion engines; contains LED, SMBus, Driver Engine, EC/SIO, NCT, fan LED symbols, and a broad static board-ID list that includes `7E75`. | High as the orchestration layer, but the `7E75` entry is not yet tied to a concrete backend. | `7E75` board-list entry found; no decoded zone/transport/register mapping found. | Important starting point; likely chooses feature/backend paths through flags, code, or data still to be mapped. |
| `LEDKeeper2.exe` | Managed Mystic Light orchestration executable; wraps `Lib\MBAPI_x86.dll`, owns `RGBControlClass` log templates, generic board support enums, profile/online-data filenames, and JARGB V2 strings. | High for support/profile/zone dispatch research. | Generic `JARGB_V2_1/2/3`, `JRGB1`, `Support list : `, `ResetItem : `, and `[RGBControlClass] mbID ` strings found; no cleartext `7E75` or `MS-7E75_1` found. Decoded MS-7E75 data selects `EnumChipest.NUC126_MB800`, consumed by `Class_MB_800`. | Strong dispatch candidate. Decompilation shows it loads decoded online data where MS-7E75 records were confirmed and routes MB800 zones to `MSI_800sLed`. |
| `MSI_LED.MSI_800sLed` / `MsiHid.dll` | LEDKeeper MB800 helper and native HID wrapper; opens common MSI HID VID/PID `0x0DB0:0x0076`, validates serial prefix, constructs Gen1/Gen2 feature reports, filters HID paths by VID/PID/MI/COL tokens, and calls `HidD_SetFeature` through `Lib\MsiHid.dll`. | High for decoded MS-7E75 zones because profile field `69` resolves to `NUC126_MB800`. | Static helper path only; no MSI binary or HID device was opened. Native `SetFeature` preserves caller buffer byte `0` as the report ID at the HID API boundary. | Current strongest zone-to-HID-report path. Still not a raw register map or ready Linux support. |
| `MysticLight_AllDevice.dll` MB800 helper | Contains a parallel `MysticLight_AllDevice.Device.MB_800.MSI_800sLed` and `HID_Basic` wrapper using `Lib\MsiHid_GameSync.dll`, which has the same SHA-256 as `Lib\MsiHid.dll` in this install. | Corroborating evidence for MB800 HID feature-report layout. | Parallel helper, not the primary `Class_MB_800` binding in LEDKeeper. | Useful cross-check only. |
| `Driver_Engine.dll` | Generic privileged access bridge through `CreateFileW`, SCM APIs, `DeviceIoControl`, `NTIOLib.sys`, and `NTIOLib_X64.sys`. | High as the low-level transport provider used by MBAPI and SMBus Engine. | No board/header strings found. | Explains generic NTIOLib-backed port/PCI/memory/MSR access, not an LED map. |
| `SMBus_Engine.dll` | Generic SMBus byte/block/check/SPD transaction engine with Intel/ATI backend selection through Driver Engine PCI config calls. | High as the strongest generic candidate for the MBAPI Mystic Light/Renesas SMBus path. | No MS-7E75/header strings found. | Likely part of the main generic transport path, but not proof MS-7E75 uses it. |
| `rtk_bridge.dll` | Realtek USB/storage bridge helper with bridge scan/open, IOCTL command wrapper, and bridge LED helpers. | Low for motherboard headers unless a future board/accessory selector points to it. | No MS-7E75/header strings found. | Probably unrelated to MS-7E75 motherboard Mystic Light; likely accessory/bridge-device support. |
| `CPU_Engine.dll` | CPU telemetry and CPU tuning helper with AMD SMU and Intel OC mailbox evidence. | Low for Mystic Light motherboard RGB. | No MS-7E75/header/Mystic strings found. | Likely unrelated to lighting; keep separate from RGB claims. |

## Likely Main Generic Transport Path

The strongest static chain for generic MSI Mystic Light SMBus/Renesas-style traffic is:

```text
MBAPI_x86.dll-like layer
  -> SMBus_Engine.dll
  -> Driver_Engine.dll function table
  -> Driver_Engine.dll CreateFileW / DeviceIoControl
  -> NTIOLib.sys or NTIOLib_X64.sys
```

This chain is likely because:

- MBAPI references and initializes both `SMBus_Engine.dll` and `Driver_Engine.dll`.
- MBAPI contains Mystic Light LED entry points and Renesas-related API names.
- `SMBus_Engine.dll` exports the byte/block wrappers MBAPI references.
- `SMBus_Engine.dll` stores a Driver Engine-like initialization argument and uses Driver Engine PCI config offsets.
- `Driver_Engine.dll` directly supplies the generic privileged bridge to NTIOLib through `DeviceIoControl`.

This is a generic transport conclusion, not an MS-7E75 board conclusion.

## Unrelated Or Low-Relevance Modules

`rtk_bridge.dll` looks like a Realtek bridge helper rather than a motherboard header controller. It scans `\\.\PhysicalDrive%i` and `\\.\%c:` handles, probes Realtek bridge targets, and exposes bridge LED helper exports. It has no static MS-7E75, B850, JRGB, or JRAINBOW strings. It should remain low relevance unless static MBAPI dispatch or external inventory ties it to the target board.

`CPU_Engine.dll` looks like CPU telemetry and CPU tuning infrastructure. Its evidence is CPU-family strings, AMD SMU exports, Intel OC mailbox strings, temperature/clock/power/tuning exports, and no Mystic Light/JRGB/JRAINBOW/Driver Engine/NTIOLib string linkage. It should remain separate from MS-7E75 lighting research unless a future Mystic Light-specific call chain is found.

## Why This Does Not Prove An MS-7E75 LED Register Map

The current evidence identifies generic MSI software layers and transports, but a board LED register map requires board-specific proof that is not present yet.

Missing proof includes:

- The `7E75` board ID is present in a broad MBAPI board list, but no analyzed static source links it to a meaningful MS-7E75 lighting dispatch path.
- No analyzed module maps MS-7E75 to SMBus address `0x52`, Renesas LED commands, EC space, Super I/O GPIO, RTK bridge, or another raw register-level backend.
- No analyzed module identifies MS-7E75 JRGB/JRAINBOW header registers, raw command bytes, or register-level payload layout.
- Decoded online data identifies the static profile source for MS-7E75 zone labels, but it does not establish register behavior.
- The MB800 path provides static HID feature-buffer helper evidence for decoded MS-7E75 zones, but report buffers are not a register map and were not executed.
- NCT/Super I/O evidence in MBAPI includes known older-board behavior such as 7A45-related flags, but that must not be reused for MS-7E75.
- Driver Engine and SMBus Engine explain how privileged and SMBus operations can be performed, not which operations are correct for this board.

Therefore MS-7E75 remains research-only, and hardware access remains blocked.

## What Remains Unknown

- Whether the decoded `NUC126_MB800` / `Class_MB_800` path is the complete live MS-7E75 runtime path.
- Whether MS-7E75 motherboard lighting also uses MBAPI, SMBus/Renesas, EC, Super I/O GPIO, ACPI/WMI, RTK bridge, or another MSI service before or beside the MB800 helper path.
- Whether the MBAPI `LEDMysticControl` SMBus address/command pattern applies to MS-7E75 or only to other MSI boards/controllers.
- Whether `NTIOLib_MysticLight` is the installed service name, device name, profile name, or a caller-provided alias.
- The kernel-side IOCTL implementation in `NTIOLib.sys` / `NTIOLib_X64.sys`.
- Exact `SMBus_Engine.dll` Intel/ATI backend vtable mappings and register-level transaction sequences.
- Which MBAPI flags or config records select `SMB_*`, `b_SMB_*`, `n_SMB_*`, EC/SIO, fan LED, RTK bridge, or CPU paths.
- Whether encoded MSI Center profile/config data contains board IDs, model IDs, LED zones, or route selectors.

## Next Static-Only Targets

- Reconstruct the optimized `Lib\MsiHid.dll` `openMyDevice*` control flow around `HidD_GetAttributes`, and search for any remaining HID usage-page/usage checks.
- Cross-reference decoded MS-7E75 style mask `1342D02C23469345A74401`, default index `10`, and suffix `+1301` against MB800 style/effect parsers.
- Decompile `MBAPI_x86.dll` around the `7E75` board-ID list to identify table consumers and dispatch effects.
- Reverse the `Mystic Light\Profile\*.tmp` binary profile blobs.
- Cross-reference MBAPI call sites that pass arguments into `DriverInitialization` and `SMBusInitialization`.
- Cross-reference MBAPI callers of `SMB_*`, `b_SMB_*`, and `n_SMB_*` to identify the Renesas/Mystic Light call families.
- Build a static vtable map for `SMBus_Engine.dll` `IntelSMBus` and `ATISMBus` backends.
- Decompile the SMBus backend byte/block read/write methods to document transaction sequencing without running them.
- Statically inspect installed `NTIOLib.sys` / `NTIOLib_X64.sys`, if available, to map IOCTL dispatch and device names.
- Decompile MBAPI profile/feature-flag logic around `SupportLED`, `LEDMysticControl`, `LEDControl`, EC/SIO helpers, and fan LED helpers.
- Search additional MSI Center modules for ACPI/WMI/HID/USB paths and MS-7E75 board selectors.

## Explicit Hardware-Access Note

No hardware access was enabled or run during this consolidation. MSI Center and Mystic Light were not launched. No hardware monitor command, `/dev/port` command, raw SMBus access, raw Super I/O access, chip-detection command, register-read command, write command, apply command, or board-control hardware support code was run or added.
