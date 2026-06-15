# MSI MS-7E75 Driver Engine Static Reverse Engineering Notes

Status: static analysis only, hardware access disabled.

## Scope

This document records a static Ghidra/headless pass on the actual MSI Mystic Light `Driver_Engine.dll`, plus the older MBAPI boundary evidence that led to this module.

The direct pass investigated exports, imports, strings, functions, `NTIOLib` references, device and service paths, driver filenames, service-control paths, `CreateFileW`, `DeviceIoControl`, visible IOCTL constants, I/O port helpers, physical-memory helpers, PCI config helpers, and board-specific strings.

This is documentation-only research for MS-7E75 / B850 GAMING PLUS WIFI PZ. It does not enable MS-7E75 hardware access and does not claim MS-7E75 uses any path unless evidence is specific.

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

| Field | Value |
| --- | --- |
| Binary | `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib\Driver_Engine.dll` |
| SHA-256 | `3A558050D1B82FE30BE75AA23338AE2A3C86E72BE65496FACB18E1838A138B6B` |
| Size | `1,858,576` bytes |
| Timestamp on disk | `2023-12-28 10:22:08` local time |
| Ghidra language | `x86:LE:32:default` |
| Image base | `10000000` |
| Function count | `10873` |
| Import count | `583` |
| String count | `2610` |

Ghidra/headless reported successful import, analysis, script execution, and save for a temporary copy of the DLL. The DLL was not executed as an operating-system process.

## Direct Driver Engine Evidence

### Exports

The actual `Driver_Engine.dll` export table contains the low-level functions previously resolved by MBAPI:

| Export | Address |
| --- | --- |
| `DriverInitialization` | `10002d90` |
| `DriverRelease` | `10002e40` |
| `PCIConfigRead` | `10002e60` |
| `PCIConfigWrite` | `10002eb0` |
| `Rdmsr` | `10002ef0` |
| `ReadCMOSByte` | `10002f40` |
| `ReadIoPortByte` | `10002f80` |
| `ReadIoPortWord` | `10002fc0` |
| `ReadPhysicalMemory` | `10003000` |
| `ReadPhysicalMemory_Byte` | `10003040` |
| `ReadPhysicalMemory_DWORD` | `10003080` |
| `WriteCMOSByte` | `100030c0` |
| `WriteIoPortByte` | `100030f0` |
| `WriteIoPortWord` | `10003120` |
| `WritePhysicalMemory_Byte` | `10003150` |
| `WritePhysicalMemory_DWORD` | `10003190` |
| `Wrmsr` | `100031d0` |

The exported wrappers stage an operation selector and arguments, then call an internal dispatcher. The wrappers do not contain inline port, PCI, MSR, or physical-memory instructions in the decompiled output inspected here.

### Imports

The direct DLL statically imports the Windows APIs needed for a user-mode privileged-driver bridge:

| Import | Evidence |
| --- | --- |
| `KERNEL32.DLL::CreateFileW` | Imported and used by the device-open helper. |
| `KERNEL32.DLL::DeviceIoControl` | Imported and referenced by the low-level request helpers. |
| `KERNEL32.DLL::CloseHandle` | Imported for device-handle cleanup. |
| `KERNEL32.DLL::GetLastError` | Imported for service/open/start error handling. |
| `KERNEL32.DLL::GetNativeSystemInfo` | Present near the NTIOLib driver filename evidence; likely supports x86/x64 driver selection. |
| `ADVAPI32.DLL::OpenSCManagerW` | Imported and used by service-management code. |
| `ADVAPI32.DLL::CreateServiceW` | Imported and used by the service-create helper. |
| `ADVAPI32.DLL::OpenServiceW` | Imported and used by service query/start/stop/delete helpers. |
| `ADVAPI32.DLL::StartServiceW` | Imported and used by the service-start helper. |
| `ADVAPI32.DLL::ControlService` | Imported and used by the service-stop helper. |
| `ADVAPI32.DLL::DeleteService` | Imported and used by the service-delete helper. |
| `ADVAPI32.DLL::ChangeServiceConfigW` | Imported and used by a service-config helper. |

### Strings

Direct strings of interest:

