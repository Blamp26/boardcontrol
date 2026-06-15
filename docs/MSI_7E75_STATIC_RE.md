# MSI MS-7E75 Static Reverse Engineering Notes

Status: static analysis only, hardware access disabled.

## Scope

This document records static reverse engineering notes from an active Ghidra MCP session while investigating possible MSI Center / Mystic Light / hardware-monitor control paths relevant to MSI MS-7E75 / B850 GAMING PLUS WIFI PZ.

The analysis is limited to strings, imports, function names, and selected decompiler output from the program currently exposed by Ghidra MCP. The MCP toolset did not expose a project/program browser, so the only loaded binary visible from this session is the active PE image. Based on strings, class names, and exported-style function names, the active image appears to be `MBAPI_x86.dll`.

No board support code was added.

## Safety Constraints

- Static analysis only.
- MSI Center was not run.
- Mystic Light was not run.
- No hardware access commands were run.
- No writes to hardware were enabled or run.
- `/dev/port` was not used.
- `detect-chip` and `read-reg` were not used.
- No MS-7E75 register map was inferred from 7A45.
- Findings below are candidates only until corroborated by additional static artifacts and safe public/OS-visible evidence.

## Analyzed Binaries

| Binary / project | Source | Notes |
| --- | --- | --- |
| Active Ghidra program, inferred `MBAPI_x86.dll` | Ghidra MCP active program | PE image with `.text`, `.rdata`, `.data`, `.rsrc`, and `.reloc` segments. Contains `MBAPI_x86.dll`, `CMBAPIApp`, MSI Center Mystic Light registry/log paths, LED control functions, SMBus helper strings, driver-engine helper strings, and NCT6779D references. |

Likely companion modules referenced by the active program but not loaded/inspected in this MCP session:

- `\SMBus_Engine.dll`
- `\Driver_Engine.dll`
- `\CPU_Engine.dll`
- `\rtk_bridge.dll`

## Candidate Control Paths

### Mystic Light / LED

The active program contains direct Mystic Light and LED entry points:

- `LEDMysticControl`
- `LEDMysticControlV2`
- `LEDMysticControlV2_1`
- `LEDControl`
- `SupportLED`
- `SetMysticLEDColor*`
- `SetMysticRainbowMode*`
- `SetMysticBreathingMode*`
- `SetMysticFlashingMode*`
- `ResetLED`
- `ControlFANLED`

`LEDMysticControl` writes through an SMBus function pointer with device address `0x52` and registers/commands `0xe4` or `0xe3`, so Mystic Light support for at least some MSI boards appears to include an SMBus/Renesas LED path.

`LEDControl` and `Init_NCT_LED_ByBoardFlags` use low-level NCT/SIO-style calls through `g_INTEL100MB`, including logical device and register-looking pairs such as `(0x09, 0xe9)`, `(0x09, 0xee)`, `(0x0b, 0xf7)`, `(0x12, 0xe4)`, and `(0x12, 0xe6)`. These are evidence of a Super I/O path in the binary, but they are not evidence that MS-7E75 uses the same map.

### SMBus / Renesas LED

The active program contains `SMBus_Engine.dll` strings and wrappers:

- `_SMBusControl@12`
- `_SMBusControlWord@12`
- `_SMBusControlBlock@16`
- `SMBus_Initial`
- `GetSMBBASE`
- `_RenesasLEDControlV3@64`
- `_RenesasLEDSetBank@4`
- `KeepRenesasLED`

`_SMBusControl@12` calls a function pointer when the SMBus engine is initialized. `LEDMysticControl` calls the same path with address `0x52`, which matches the existing 7A45 research model but must not be treated as proof for MS-7E75.

### Super I/O / EC / Board GPIO

The active program references:

- `NCT6779D`
- `Nuvoton`
- `AVNCT6779D`
- `GetSIO_DefaultWhite`
- `SetSIOGPIO`
- `SetECSpace`
- `GetECSpace`
- `SetECRAM_Mode`
- `SetECRAM_Color`

