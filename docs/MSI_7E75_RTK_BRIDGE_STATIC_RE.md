# MSI MS-7E75 RTK Bridge Static Reverse Engineering Notes

Status: research-only, static analysis only.

## Scope

This note records a static-only Ghidra/headless pass over MSI Center Mystic Light's `rtk_bridge.dll` companion module. The goal was to determine whether this Realtek bridge DLL explains any MS-7E75 Mystic Light transport path, especially USB/HID/device-handle paths, LED bridge commands, vendor/product IDs, or links from MBAPI/SMBus/Driver Engine.

These notes are evidence for future research only. They do not add or enable MS-7E75 hardware support.

## Safety Constraints

- No MSI Center process was started.
- No `doctor`, `detect-chip`, `read-reg`, write, or apply command was run.
- No `/dev/port` access was attempted.
- No raw SMBus or Super I/O access was attempted.
- No MS-7E75 hardware access was enabled.
- The target DLL was imported into Ghidra/headless for static analysis only. A byte-identical temporary copy was used for one headless pass to avoid Windows batch quoting problems with `Program Files (x86)`.

## Analyzed Binary

| Field | Value |
| --- | --- |
| Original path | `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib\rtk_bridge.dll` |
| Size | `135272` bytes |
| Last modified | `2024-05-24 11:13:36` |
| SHA-256 | `9809E84E4C3BE0558A9CB513F10945D4C2F606DFE208EB7C5DB55594952EEDF3` |
| Ghidra language | `x86:LE:32:default` |
| Compiler spec | `windows` |
| Image base | `0x10000000` |
| Resource company | `Realtek Corp.` |
| Resource description | `Realtek bridge demo tool` |
| Resource version | `1.0.0.120621` |
| Resource internal/original filename | `rtk_bridge.exe` |

## Evidence Table

| Area | Evidence | Assessment |
| --- | --- | --- |
| Realtek bridge purpose | Resource strings identify `Realtek Corp.` and `Realtek bridge demo tool`; string `Realtek USB CD-ROM` appears near storage scan strings. | Strong evidence this is a Realtek USB/storage bridge helper, not an MSI board-specific DLL. |
| Exports | Exports include `bridge_dev_scan`, `bridge_disk_scan`, `bridge_open_handle`, `bridge_close_handle`, `bridge_check_target`, `bridge_write_unlock`, `led_mem_init`, `led_mem_write_rgb_direct`, `led_set_appctl`, `led_get_ic_support_argb`, `led_get_ic_support_sync_light`, and many `led_eff_set_*` routines. | Strong evidence the DLL exposes a bridge-open/scan layer plus LED control helpers. |
| Imports | Imports include `CreateFileA`, `CloseHandle`, `DeviceIoControl`, `GetLastError`, `CreateFileW`, `LoadLibraryExW`, and `GetProcAddress` from `KERNEL32.DLL`. | Strong evidence it talks through Windows file/device handles and IOCTLs. |
| SetupAPI/HIDAPI | No `SetupDi*`, `HidD*`, `hid.dll`, `VID_`, or `PID_` strings/imports were found in this pass. | No static evidence of HIDAPI/SetupAPI enumeration or embedded USB VID/PID matching. |
| Device path strings | Strings include `\\.\PhysicalDrive%i` and `\\.\%c:`. | Strong evidence scan/open paths target physical-drive and drive-letter handles. |
| Device scanning | Internal `FUN_100011b0` iterates ten `PhysicalDrive` handles, then ten drive-letter handles, calling `CreateFileA(..., 0xc0000000, share 3, OPEN_EXISTING)` on generated paths. | Strong evidence of Windows storage-device scanning. |
| Storage probe IOCTL | `FUN_100011b0` calls `DeviceIoControl(hDevice, 0x2d1400, ..., out 0x28, ...)` and checks an output field equals `7` before `bridge_check_target`. | Candidate storage-property-style probe before Realtek bridge validation. Exact structure naming remains unconfirmed. |
| Realtek command IOCTL | Internal `FUN_10002a60` packages a command buffer and calls `DeviceIoControl(param_1, 0x4d014, in/out 0x50, ...)`, retrying up to five times when `GetLastError()` returns `0x1f`. | Strong evidence of a direct IOCTL wrapper for Realtek bridge commands. |
| Bridge target validation | `bridge_check_target` calls `FUN_100014f0`; it accepts returned target/status values including `0xa0010001`, `0xa0010002`, `0xa0010003`, `0xa0020000`, `0xa0020001`, `0xa0020002`, `0xa0030000`, and `0xa0030001`. | Candidate Realtek bridge family/status identifiers, not board IDs. |
| Realtek command strings | Strings include `dev_scan`, `query`, `get_api_ver`, `get_uuid`, `i2c_init`, `i2c_write`, `i2c_read`, `pcie_read_conf`, `pcie_write_conf`, `pcie_read_mem`, `pcie_write_mem`, `gpio_*`, `led_*`, `uart_*`, `nvme_*`, and `ata_*`. | Strong evidence this generic bridge library can tunnel many bridge functions, including I2C, PCIe, GPIO, UART, storage, and LED commands. |
| LED support strings | Strings include `HW_LED_CFG`, `CUSTOMIZED_LED`, `Color_Always`, `Color_Cycle`, `Color_Blink`, `Color_Breathe`, `led_get_sup_argb`, `led_get_sup_aura`, `led_get_sup_mystic`, and `led_set_rgb_direct`. | Strong evidence of bridge-attached RGB/ARGB feature support. |
| LED memory format | `led_mem_init` allocates `0x4000` bytes and related helpers validate magic `0x4c454453` plus version `2`. `FUN_10003170` writes chunks with bridge command byte `0xe3`, subcommand `0x024c`; `led_mem_write_rgb_direct` uses command `0xe3`, subcommand `0x034c`. | Strong evidence of a Realtek bridge LED memory/config protocol. |
| MBAPI relationship | Earlier MBAPI static notes found companion module string `\rtk_bridge.dll`; this DLL itself has no `MBAPI`, `SMBus_Engine`, or `Driver_Engine` strings. | MBAPI may load/use this module dynamically, but this pass found no reverse dependency from `rtk_bridge.dll`. |
| MSI board/profile strings | No `MS-7E75`, `7E75`, `B850`, `JRGB`, or `JRAINBOW` strings were found. | No static evidence that this DLL specifically selects or describes MS-7E75 motherboard lighting headers. |