| String | Address | Meaning |
| --- | --- | --- |
| `\\.\` | `10142cb4` | DOS device-path prefix constructed before `CreateFileW`. |
| `NTIOLib_X64.sys` | `10142cc0` | Candidate x64 kernel-driver filename. |
| `NTIOLib.sys` | `10142ce0` | Candidate x86/kernel-driver filename. |
| `DeviceIoControl` | `1018d2d2` | Imported API name. |
| `CreateFileW` | `1018d2e4` | Imported API name. |
| `ControlService` | `1018f330` | Imported API name. |
| `CreateServiceW` | `1018f342` | Imported API name. |
| `DeleteService` | `1018f354` | Imported API name. |
| `OpenSCManagerW` | `1018f364` | Imported API name. |
| `OpenServiceW` | `1018f376` | Imported API name. |
| `StartServiceW` | `1018f39c` | Imported API name. |
| `Driver_Engine.dll` | `1018fc02` | Self/module string. |

String searches in the direct DLL found no `NTIOLib_MysticLight`, `Mystic`, `MS-7E75`, `7E75`, `B850`, `JRGB`, `JRAINBOW`, `ARGB`, `SMBus`, or Nuvoton/NCT board-control strings. `RGB` hits were generic MFC/UI color strings, not board LED evidence.

### Service And Device Path

The device-open helper constructs:

```text
\\.\<caller-provided service/device name>
```

It then calls:

```text
CreateFileW(path, 0xc0000000, 0, NULL, 3, 0x80, NULL)
```

The handle is stored in the Driver Engine object and later used by `DeviceIoControl` helpers.

The service-management code opens the Service Control Manager with `OpenSCManagerW(NULL, NULL, 0xf003f)`. Static decompilation shows helper paths for:

- `CreateServiceW(serviceName, serviceName, 0xf01ff, 1, 3, 1, driverPath, ...)`
- treating `GetLastError() == 0x431` as an already-existing service condition.
- `OpenServiceW(..., 0xf01ff)` followed by `StartServiceW`.
- treating `GetLastError() == 0x420` as an already-running service condition.
- `ControlService(service, 1, ...)` for stop.
- `DeleteService` for removal.
- `ChangeServiceConfigW(..., driverPath, ...)` for path/config updates.

The direct DLL hard-codes `NTIOLib.sys` and `NTIOLib_X64.sys`, but it does not hard-code `NTIOLib_MysticLight`. The older MBAPI boundary evidence indicates `NTIOLib_MysticLight` is supplied from MBAPI/caller context rather than embedded in this DLL.

### DeviceIoControl And IOCTL Constants

The direct DLL uses `DeviceIoControl` through internal helper functions. References to the imported API were found from multiple internal functions, including helpers corresponding to init/auth, MSR, I/O port, physical memory, and PCI config operations.

Visible IOCTL constants:

| IOCTL | Static role observed |
| --- | --- |
| `0xc350214c` | Initialization/auth request carrying the third `DriverInitialization` argument. MBAPI previously passes `0x2f405a34`. |
| `0xc3502084` | `Rdmsr` helper: 4-byte input, 8-byte output. |
| `0xc3502088` | `Wrmsr` helper: 12-byte input, status/output word. |
| `0xc35060cc` | Read I/O port byte; also used after CMOS address selection. |
| `0xc35060d0` | Read I/O port word. |
| `0xc350a0d8` | Write I/O port byte; also used for CMOS address/data writes. |
| `0xc350a0dc` | Write I/O port word. |
| `0xc3506104` | Read physical memory. |
| `0xc350a108` | Write physical memory. |
| `0xc3506144` | PCI config read. |
| `0xc350a148` | PCI config write. |

### Export Operation Selectors

The exported wrappers write a selector into internal state before dispatching:

| Selector | Exported operation |
| --- | --- |
| `0x01` | `ReadIoPortByte` |
| `0x02` | `WriteIoPortByte` |
| `0x03` | `ReadIoPortWord` |
| `0x04` | `WriteIoPortWord` |
| `0x05` | `ReadCMOSByte` |
| `0x06` | `WriteCMOSByte` |
| `0x07` | `Rdmsr` |
| `0x08` | `Wrmsr` |
| `0x09` | `ReadPhysicalMemory` |
| `0x0a` | `ReadPhysicalMemory_Byte` |
| `0x0b` | `WritePhysicalMemory_Byte` |
| `0x0c` | `ReadPhysicalMemory_DWORD` |
| `0x0d` | `WritePhysicalMemory_DWORD` |
| `0x0e` | `PCIConfigRead` |
| `0x0f` | `PCIConfigWrite` |

`PCIConfigRead` and `PCIConfigWrite` compute a PCI address from bus/device/function fields plus offset and size arguments, then dispatch to the PCI IOCTL helpers.

## Older MBAPI Boundary Evidence

The previous MBAPI-focused pass remains useful as boundary evidence:

- The active MBAPI-like binary dynamically constructs a sibling path ending in `\Driver_Engine.dll`.
- It calls `LoadLibraryA` and resolves Driver Engine exports with `GetProcAddress`.
- It calls `DriverInitialization(param_2, param_3, 0x2f405a34)`.
- It stores function pointers for I/O port, CMOS, MSR, physical-memory, PCI config, and release helpers.
- MBAPI-side NCT helper functions call the resolved `ReadIoPortByte` and `WriteIoPortByte` pointers.
- `NTIOLib_MysticLight` appears in MBAPI-side strings, not in the direct `Driver_Engine.dll` strings from this pass.

This boundary evidence explains how MBAPI reaches the Driver Engine transport layer. It does not prove that MS-7E75 uses the NCT/Super I/O paths.

## Evidence Table

| Module | Artifact | Address / value | Evidence | Interpretation | Confidence |
| --- | --- | --- | --- | --- | --- |
| `Driver_Engine.dll` | SHA-256 | `3A558050D1B82FE30BE75AA23338AE2A3C86E72BE65496FACB18E1838A138B6B` | Hash of actual MSI Center Mystic Light library file. | Identifies the analyzed binary. | High |
| `Driver_Engine.dll` | Export table | `10002d90`-`100031d0` | Exports Driver initialization, release, port I/O, CMOS, MSR, physical memory, and PCI config helpers. | Confirms MBAPI's resolved export list exists in the actual DLL. | High |
| `Driver_Engine.dll` | `CreateFileW` import/use | `EXTERNAL:00000099` | Device-open helper constructs `\\.\` plus caller-provided name and calls `CreateFileW`. | User-mode access is through a DOS device path. | High |
| `Driver_Engine.dll` | `DeviceIoControl` import/use | `EXTERNAL:0000009a` | Multiple internal helpers call `DeviceIoControl` with `0xc350....` constants. | Low-level operations are delegated to a kernel driver. | High |
| `Driver_Engine.dll` | Driver filenames | `NTIOLib_X64.sys`, `NTIOLib.sys` | Strings are embedded near driver/service setup code. | Candidate kernel driver files used by Driver Engine. | High |
| `Driver_Engine.dll` | Service-management imports | `CreateServiceW`, `OpenServiceW`, `StartServiceW`, `ControlService`, `DeleteService` | Static imports and decompiled helper paths. | Driver Engine can install/start/stop/delete or reconfigure its kernel service. | High |
| `Driver_Engine.dll` | Init IOCTL | `0xc350214c` | Initialization helper sends the third init argument to the driver. | Explains MBAPI's `0x2f405a34` initialization constant crossing into the kernel-driver bridge. | High |
| `Driver_Engine.dll` | Port I/O IOCTLs | `0xc35060cc`, `0xc35060d0`, `0xc350a0d8`, `0xc350a0dc` | Read/write byte/word helpers dispatch via `DeviceIoControl`. | Port access is delegated to the kernel driver, not done inline in user-mode wrappers. | High |
| `Driver_Engine.dll` | Physical-memory IOCTLs | `0xc3506104`, `0xc350a108` | Physical-memory helpers dispatch via `DeviceIoControl`. | Physical-memory access is delegated to the kernel driver. | High |
| `Driver_Engine.dll` | PCI config IOCTLs | `0xc3506144`, `0xc350a148` | PCI helpers compute bus/device/function address and dispatch via `DeviceIoControl`. | PCI config access is delegated to the kernel driver. | High |
| `Driver_Engine.dll` | `NTIOLib_MysticLight` string | Not found | Focused string search returned zero direct hits. | The name is likely supplied by MBAPI/caller context, not embedded in Driver Engine. | High |
| `Driver_Engine.dll` | Board-specific strings | Not found | No `MS-7E75`, `7E75`, `B850`, `JRGB`, `JRAINBOW`, or `ARGB` strings. | Driver Engine appears generic and does not itself identify MS-7E75 Mystic Light routing. | High |
| MBAPI boundary | Dynamic load | `\Driver_Engine.dll` | Older static pass showed `LoadLibraryA` and `GetProcAddress` resolution. | MBAPI reaches Driver Engine dynamically. | High |
| MBAPI boundary | `NTIOLib_MysticLight` | MBAPI string | Present in MBAPI-side strings. | Candidate service/device identity passed into Driver Engine. Exact role remains unresolved. | Medium |
| MBAPI boundary | NCT helpers | MBAPI functions | Calls resolved Driver Engine port I/O pointers. | Explains old NCT helper transport but is not MS-7E75-specific evidence. | High |

## Candidate Driver / Device / IOCTL Paths

Confirmed candidate path shape:

```text
MBAPI/control layer
  -> LoadLibraryA("...\Driver_Engine.dll")
  -> DriverInitialization(serviceOrDeviceName, driverPathOrContext, 0x2f405a34)
  -> Driver_Engine.dll service setup/open
  -> CreateFileW("\\\\.\\<caller-provided name>", ...)
  -> DeviceIoControl(handle, 0xc350...., ...)
  -> NTIOLib.sys or NTIOLib_X64.sys kernel driver