`SetECSpace` and `GetECSpace` are thin wrappers around helper functions that write/read an indexed space. Names suggest an EC path, but the underlying transport is not resolved from the currently inspected functions.

### Driver / Low-Level Access

The active program references `\Driver_Engine.dll` and dynamic helper names:

- `DriverInitialization`
- `DriverRelease`
- `ReadIoPortByte`
- `WriteIoPortByte`
- `ReadIoPortWord`
- `WriteIoPortWord`
- `ReadPhysicalMemory`
- `ReadPhysicalMemory_Byte`
- `WritePhysicalMemory_Byte`
- `ReadPhysicalMemory_DWORD`
- `WritePhysicalMemory_DWORD`
- `PCIConfigRead`
- `PCIConfigWrite`
- `NTIOLib_MysticLight`

The static evidence strongly suggests that MSI's API layer delegates privileged I/O, physical memory, and PCI config access to a driver engine and/or NTIOLib-named component. The active program imports `CreateFileA/W`, but no static import named `DeviceIoControl` was present in the inspected import list; driver IOCTL details likely live in `Driver_Engine.dll` or are loaded dynamically.

### Hardware Monitor / Fan

Hardware-monitor-related entry points include:

- `GetCPUTemp`
- `GetCPUInfo`
- `GetDRAMInfo3`
- `ControlFANLED`
- `SaveFANLED`

`GetCPUTemp` calls into CPU/helper objects and applies CPU-family-specific offsets for strings like `Summit` and `Threadripper`. `ControlFANLED` writes a sequence of opaque keys like `0xfdae04f8` through a helper object, suggesting a separate firmware/driver-backed LED path for fan LEDs. This is not enough to identify fan-speed control.

### ACPI / WMI / HID / USB

Focused string searches did not find meaningful ACPI/WMI/HID/USB control evidence in the active program. Only unrelated or vendor-table strings appeared, such as a JEDEC vendor string containing `ACPI` and UI framework text containing `Hide`.

## Evidence Table