## Exports, Imports, Functions, and Strings

Notable exports:

- Bridge/device routines: `bridge_check_target`, `bridge_close_handle`, `bridge_dev_scan`, `bridge_disconnect`, `bridge_disk_scan`, `bridge_open_handle`, `bridge_write_unlock`.
- LED memory/control routines: `led_mem_init`, `led_mem_deinit`, `led_mem_get_rgb_num`, `led_mem_set_bright`, `led_mem_write_rgb_direct`, `led_set_appctl`, `led_get_appctl`, `led_get_ic_support_argb`, `led_get_ic_support_sync_light`.
- LED effects: `led_eff_set_always_on`, `led_eff_set_blink`, `led_eff_set_breathe`, `led_eff_set_meteor`, `led_eff_set_newton_cradle`, `led_eff_set_rainbow_scroll`, `led_eff_set_running_water`, `led_eff_set_scroll`, `led_eff_set_sliding`, `led_eff_set_spectrum`.

Notable imports:

- `KERNEL32.DLL::CreateFileA`
- `KERNEL32.DLL::CloseHandle`
- `KERNEL32.DLL::DeviceIoControl`
- `KERNEL32.DLL::GetLastError`
- `KERNEL32.DLL::CreateFileW`
- `KERNEL32.DLL::LoadLibraryExW`
- `KERNEL32.DLL::GetProcAddress`
- `KERNEL32.DLL::FreeLibrary`

Notable strings:

- `Realtek USB CD-ROM`
- `\\.\PhysicalDrive%i`
- `\\.\%c:`
- `get ata id error\n`
- `PRODUCT`, `MANUFACTURE`, `SERIAL`, `SCSI_VENDOR`, `SCSI_PRODUCT`
- `HW_LED_CFG`, `CUSTOMIZED_LED`, `SUSPEND_LED_OFF`
- `Color_Always`, `Color_Cycle`, `Color_Blink`, `Color_Breathe`
- `i2c_init`, `i2c_write`, `i2c_read`, `i2c_read_wf`
- `pcie_read_conf`, `pcie_write_conf`, `pcie_read_mem`, `pcie_write_mem`
- `led_get_sup_argb`, `led_get_sup_aura`, `led_get_sup_mystic`, `led_set_rgb_direct`, `led_set_app_ctl`, `led_get_app_ctl`

Strings not found in this pass:

- `MS-7E75`
- `7E75`
- `B850`
- `JRGB`
- `JRAINBOW`
- `VID_`
- `PID_`
- `SetupDi`
- `HidD`
- `MBAPI`
- `SMBus_Engine`
- `Driver_Engine`

## Candidate Driver, Device, and IOCTL Paths

Candidate device paths:

- `\\.\PhysicalDrive%i`
- `\\.\%c:`
- Caller-supplied path passed to exported `bridge_open_handle`.

Candidate IOCTLs and bridge commands:

