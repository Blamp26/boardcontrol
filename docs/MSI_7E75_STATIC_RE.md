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
| [MSI_7E75_DRIVER_ENGINE_STATIC_RE.md](MSI_7E75_DRIVER_ENGINE_STATIC_RE.md) | Direct `Driver_Engine.dll` transport, service, device, and IOCTL evidence. |
| [MSI_7E75_SMBUS_ENGINE_STATIC_RE.md](MSI_7E75_SMBUS_ENGINE_STATIC_RE.md) | Direct `SMBus_Engine.dll` SMBus transaction, controller-selection, and Renesas-adjacent evidence. |
| [MSI_7E75_RTK_BRIDGE_STATIC_RE.md](MSI_7E75_RTK_BRIDGE_STATIC_RE.md) | Direct `rtk_bridge.dll` Realtek bridge/device-handle evidence and relevance assessment. |
| [MSI_7E75_CPU_ENGINE_STATIC_RE.md](MSI_7E75_CPU_ENGINE_STATIC_RE.md) | Direct `CPU_Engine.dll` CPU telemetry/tuning evidence and relevance assessment. |

## Confirmed Static Evidence

- The MBAPI-like layer contains Mystic Light and LED entry points such as `LEDMysticControl`, `LEDMysticControlV2`, `LEDControl`, `SupportLED`, `SetMysticLEDColor*`, `SetMysticRainbowMode*`, `ResetLED`, and `ControlFANLED`.
- The MBAPI-like layer dynamically references companion modules including `\SMBus_Engine.dll`, `\Driver_Engine.dll`, `\CPU_Engine.dll`, and `\rtk_bridge.dll`.
- The MBAPI-like layer contains `NTIOLib_MysticLight` and resolves `Driver_Engine.dll` exports for port I/O, CMOS, MSR, physical memory, and PCI config access.
- The actual `Driver_Engine.dll` imports and uses `CreateFileW`, `DeviceIoControl`, and Service Control Manager APIs.
- The actual `Driver_Engine.dll` embeds `NTIOLib.sys` and `NTIOLib_X64.sys`, constructs `\\.\<caller-provided-name>` device paths, and delegates privileged operations to a kernel driver through visible `0xc350....` IOCTL constants.
- The actual `SMBus_Engine.dll` exports SMBus byte/block/check/SPD wrappers and initializes from a Driver Engine-like object supplied by its caller.
- `SMBus_Engine.dll` uses Driver Engine PCI config function-table offsets to select an `IntelSMBus` or `ATISMBus` backend and discover SMBus controller/base information.
- `SMBus_Engine.dll` contains Renesas-labeled synchronization/log strings, but no hard-coded MS-7E75, JRGB, JRAINBOW, or Renesas LED address proof was found in that DLL.
- `rtk_bridge.dll` is a Realtek bridge helper with storage-device scan paths, `DeviceIoControl` bridge commands, and bridge-attached LED helper exports.
- `CPU_Engine.dll` is a CPU telemetry/tuning helper with AMD SMU, Intel mailbox, temperature, ratio, power, and current-limit exports.
- None of the analyzed modules contains specific `MS-7E75`, `7E75`, `B850`, `JRGB`, or `JRAINBOW` proof tying MS-7E75 motherboard headers to a concrete transport or register map.

## Summary Table

| Module | Confirmed role | Relevance to MS-7E75 Mystic Light | Board-specific proof found? | Current assessment |
| --- | --- | --- | --- | --- |
| MBAPI-like boundary | Mystic Light API layer; loads companion engines; contains LED, SMBus, Driver Engine, EC/SIO, NCT, and fan LED symbols. | High as the orchestration layer, but still not board-specific for MS-7E75. | No `MS-7E75` / `7E75` / `B850` proof found. | Important starting point; likely chooses feature/backend paths elsewhere through flags or data. |
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

- No analyzed module contains `MS-7E75`, `7E75`, or meaningful `B850` board dispatch evidence.
- No analyzed module maps MS-7E75 to SMBus address `0x52`, Renesas LED commands, EC space, Super I/O GPIO, RTK bridge, or another concrete backend.
- No analyzed module identifies MS-7E75 JRGB/JRAINBOW header registers, command bytes, payload layout, zones, or controller identity.
- NCT/Super I/O evidence in MBAPI includes known older-board behavior such as 7A45-related flags, but that must not be reused for MS-7E75.
- Driver Engine and SMBus Engine explain how privileged and SMBus operations can be performed, not which operations are correct for this board.

Therefore MS-7E75 remains research-only, and hardware access remains blocked.

## What Remains Unknown

- Where MSI Center stores or computes the MS-7E75 board/profile selection.
- Whether MS-7E75 motherboard lighting uses SMBus/Renesas, EC, Super I/O GPIO, ACPI/WMI, USB/HID, RTK bridge, or another MSI service.
- Whether the MBAPI `LEDMysticControl` SMBus address/command pattern applies to MS-7E75 or only to other MSI boards/controllers.
- Whether `NTIOLib_MysticLight` is the installed service name, device name, profile name, or a caller-provided alias.
- The kernel-side IOCTL implementation in `NTIOLib.sys` / `NTIOLib_X64.sys`.
- Exact `SMBus_Engine.dll` Intel/ATI backend vtable mappings and register-level transaction sequences.
- Which MBAPI flags or config records select `SMB_*`, `b_SMB_*`, `n_SMB_*`, EC/SIO, fan LED, RTK bridge, or CPU paths.
- Whether MSI Center profile/config data outside these DLLs contains board IDs, model IDs, or route selectors.

## Next Static-Only Targets

- Search MSI Center Mystic Light profile/config/database files for `MS-7E75`, `7E75`, `B850`, `JRGB`, `JRAINBOW`, board IDs, and route selectors.
- Cross-reference MBAPI call sites that pass arguments into `DriverInitialization` and `SMBusInitialization`.
- Cross-reference MBAPI callers of `SMB_*`, `b_SMB_*`, and `n_SMB_*` to identify the Renesas/Mystic Light call families.
- Build a static vtable map for `SMBus_Engine.dll` `IntelSMBus` and `ATISMBus` backends.
- Decompile the SMBus backend byte/block read/write methods to document transaction sequencing without running them.
- Statically inspect installed `NTIOLib.sys` / `NTIOLib_X64.sys`, if available, to map IOCTL dispatch and device names.
- Decompile MBAPI profile/feature-flag logic around `SupportLED`, `LEDMysticControl`, `LEDControl`, EC/SIO helpers, and fan LED helpers.
- Search additional MSI Center modules for ACPI/WMI/HID/USB paths and MS-7E75 board selectors.

## Explicit Hardware-Access Note

No hardware access was enabled or run during this consolidation. MSI Center and Mystic Light were not launched. No hardware monitor command, `/dev/port` command, raw SMBus access, raw Super I/O access, chip-detection command, register-read command, write command, apply command, or board-control hardware support code was run or added.
