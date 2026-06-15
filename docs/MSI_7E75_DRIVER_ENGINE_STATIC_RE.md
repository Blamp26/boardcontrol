# MSI MS-7E75 Driver Engine Static Reverse Engineering Notes

Status: static analysis only, hardware access disabled.

## Scope

This document records the Driver Engine focused follow-up from a Ghidra MCP static-analysis session for MSI MS-7E75 / B850 GAMING PLUS WIFI PZ research.

The task was to inspect `Driver_Engine.dll` for exports, imports, functions, strings, device names, service names, `NTIOLib_MysticLight` references, service/device-open paths, `DeviceIoControl` usage, visible IOCTL constants, I/O port helpers, physical-memory helpers, PCI config helpers, and whether this explains the `MBAPI_x86.dll` transport.

Important limitation: the Ghidra MCP session did not expose a project/program switcher or load command. The active Ghidra program presented the same PE image previously inferred as `MBAPI_x86.dll`, not `Driver_Engine.dll` itself. Therefore, this note documents confirmed static evidence for how the active MBAPI-like binary loads and calls `Driver_Engine.dll`, plus the unknowns that require loading `Driver_Engine.dll` directly in Ghidra. It does not claim to have resolved the internal driver/device/IOCTL implementation.

No board support code was added.

## Safety Constraints

- Static analysis only.
- MSI Center was not run.
- Mystic Light was not run.
- `cargo run -- doctor` was not run.
- `detect-chip`, `read-reg`, `write`, and `apply` were not run.
- `/dev/port` was not touched.
- No raw Super I/O access was performed.
- No raw SMBus access was performed.
- MS-7E75 hardware access was not enabled.
- No MS-7E75 register map was inferred from 7A45.
- The 7A45 NCT register map was not reused for MS-7E75.

## Analyzed Binary

| Binary / project | Source | Status | Notes |
| --- | --- | --- | --- |
| Active Ghidra program, inferred `MBAPI_x86.dll` | Ghidra MCP active program | Analyzed | PE image with `.text`, `.rdata`, `.data`, `.rsrc`, and `.reloc` segments. Exports LED, SMBus, EC, NCT, DRAM, and hardware-monitor helper functions. Contains strings and loader code for `\Driver_Engine.dll`. |
| `Driver_Engine.dll` | Referenced by active program | Not directly loaded through available MCP tools | Internals such as device names, service control-manager use, `DeviceIoControl`, IOCTL constants, and low-level driver calls remain unknown until the DLL itself is active in Ghidra. |

## Candidate Driver / Device / IOCTL Paths

The active MBAPI-like binary constructs a path ending in `\Driver_Engine.dll`, calls `LoadLibraryA`, resolves low-level exports with `GetProcAddress`, and then calls `DriverInitialization` with the caller-provided parameters plus constant `0x2f405a34`.

Resolved Driver Engine export names:

- `DriverInitialization`
- `ReadIoPortByte`
- `WriteIoPortByte`
- `ReadIoPortWord`
- `WriteIoPortWord`
- `ReadCMOSByte`
- `WriteCMOSByte`
- `Rdmsr`
- `Wrmsr`
- `ReadPhysicalMemory`
- `ReadPhysicalMemory_Byte`
- `WritePhysicalMemory_Byte`
- `ReadPhysicalMemory_DWORD`
- `WritePhysicalMemory_DWORD`
- `PCIConfigRead`
- `PCIConfigWrite`
- `DriverRelease`

The MBAPI-side function-pointer layout explains the transport boundary:

- Function pointer offset `0x00`: `DriverInitialization`
- Offset `0x04`: `ReadIoPortByte`
- Offset `0x08`: `WriteIoPortByte`
- Offset `0x0c`: `ReadIoPortWord`
- Offset `0x10`: `WriteIoPortWord`
- Offset `0x14`: `ReadCMOSByte`
- Offset `0x18`: `WriteCMOSByte`
- Offset `0x1c`: `Rdmsr`
- Offset `0x20`: `Wrmsr`
- Offset `0x24`: `ReadPhysicalMemory`
- Offset `0x28`: `ReadPhysicalMemory_Byte`
- Offset `0x2c`: `WritePhysicalMemory_Byte`
- Offset `0x30`: `ReadPhysicalMemory_DWORD`
- Offset `0x34`: `WritePhysicalMemory_DWORD`
- Offset `0x38`: `PCIConfigRead`
- Offset `0x3c`: `PCIConfigWrite`
- Offset `0x40`: `DriverRelease`
- Offset `0x44`: initialization/available flag used by higher-level wrappers