| Binary / module | Function | Address | String / import / evidence | Why it matters | Confidence | Suggested path |
| --- | --- | --- | --- | --- | --- | --- |
| Active program, inferred `MBAPI_x86.dll` | N/A | `1021dfa0`, `1024106e` | `MBAPI_x86.dll` | Identifies the active binary as MSI MBAPI-like layer or a program carrying that module identity. | Medium | Unknown |
| Active program, inferred `MBAPI_x86.dll` | N/A | `1021dfd4`, `1021e100`, `1021e148` | MSI Center Mystic Light log and registry paths | Ties the active binary to MSI Center / Mystic Light LED component configuration. | High | Unknown |
| Active program, inferred `MBAPI_x86.dll` | `LEDMysticControl` | `10016360` | Calls SMBus function pointer with address `0x52`, command/register `0xe4` or `0xe3` | Direct Mystic Light control path using SMBus-like engine for LED control. | High | SMBus |
| Active program, inferred `MBAPI_x86.dll` | `_SMBusControl@12` | `100153e0` | Function pointer call through SMBus engine object | Generic byte/control wrapper into `SMBus_Engine.dll` object. | High | SMBus |
| Active program, inferred `MBAPI_x86.dll` | `_SMBusControlBlock@16` | `10015440` | Function pointer call through SMBus engine object | Generic block write/control wrapper; relevant for RGB controllers needing multi-byte payloads. | High | SMBus |
| Active program, inferred `MBAPI_x86.dll` | `SMBus_Initial` | `10040140` | Calls SMBus engine initialization function pointer | Confirms the module initializes an SMBus helper before LED/DDR operations. | High | SMBus |
| Active program, inferred `MBAPI_x86.dll` | `GetSMBBASE` | `1003f620` | Calls SMBus engine function pointer at offset `0x2c` | Suggests the module can query SMBus base/addressing data through the helper engine. | Medium | SMBus |
| Active program, inferred `MBAPI_x86.dll` | `_RenesasLEDControlV3@64` | `10015b40` | Calls internal `FUN_10015470` when global init flag is set | Explicit Renesas LED control entry point, likely layered over SMBus. | High | SMBus |
| Active program, inferred `MBAPI_x86.dll` | `LEDControl` | `10015bd0` | Reads/writes NCT/SIO-style LDN/register pairs via `g_INTEL100MB` helper | Evidence of a Super I/O LED control path in the binary. Not board proof for MS-7E75. | High | Driver IOCTL / unknown SIO transport |
| Active program, inferred `MBAPI_x86.dll` | `Init_NCT_LED_ByBoardFlags` | `1000a2f0` | Branches on `flag_7A45_NCT_LED`, uses NCT-style register accesses | Shows 7A45-specific NCT LED behavior exists in this binary and must not be reused for 7E75. | High | Driver IOCTL / unknown SIO transport |
| Active program, inferred `MBAPI_x86.dll` | `SupportLED` | `10015380` | Checks multiple LED capability flags including `flag_7A45_NCT_LED` | Indicates board-feature flags select LED control backends. | Medium | Unknown |
| Active program, inferred `MBAPI_x86.dll` | N/A | `1022184c`, `1024c54c` | `NCT6779D`, `.?AVNCT6779D@@` | Confirms Nuvoton NCT6779D support in this binary. Not evidence for MS-7E75 controller identity. | High | Driver IOCTL / unknown SIO transport |
| Active program, inferred `MBAPI_x86.dll` | N/A | `1020c208` | `0xAD;Nuvoton;` | Vendor string found in likely JEDEC/manufacturer tables; weak Nuvoton corroboration only. | Low | Unknown |
| Active program, inferred `MBAPI_x86.dll` | `SetSIOGPIO` | `1003f680` | Calls helper `FUN_10045490` when SIO object is present | Suggests a board GPIO path separate from high-level Mystic Light calls. | Medium | Driver IOCTL / unknown SIO transport |
| Active program, inferred `MBAPI_x86.dll` | `SetECSpace` | `1003f6c0` | Calls `FUN_10047680(param_1, param_2, param_3)` | Name suggests EC indexed write path, transport unresolved. | Medium | Unknown / EC |
| Active program, inferred `MBAPI_x86.dll` | `GetECSpace` | `1003f700` | Calls `FUN_100478a0(param_1, param_2)` and returns byte | Name suggests EC indexed read path, transport unresolved. | Medium | Unknown / EC |
| Active program, inferred `MBAPI_x86.dll` | `_IT8295QFN_OP@20` | `1003e9e0` | Uses SMBus-style function pointers and ITE DDR strings nearby | Evidence for ITE LED/DDR device handling in the module. | Medium | SMBus / unknown |
| Active program, inferred `MBAPI_x86.dll` | N/A | `10221164` through `1022124c` | `\SMBus_Engine.dll`, `SMBusInitialization`, `SMBusGetAddress`, `SMBusReload`, `SMBusRelease` | Companion SMBus module is dynamically referenced. | High | SMBus |
| Active program, inferred `MBAPI_x86.dll` | N/A | `10221318` through `10221454` | `\Driver_Engine.dll`, `DriverInitialization`, `ReadIoPort*`, `WriteIoPort*`, `Read/WritePhysicalMemory*`, `PCIConfigRead/Write` | Strong evidence that privileged hardware I/O is delegated to a low-level driver module. | High | Driver IOCTL |
| Active program, inferred `MBAPI_x86.dll` | N/A | `1021e1a8` | `NTIOLib_MysticLight` | Suggests a named low-level driver/library path associated with Mystic Light. | High | Driver IOCTL |
| Active program, inferred `MBAPI_x86.dll` | Imports | `EXTERNAL:00000003`, `EXTERNAL:00000068` | `CreateFileW`, `CreateFileA` imports | Could open files/devices/modules; not sufficient alone to prove direct device IOCTL in this binary. | Low | Unknown / driver |
| Active program, inferred `MBAPI_x86.dll` | Imports | N/A | No `DeviceIoControl` import found in paged import list | Suggests IOCTL may be hidden in `Driver_Engine.dll` or dynamically loaded, or not in this module. | Medium | Driver IOCTL |
| Active program, inferred `MBAPI_x86.dll` | Strings | N/A | No meaningful `MS-7E75`, `7E75`, or `B850` strings found | Active binary does not appear to contain explicit MS-7E75 board matching text. | Medium | Unknown |
| Active program, inferred `MBAPI_x86.dll` | Strings | N/A | No meaningful `JRAINBOW` or `JRGB` strings found | Header labels may be represented elsewhere, not in this module, or by numeric board profiles. | Medium | Unknown |
| Active program, inferred `MBAPI_x86.dll` | Strings | N/A | No meaningful `ACPI`, `WMI`, `HID`, `USB`, `SetupDi`, or `HidD` control strings found | The active binary does not currently point to these transport paths. | Medium | Unknown |
| Active program, inferred `MBAPI_x86.dll` | `GetCPUTemp` | `100151f0` | CPU helper calls and CPU-family strings `Summit`, `Threadripper` | Hardware-monitor-like sensor path exists, but not enough to map motherboard sensors or fans. | Medium | Unknown |
| Active program, inferred `MBAPI_x86.dll` | `ControlFANLED` | `1001c1c0` | Writes opaque keys including `0xfdae04f8` through helper object | Candidate fan LED path, not fan-speed control proof. | Medium | Unknown / driver |
| Active program, inferred `MBAPI_x86.dll` | `SaveFANLED` | Function name present | `C:\MSI\GamingAPP\FANLED.cfg` | Fan LED state persistence path; older GamingAPP path may still be reused. | Medium | Unknown |