| Candidate | Evidence | Notes |
| --- | --- | --- |
| `CreateFileA(path, 0xc0000000, 3, NULL, 3, 0, NULL)` | Used by `bridge_open_handle` and internal scan helper. | Opens read/write shared handles to caller-supplied, physical-drive, or drive-letter paths. |
| `DeviceIoControl(..., 0x2d1400, ...)` | Used during scan before `bridge_check_target`. | Candidate storage-device query/probe. It is not yet named to a Windows IOCTL constant in these notes. |
| `DeviceIoControl(..., 0x4d014, ...)` | Used by `FUN_10002a60`, the central bridge command wrapper. | Candidate Realtek bridge command IOCTL. |
| Command `0x2000012` | Used by target/version checks and scan metadata reads. | Candidate bridge query command. |
| Command byte `0xe2`, subcommands `0x92`, `0xd0`, `0xbb`, `0x05cc`, `0x02cc` | Used by target validation, scan filtering, config reads, and LED memory init. | Candidate Realtek bridge command family. |
| Command byte `0xe3`, subcommands `0x014c`, `0x024c`, `0x034c`, `0x30` | Used by app-control, LED memory chunk writes, direct RGB writes, and disconnect path. | Candidate Realtek LED/app-control command family. |
| Magic `0x4c454453` with version `2` | Checked by LED memory helpers. | Candidate in-memory LED config header, not a board ID. |

## Confirmed vs Unknown

Confirmed:

- `rtk_bridge.dll` is a 32-bit Realtek bridge DLL with exported bridge and LED helper functions.
- It imports and calls `CreateFileA`, `CloseHandle`, `DeviceIoControl`, and `GetLastError`.
- It scans Windows physical-drive and drive-letter handles and probes them before accepting a Realtek target.
- It contains a central `DeviceIoControl(..., 0x4d014, ...)` wrapper for bridge commands.
- It contains LED/ARGB/Mystic/Aura support strings and exported LED effect/control helpers.
- It contains no MS-7E75, 7E75, B850, JRGB, or JRAINBOW strings in this pass.
- No hardware access was enabled or run.

Unknown:

- Whether any MS-7E75 Mystic Light workflow ever loads or calls this DLL on real hardware.
- Whether `0x2d1400` maps to a specific documented storage IOCTL in this exact call shape.
- Whether `0x4d014` is Realtek-specific, vendor-filter-specific, or handled by an inbox storage/USB bridge driver stack.
- The exact binary contract between MBAPI and these exports.
- Whether bridge target values such as `0xa0010001` identify chip families, firmware states, device classes, or product modes.

## Relevance to MS-7E75 Mystic Light Path

This DLL explains a possible MBAPI companion path for Realtek bridge-attached LED devices. It does not, by itself, explain the MS-7E75 motherboard RGB/ARGB header path.

The evidence points to a Realtek USB/storage bridge LED helper:

- storage-style paths (`\\.\PhysicalDrive%i`, `\\.\%c:`),
- Realtek bridge resource strings,
- `Realtek USB CD-ROM`,
- generic bridge command strings for storage, I2C, PCIe, GPIO, UART, and LED features,
- no MS-7E75/B850/JRGB/JRAINBOW strings.

Therefore, this module should be treated as likely unrelated to the MS-7E75 motherboard Mystic Light path unless future static evidence shows board-specific dispatch into `rtk_bridge.dll`, or OS-visible inventory shows a Realtek bridge LED device relevant to the target system.

## Open Questions

- Which MBAPI functions load and call `rtk_bridge.dll`, and under what product/device condition?
- Does MBAPI select `rtk_bridge.dll` only for external Realtek bridge devices, storage enclosures, monitors, or another Mystic Light accessory class?
- Can the `0x2d1400` and `0x4d014` IOCTLs be named from public Windows headers or Realtek bridge tooling without executing them?
- Are the `0xa001*`, `0xa002*`, and `0xa003*` target codes documented in Realtek bridge SDKs or sample tools?
- Does any MSI Center profile database map MS-7E75 to RTK bridge exports, or is MS-7E75 handled by SMBus/Driver Engine/another module?

## Next Static-Analysis Tasks

- In MBAPI, find imports/dynamic load sites for `rtk_bridge.dll` and identify the decision logic around those calls.
- Decompile MBAPI call sites for `bridge_dev_scan`, `bridge_disk_scan`, `bridge_open_handle`, `led_mem_init`, and `led_mem_write_rgb_direct`.
- Search MSI Center profile/config data for `rtk_bridge`, Realtek bridge target IDs, or accessory-class dispatch names.
- Name `0x2d1400` and `0x4d014` against Windows headers/public documentation if possible.
- Continue keeping RTK bridge evidence separate from MS-7E75 board claims until a board-specific selector is found.

## Explicit Safety Note

No hardware access was enabled or run. This pass used static Ghidra/headless analysis of `rtk_bridge.dll` only.