Candidate device/service/IOCTL status:

- Device names: not visible in the active MBAPI-like program. No `\\.\...` device path strings were found by focused string search.
- Service names: not visible in the active MBAPI-like program. Focused string searches for `CreateService`, `OpenService`, and `StartService` returned no string hits, and the import table did not show those SCM APIs.
- `NTIOLib_MysticLight`: present as a string at `1021e1a8`; likely passed into `DriverInitialization`, but exact use is unresolved without direct `Driver_Engine.dll` analysis.
- `CreateFileA/W`: imported by the active program, but no decompiled direct device-open path was confirmed in this follow-up.
- `DeviceIoControl`: no static import and no string hit in the active program. The likely location for IOCTL dispatch is `Driver_Engine.dll` and/or its kernel driver, not the active MBAPI-like binary.
- IOCTL constants: no visible IOCTL constants were confirmed from the active program during this follow-up.

## Evidence Table

| Binary / module | Function / artifact | Address | Evidence | Why it matters | Confidence | Confirmed path |
| --- | --- | --- | --- | --- | --- | --- |
| Active program, inferred `MBAPI_x86.dll` | Memory segments | `10000000`-`102811ff` | PE sections `.text`, `.rdata`, `.data`, `.rsrc`, `.reloc`; no program-switching tool exposed | Confirms the analyzed MCP-visible image is the active program, not necessarily the requested companion DLL. | High | Ghidra MCP active program |
| Active program, inferred `MBAPI_x86.dll` | Exports | Various | Exports include `LEDMysticControl`, `LEDControl`, `_SMBusControl@12`, `SetMysticLEDColor`, `ControlFANLED`, `GetCPUTemp`, `SetECSpace`, `GetECSpace`, NCT helpers | Export surface matches an MSI MBAPI/control-layer DLL, not a low-level driver-engine-only DLL. | High | MBAPI/control layer |
| Active program, inferred `MBAPI_x86.dll` | Imports | `EXTERNAL:00000003`, `EXTERNAL:00000068` | `CreateFileW`, `CreateFileA` imported | The active binary can open files/handles, but this is not proof of a direct device open in the inspected paths. | Low | Unknown |
| Active program, inferred `MBAPI_x86.dll` | Imports | `EXTERNAL:0000008a`-`EXTERNAL:000000b4` | `LoadLibraryW`, `LoadLibraryExW`, `LoadLibraryA`, `GetProcAddress`, `FreeLibrary` imported | Supports dynamic loading of companion engines such as `Driver_Engine.dll`. | High | Dynamic DLL transport |
| Active program, inferred `MBAPI_x86.dll` | Imports | N/A | No static `DeviceIoControl` import found in import pages | Suggests IOCTL usage is not statically imported by this active program, or is hidden in `Driver_Engine.dll`/another layer. | Medium | Unknown / Driver Engine |
| Active program, inferred `MBAPI_x86.dll` | Imports | N/A | No `CreateService`, `OpenService`, or `StartService` imports found in import pages | Service installation/start logic was not visible in the active program import table. | Medium | Unknown / Driver Engine |
| Active program, inferred `MBAPI_x86.dll` | String | `10221318` | `\Driver_Engine.dll` | Confirms the active program dynamically references the Driver Engine companion module. | High | Dynamic DLL transport |
| Active program, inferred `MBAPI_x86.dll` | `FUN_100460d0` | `100460d0` | Builds `\Driver_Engine.dll` path, calls `LoadLibraryA`, resolves Driver Engine exports with `GetProcAddress` | Core evidence that MBAPI delegates privileged operations through `Driver_Engine.dll`. | High | MBAPI -> Driver Engine |
| Active program, inferred `MBAPI_x86.dll` | `FUN_100460d0` | `100460d0` | Calls resolved `DriverInitialization(param_2, param_3, 0x2f405a34)` and stores a readiness byte at object offset `0x44` | Shows Driver Engine initialization is explicit and gated before higher-level wrappers use I/O helpers. | High | MBAPI -> Driver Engine |
| Active program, inferred `MBAPI_x86.dll` | Strings | `10221344`, `10221354`, `10221364`, `10221374` | `ReadIoPortByte`, `WriteIoPortByte`, `ReadIoPortWord`, `WriteIoPortWord` | Driver Engine export names for port I/O. Exact implementation remains inside `Driver_Engine.dll`. | High | Driver Engine export |
| Active program, inferred `MBAPI_x86.dll` | Strings | `10221384`, `10221394`, `102213a4` | `ReadCMOSByte`, `WriteCMOSByte`, `Rdmsr` / nearby `Wrmsr` resolution in decompile | Driver Engine export names include CMOS and MSR access. | High | Driver Engine export |
| Active program, inferred `MBAPI_x86.dll` | Strings | `102213b4`-`10221418` | `ReadPhysicalMemory`, `ReadPhysicalMemory_Byte`, `WritePhysicalMemory_Byte`, `ReadPhysicalMemory_DWORD`, `WritePhysicalMemory_DWORD` | Driver Engine export names for physical memory access. | High | Driver Engine export |
| Active program, inferred `MBAPI_x86.dll` | Strings | `10221434`, `10221444` | `PCIConfigRead`, `PCIConfigWrite` | Driver Engine export names for PCI configuration access. | High | Driver Engine export |
| Active program, inferred `MBAPI_x86.dll` | String | `1021e1a8` | `NTIOLib_MysticLight` | Likely low-level driver/library identity associated with Mystic Light. Exact role remains unresolved. | High | Driver Engine / NTIOLib candidate |
| Active program, inferred `MBAPI_x86.dll` | String search | N/A | No `DeviceIoControl` string hit | No direct string evidence for DeviceIoControl in active program. | Medium | Unknown |
| Active program, inferred `MBAPI_x86.dll` | String search | N/A | No `\\.\` device path strings found | Candidate NT device/user-mode DOS device names are not visible in active program strings. | Medium | Unknown / Driver Engine |
| Active program, inferred `MBAPI_x86.dll` | String search | N/A | No `CreateService`, `OpenService`, or `StartService` string hits | Service-management path is not visible as plain text in the active program. | Medium | Unknown / Driver Engine |
| Active program, inferred `MBAPI_x86.dll` | `NCT6779D_EnterConfig` | Function name present | Calls function pointer at Driver Engine object offset `0x08` with `(0x4e, 0x87)` twice | MBAPI NCT config-entry write is implemented through the resolved `WriteIoPortByte` pointer, not inline raw I/O. This is not MS-7E75 register-map evidence. | High | MBAPI -> Driver Engine port I/O |
| Active program, inferred `MBAPI_x86.dll` | `NCT6779D_ExitConfig` | Function name present | Calls function pointer at Driver Engine object offset `0x08` with `(0x4e, 0xaa)` | MBAPI NCT config-exit write is delegated through Driver Engine. This must not be reused for MS-7E75. | High | MBAPI -> Driver Engine port I/O |
| Active program, inferred `MBAPI_x86.dll` | `NCT6779D_ReadLDNReg` | Function name present | Uses offset `0x08` writes to `0x4e/0x4f`, then offset `0x04` read from `0x4f` | Shows how MBAPI wraps Super I/O indexed reads over Driver Engine `WriteIoPortByte`/`ReadIoPortByte`. This does not identify MS-7E75 hardware. | High | MBAPI -> Driver Engine port I/O |
| Active program, inferred `MBAPI_x86.dll` | `NCT6779D_WriteLDNReg` | Function name present | Uses offset `0x08` writes to `0x4e/0x4f` for LDN/register/value | Shows how MBAPI wraps Super I/O indexed writes over Driver Engine `WriteIoPortByte`. This was observed only statically and was not run. | High | MBAPI -> Driver Engine port I/O |
| Active program, inferred `MBAPI_x86.dll` | `ReleaseDll` | `10040070` | Calls function pointer at Driver Engine object offset `0x40` when readiness flag at `0x44` is set | Confirms a matching `DriverRelease` cleanup callback. | High | MBAPI -> Driver Engine |

## Confirmed vs Unknown

Confirmed:

- The MCP-visible active program is an MBAPI/control-layer style DLL, not directly proven to be `Driver_Engine.dll`.
- The active program dynamically loads a sibling `\Driver_Engine.dll`.
- The active program resolves Driver Engine exports for I/O port, CMOS, MSR, physical memory, and PCI config operations.
- The active program calls `DriverInitialization` with magic-looking constant `0x2f405a34`.
- MBAPI-side NCT helpers use the resolved Driver Engine function-pointer table for port I/O.
- `NTIOLib_MysticLight` appears as a string in the active program.
- No hardware access was enabled or run.

Unknown:

- Exact `Driver_Engine.dll` exports/imports from the DLL's own export/import tables.
- Whether `Driver_Engine.dll` itself imports or dynamically resolves `DeviceIoControl`.
- Exact IOCTL constants, request/response structures, and device handle lifetime.
- Device object names or DOS device paths opened by `Driver_Engine.dll`.
- Service names and whether `CreateService`, `OpenService`, or `StartService` are used in `Driver_Engine.dll`.
- Whether `NTIOLib_MysticLight` is a service name, device name, kernel-driver name, library selector, mutex/registry value, or initialization argument.
- Whether MS-7E75 uses any of the NCT/Super I/O paths present in the active program.
- Whether MS-7E75 RGB/fan/sensor control uses Driver Engine, SMBus Engine, EC helpers, USB/HID, ACPI/WMI, or another MSI layer.

## Open Questions

- Can the actual `Driver_Engine.dll` be loaded as the active Ghidra MCP program for direct inspection?
- Which MSI Center package version supplied `MBAPI_x86.dll`, `Driver_Engine.dll`, `SMBus_Engine.dll`, and the NTIOLib driver?
- Does `Driver_Engine.dll` statically import `DeviceIoControl`, or does it resolve it dynamically through `GetProcAddress`?
- What user-mode device path does Driver Engine open, if any?
- Is `NTIOLib_MysticLight` the kernel service name, device name, driver filename stem, or a logical driver profile name?
- What does the `0x2f405a34` initialization constant select or authenticate?
- Are IOCTL numbers visible as constants in `Driver_Engine.dll`, and can they be grouped by port I/O, physical memory, PCI config, CMOS, and MSR operations?
- Does `Driver_Engine.dll` install/start a service itself, or does MSI Center install the driver elsewhere?
- Is the MBAPI x86 layer talking to a 32-bit `Driver_Engine.dll` that then talks to a 64-bit kernel driver on modern Windows?

## Next Static-Analysis Tasks

- Load `Driver_Engine.dll` directly in Ghidra and make it the active MCP program.
- Inventory `Driver_Engine.dll` exports and compare them with the `GetProcAddress` list from `FUN_100460d0`.
- Inventory `Driver_Engine.dll` imports for `CreateFileA/W`, `DeviceIoControl`, `CloseHandle`, `CreateServiceA/W`, `OpenServiceA/W`, `StartServiceA/W`, `OpenSCManagerA/W`, `GetProcAddress`, and `LoadLibraryA/W`.
- Search `Driver_Engine.dll` strings for `NTIOLib_MysticLight`, `NTIOLib`, `.sys`, `\\.\`, `\Device\`, service names, registry service paths, and IOCTL/debug labels.
- Decompile `DriverInitialization`, `DriverRelease`, `ReadIoPortByte`, `WriteIoPortByte`, physical-memory helpers, and `PCIConfigRead`/`PCIConfigWrite`.
- Identify any `DeviceIoControl` call wrappers and record IOCTL constants, buffer shapes, and operation selectors from static code only.
- Cross-check whether Driver Engine embeds a driver resource or only opens an already installed NTIOLib service.
- Keep MS-7E75 hardware support disabled unless separate, board-specific evidence later justifies a reviewed profile proposal.

## Explicit Hardware-Access Note

No hardware access was enabled or run during this investigation. MSI Center and Mystic Light were not launched. No hardware monitor command, `/dev/port` command, raw Super I/O access, raw SMBus access, chip-detection command, register-read command, write command, apply command, or board-control hardware support code was run or added.