## Open Questions

- Which exact MSI Center package and version supplied the active binary?
- Can Ghidra expose or load the companion `SMBus_Engine.dll`, `Driver_Engine.dll`, `CPU_Engine.dll`, and `rtk_bridge.dll` for static inspection?
- Does MS-7E75 select an SMBus/Renesas path, an EC/SIO path, a USB/HID path, or an MSI firmware service path?
- Where are board IDs such as MS-7E75 represented if not as plain strings in the active binary?
- What does `Driver_Engine.dll` use internally: device object names, `DeviceIoControl`, service control manager APIs, or another IPC layer?
- What do the `SetECSpace` / `GetECSpace` helper functions actually access?
- Are JRGB/JRAINBOW headers controlled by a board LED controller, EC, Super I/O GPIO, SMBus LED controller, or a combination?
- Are hardware monitor fan/sensor controls separate from Mystic Light LED controls?

## Next Static-Analysis Tasks

- Load and inventory `SMBus_Engine.dll`, `Driver_Engine.dll`, `CPU_Engine.dll`, and `rtk_bridge.dll` in Ghidra.
- In `Driver_Engine.dll`, search for device names, service names, `CreateFile`, `DeviceIoControl`, IOCTL constants, and `NTIOLib_MysticLight`.
- In `SMBus_Engine.dll`, identify address, command, word, byte, and block transaction wrappers and any bus-locking or chipset-specific logic.
- Cross-reference the `MSI Center\Component\Mystic Light` registry paths and capability flags to find board-profile selection.
- Cross-reference `flag_7A45_NCT_LED` and neighboring flags to identify how board IDs map to LED backends.
- Search for MS-7E75 / 7E75 / B850 in additional MSI Center modules and data files, not only the active `MBAPI_x86.dll`-like program.
- Decompile `FUN_10047680` and `FUN_100478a0` to resolve the `SetECSpace` / `GetECSpace` transport.
- Decompile `FUN_10045490` to resolve the `SetSIOGPIO` transport.
- Inspect export tables and callers for public API surface, but keep all work static.

## Explicit Hardware-Access Note

No hardware access was enabled or run during this investigation. MSI Center was not launched. No hardware monitor command, `/dev/port` command, Super I/O detection command, register-read command, write command, or board-control support code was run or added.