```

Candidate service/device names:

- `NTIOLib_MysticLight` is confirmed in MBAPI-side strings.
- `NTIOLib_MysticLight` is not embedded in direct `Driver_Engine.dll`.
- The direct DLL accepts service/device names from caller-controlled initialization state.

Candidate driver filenames:

- `NTIOLib.sys`
- `NTIOLib_X64.sys`

Visible IOCTL families:

- initialization/auth: `0xc350214c`
- MSR: `0xc3502084`, `0xc3502088`
- I/O port and CMOS helper path: `0xc35060cc`, `0xc35060d0`, `0xc350a0d8`, `0xc350a0dc`
- physical memory: `0xc3506104`, `0xc350a108`
- PCI config: `0xc3506144`, `0xc350a148`

## Confirmed vs Unknown

Confirmed:

- The actual `Driver_Engine.dll` was analyzed directly by static Ghidra/headless tooling.
- The DLL exports the low-level functions MBAPI resolves dynamically.
- The DLL imports and uses `CreateFileW` and `DeviceIoControl`.
- The DLL imports and uses Service Control Manager APIs.
- The DLL embeds `NTIOLib.sys` and `NTIOLib_X64.sys`.
- The DLL constructs a `\\.\<caller-provided-name>` device path.
- Port I/O, CMOS helper behavior, MSR, physical-memory, and PCI config operations are delegated to a kernel driver through `DeviceIoControl`.
- Several IOCTL constants are visible.
- `NTIOLib_MysticLight` is MBAPI-side evidence, not a direct Driver Engine string.
- No MS-7E75 hardware access was enabled or run.

Unknown:

- The exact kernel-driver implementation behind the `0xc350....` IOCTLs.
- Exact input/output structure layouts for every IOCTL beyond the visible buffer sizes and decompiled call shapes.
- Whether `NTIOLib_MysticLight` is always the service name, always the device name, or one profile among several caller-provided names.
- Whether MS-7E75 Mystic Light uses this Driver Engine path for RGB control.
- Whether MS-7E75 RGB routing uses Driver Engine, SMBus Engine, RTK bridge, CPU Engine, another MSI module, or a combination.
- Whether any board-specific dispatch exists outside Driver Engine.

## Open Questions

- Which caller passes the definitive `DriverInitialization` service/device name and driver path for the installed Mystic Light package?
- Does the installed service name match `NTIOLib_MysticLight`, or is that only an MBAPI string/profile?
- Can static analysis of the NTIOLib kernel driver map each `0xc350....` IOCTL to kernel-side handlers?
- Are the visible physical-memory and PCI config helpers used by Mystic Light paths, or are they generic Driver Engine capabilities exposed to multiple MSI modules?
- Is any MS-7E75-specific board/profile dispatch present in MBAPI or another companion module rather than Driver Engine?

## Next Static-Analysis Tasks

- Cross-reference MBAPI call sites that provide the first two `DriverInitialization` arguments.
- Statically inspect any installed `NTIOLib.sys` / `NTIOLib_X64.sys` files, if present, for IOCTL dispatch tables and device-name creation.
- Continue comparing MBAPI, SMBus Engine, RTK bridge, CPU Engine, and other Mystic Light modules for board-specific MS-7E75 routing.
- Keep MS-7E75 support disabled until a separate static evidence chain identifies a board-specific, reviewed transport path.

## Explicit Hardware-Access Note

No hardware access was enabled or run during this investigation. MSI Center and Mystic Light were not launched. No hardware monitor command, `/dev/port` command, raw Super I/O access, raw SMBus access, chip-detection command, register-read command, write command, apply command, or board-control hardware support code was run or added.
