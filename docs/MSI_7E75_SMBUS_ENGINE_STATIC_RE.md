# MSI MS-7E75 SMBus Engine Static Reverse Engineering Notes

Status: static analysis only, hardware access disabled.

## Scope

This document records a static Ghidra pass over the actual MSI `SMBus_Engine.dll` used by MSI Center / Mystic Light, with focus on MSI MS-7E75 / B850 GAMING PLUS WIFI PZ research.

The goals were to inspect exports, imports, strings, functions, SMBus byte/block transaction wrappers, Renesas/controller references, controller selection logic, board/profile dispatch strings or IDs, calls into `Driver_Engine.dll` / `NTIOLib_MysticLight`, MS-7E75-related strings, and whether this explains the MBAPI SMBus/Renesas path.

Tooling note: the live Ghidra MCP inspection endpoint still exposed the previously loaded MBAPI-like program and did not provide a load/switch-program operation. To actually load and analyze `SMBus_Engine.dll` itself, this pass used the local Ghidra 12.1.2 headless analyzer against the DLL as a static PE input and dumped the analyzed program metadata/decompiler output with temporary Ghidra scripts. No MSI executable or DLL entry point was run by the OS; this was Ghidra import/decompile only.

No board support code was added.

## Safety Constraints

- Static analysis only.
- MSI Center was not run.
- Mystic Light was not run.
- `cargo run -- doctor` was not run.
- `detect-chip`, `read-reg`, `write`, and `apply` were not run.
- `/dev/port` was not touched.
- No raw SMBus access was performed.
- No raw Super I/O access was performed.
- No hardware access was performed.
- MS-7E75 support was not enabled.
- No MS-7E75 register map was inferred from 7A45.
- The 7A45 NCT register map was not reused for MS-7E75.

## Analyzed Binary

| Binary | Path | Size | Hash / identity | Notes |
| --- | --- | --- | --- | --- |
| `SMBus_Engine.dll` | `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib\SMBus_Engine.dll` | `1,996,816` bytes | SHA-256 `5506f54ec6731c601cd17c8a6419774491b4bc1b65bb5e8d2e4293edbc05a045`; Ghidra import MD5 `cd59f6c9cc728fb16b4014d1836ce0e1` | Actual DLL loaded into Ghidra headless and analyzed as PE32 `x86:LE:32:default:windows`. |
| `SMBus_Engine.dll` | `C:\Program Files (x86)\MSI\MSI Center\Lib\SMBus_Engine.dll` | `1,996,816` bytes | Same SHA-256 as above | Byte-identical copy; not separately analyzed. |

Ghidra memory layout for the analyzed DLL:

- `Headers`: `10000000`-`100003ff`
- `.text`: `10001000`-`1015dfff`
- `.rdata`: `1015e000`-`101afdff`
- `.data`: `101b0000`-`101be8cf`
- `.rsrc`: `101bf000`-`101c21ff`
- `.reloc`: `101c3000`-`101ee5ff`

## Export Surface

The DLL exports three families of SMBus functions:

- Default wrappers: `SMB_*` and `SMBus*`
- `b_` wrappers: likely one bus/method variant
- `n_` wrappers: same operations with a different internal mode flag

Exported default wrappers:

- `SMB_BlockRead1` at `10006f70`
- `SMB_BlockRead2` at `10006fc0`
- `SMB_BlockRead3` at `10007010`
- `SMB_BlockRead` at `10007060`
- `SMB_BlockWrite1` at `100070c0`
- `SMB_BlockWrite2` at `10007110`
- `SMB_BlockWrite3` at `10007160`
- `SMB_ByteRead` at `100071b0`
- `SMB_ByteWrite` at `10007200`
- `SMBusCheckAddress` at `10007250`
- `SMBusCheckAndReset` at `100072a0`
- `SMBusCheckWriteAddress` at `100072e0`
- `SMBusGetAddress` at `10007330`
- `SMBusInitialization` at `10007380`
- `SMBusRelease` at `100073c0`
- `SMBusReload` at `100073e0`
- `SMBus_SetSPDPage` at `10007420`

Additional `b_` exports:

- `b_SMB_BlockRead1`, `b_SMB_BlockRead2`, `b_SMB_BlockRead3`, `b_SMB_BlockRead`
- `b_SMB_BlockWrite1`, `b_SMB_BlockWrite2`, `b_SMB_BlockWrite3`
- `b_SMB_ByteRead`, `b_SMB_ByteWrite`
- `b_SMBusCheckAndReset`, `b_SMBus_SetSPDPage`

Additional `n_` exports:

- `n_SMB_BlockRead1`, `n_SMB_BlockRead2`, `n_SMB_BlockRead3`, `n_SMB_BlockRead`
- `n_SMB_BlockWrite1`, `n_SMB_BlockWrite2`, `n_SMB_BlockWrite3`
- `n_SMB_ByteRead`, `n_SMB_ByteWrite`
- `n_SMBusCheckAddress`, `n_SMBusCheckWriteAddress`, `n_SMBus_SetSPDPage`

No exported `SMB_WordRead` or `SMB_WordWrite` symbol was found in this DLL.

## Transaction Wrappers

The exported functions do not appear to perform SMBus port I/O directly. They populate a global request structure through `FUN_10008d00`, set a mode byte at object offset `2`, and call the dispatcher `FUN_100087e0`.

Default wrappers set the mode byte to `1`; `n_` wrappers set it to `0`. The dispatcher passes that mode byte as the last argument to backend virtual methods. The `b_` family was exported but not fully decompiled in this pass.

Observed operation IDs passed into `FUN_10008d00` / `FUN_100087e0`:

- `1`: byte read
- `2`: byte write
- `3`: block read 1-byte result
- `4`: block read 2-byte result
- `5`: block read larger/buffer result
- `6`: block write 1-byte payload
- `7`: block write 2-byte payload
- `8`: block write buffer payload
- `9`: block read with caller buffer/length
- `10`: get SMBus address/base information
- `0x0b`: reload/reset
- `0x0c`: check address
- `0x0d`: check write address
- `0x0e`: check/reset variant
- `0x0f`: set SPD page

Return-copy helpers:

- `FUN_10008660` copies a byte result from global `DAT_101b6fab`.
- `FUN_10008680` copies a word result from global `DAT_101b6fb4`.
- `FUN_10008610` copies `0x100` bytes from global buffer `DAT_101b6fc0` and clears that buffer afterward.

## Controller Selection Logic

`SMBusInitialization(param_1, param_2)` allocates a small global object on first call and passes its arguments to `FUN_100082d0`.

`FUN_100082d0` stores `param_1` into global `DAT_101b70cc`, stores `param_2` as a mode flag, and calls `FUN_100086a0`.

`FUN_100086a0` treats `DAT_101b70cc` as a Driver Engine-like function-pointer table:

- It calls function pointer offset `0x38` with PCI config-style arguments and magic constant `0x2f405a34`.
- If the low word of the PCI config result is `0x8086` (`(short)local_18 == -0x7f7a`), it allocates an `IntelSMBus` object.
- Otherwise it allocates an `ATISMBus` object.

The Intel path (`FUN_10007ed0`) scans PCI bus/device/function combinations using the same Driver Engine table:

- Reads PCI config through offset `0x38`.
- Looks for class/vendor-style value where `local_18 >> 16 == 0x0c05`.
- Writes PCI command register value `3` through offset `0x3c`.
- Reads BAR/config values at offsets `0x20` and `0x40`.
- Derives SMBus register addresses from the discovered base: `base - 1`, `base + 1`, `base + 2`, `base + 3`, `base + 4`, `base + 5`, `base + 6`, `base + 0x0c`, `base + 0x0d`, `base + 0x0e`.
- Checks both bus `0` and bus `0x80`, storing a bus selector at object offset `0x44`.

This strongly explains the MBAPI-to-SMBus path: MBAPI loads `SMBus_Engine.dll`, passes the already-initialized Driver Engine object into `SMBusInitialization`, and the SMBus engine uses Driver Engine PCI config and likely I/O helpers internally through a virtual SMBus backend object.

## Driver Engine / NTIOLib Linkage

Confirmed:

- `SMBus_Engine.dll` does not contain the string `Driver_Engine.dll`.
- `SMBus_Engine.dll` does not contain the string `NTIOLib_MysticLight`.
- `SMBus_Engine.dll` imports `CreateFileW`, `LoadLibraryA/W`, `GetProcAddress`, `CloseHandle`, and mutex/security APIs, but no static `DeviceIoControl` import was found in the dump.
- Driver Engine linkage is through the `SMBusInitialization` argument, not through this DLL loading `Driver_Engine.dll` itself.
- The same magic constant `0x2f405a34` appears in PCI config calls made through the incoming Driver Engine function-pointer table.

Unknown:

- Exact Driver Engine object type and lifetime.
- Whether `SMBus_Engine.dll` ever uses imported `CreateFileW` in SMBus-specific code or only through linked MFC/file support.
- Exact backend method names for the `IntelSMBus` and `ATISMBus` vtables.

## Board / Profile / Renesas Strings

Focused string searches in the analyzed `SMBus_Engine.dll` found:

- No `MS-7E75`
- No `7E75`
- No meaningful `B850` board string; the only `B850` matches were unrelated MFC/code addresses or UI/resource text.
- No `JRGB`
- No `JRAINBOW`
- No `0x52` string
- No `Driver_Engine`
- No `NTIOLib`
- No `Nuvoton`
- No `IT8295`

Renesas-related strings found:

- `Global\Access_SMBUS.HTP.Renesas.Method` at `1015eff0`
- `Access_SMBUS.HTP.Renesas.Method Error Code : %X` at `1015f040`

The Renesas evidence is synchronization/logging-oriented. This DLL provides generic SMBus byte/block primitives that MBAPI can use for Renesas LED controller traffic, but this pass did not find hard-coded Renesas LED addresses, board IDs, or JRGB/JRAINBOW header labels inside `SMBus_Engine.dll` itself.

## Evidence Table

| Binary / module | Function / artifact | Address | Evidence | Why it matters | Confidence | Suggested path |
| --- | --- | --- | --- | --- | --- | --- |
| `SMBus_Engine.dll` | Program identity | N/A | Ghidra loaded `/C:/Program Files (x86)/MSI/MSI Center/Mystic Light/Lib/SMBus_Engine.dll`, PE32 x86, image base `10000000` | Confirms the actual SMBus Engine DLL was loaded/analyzed, not only MBAPI references. | High | Static Ghidra analysis |
| `SMBus_Engine.dll` | File hash | N/A | SHA-256 `5506f54ec6731c601cd17c8a6419774491b4bc1b65bb5e8d2e4293edbc05a045`; MSI Center `Lib` copy is byte-identical | Establishes artifact identity and duplicate copy equivalence. | High | Static artifact identity |
| `SMBus_Engine.dll` | Exports | `10006f70`-`10007dc0` | Exports `SMB_*`, `b_SMB_*`, and `n_SMB_*` byte/block/check/SPD functions | Confirms this is the companion DLL MBAPI dynamically resolves for SMBus operations. | High | MBAPI -> SMBus Engine |
| `SMBus_Engine.dll` | Imports | `EXTERNAL:0000005c`, `00000071`, `0000008a`, `00000090` | Imports `LoadLibraryA`, `LoadLibraryW`, `GetProcAddress`, `FreeLibrary` | Dynamic loading support exists, but no Driver Engine DLL string was found. | Medium | Generic/MFC or helper loading |
| `SMBus_Engine.dll` | Imports | `EXTERNAL:0000006d` | Imports `CreateFileW` | File/handle opening exists in the binary, but SMBus-specific device-open usage was not confirmed. | Low | Unknown |
| `SMBus_Engine.dll` | Imports / strings | N/A | No `DeviceIoControl`, `Driver_Engine`, or `NTIOLib_MysticLight` hit | Suggests direct IOCTL and NTIOLib naming are below/elsewhere, likely in Driver Engine. | Medium | Driver Engine |
| `SMBus_Engine.dll` | `SMBusInitialization` | `10007380` | Allocates a global object and calls `FUN_100082d0(param_1, param_2)` | Entry point for MBAPI to initialize SMBus engine with a Driver Engine-like object. | High | MBAPI -> SMBus Engine |
| `SMBus_Engine.dll` | `FUN_100082d0` | `100082d0` | Stores `param_1` in `DAT_101b70cc`, stores mode flag, calls `FUN_100086a0` | Shows the initialization argument becomes the hardware-access table used later. | High | SMBus Engine -> Driver Engine table |
| `SMBus_Engine.dll` | `FUN_100086a0` | `100086a0` | Calls table offset `0x38` with PCI-config-like args and `0x2f405a34`; selects `IntelSMBus` if low word is `0x8086`, otherwise `ATISMBus` | Controller selection logic is chipset/vendor-based, not board-string-based. | High | Driver Engine PCI config |
| `SMBus_Engine.dll` | `FUN_10007ed0` | `10007ed0` | Intel path scans PCI config for class `0x0c05`, enables command register, reads BAR/config offsets, derives SMBus register addresses | Explains how the DLL discovers Intel SMBus base/addressing using Driver Engine PCI config helpers. | High | Driver Engine PCI config -> SMBus |
| `SMBus_Engine.dll` | `FUN_10008bc0` | `10008bc0` | Refreshes/rescans Intel SMBus PCI info and base-derived register offsets | Likely reload/reset path for Intel SMBus backend. | Medium | Driver Engine PCI config |
| `SMBus_Engine.dll` | `FUN_100087e0` | `100087e0` | Dispatches operation IDs `1`-`0x0f` to backend virtual methods, protected by mutex `DAT_101b70c8` | Core transaction dispatcher behind exported byte/block functions. | High | SMBus backend vtable |
| `SMBus_Engine.dll` | `SMB_ByteRead` | `100071b0` | Sets operation ID `1`, mode byte `1`, executes dispatcher, copies byte result | Exported default byte read wrapper. | High | SMBus transaction wrapper |
| `SMBus_Engine.dll` | `SMB_ByteWrite` | `10007200` | Sets operation ID `2`, mode byte `1`, executes dispatcher | Exported default byte write wrapper. | High | SMBus transaction wrapper |
| `SMBus_Engine.dll` | `SMB_BlockRead*` | `10006f70`, `10006fc0`, `10007010`, `10007060` | Set operation IDs `3`, `4`, `5`, and `9` for byte/word/buffer block read variants | Confirms block-read API family. | High | SMBus transaction wrapper |
| `SMBus_Engine.dll` | `SMB_BlockWrite*` | `100070c0`, `10007110`, `10007160` | Set operation IDs `6`, `7`, and `8` for byte/word/buffer block write variants | Confirms block-write API family. | High | SMBus transaction wrapper |
| `SMBus_Engine.dll` | `n_SMB_ByteRead`, `n_SMB_ByteWrite`, `n_SMB_BlockRead`, `n_SMBus_SetSPDPage` | `10007c80`, `10007cd0`, `10007b30`, `10007dc0` | Same operation IDs as default wrappers but set mode byte `0` | Indicates alternate transaction mode exposed to MBAPI as `n_` functions. | Medium | SMBus transaction wrapper |
| `SMBus_Engine.dll` | `SMBusGetAddress` | `10007330` | Operation ID `10`, copies word result through `FUN_10008680` | Exposes discovered SMBus address/base data back to caller. | High | SMBus base/address query |
| `SMBus_Engine.dll` | `SMBusReload` | `100073e0` | Operation ID `0x0b`, executes dispatcher | Exposes backend reload/reset behavior. | Medium | SMBus backend reload |
| `SMBus_Engine.dll` | `SMBus_SetSPDPage` | `10007420` | Operation ID `0x0f`, passes page/address inputs into dispatcher | SPD page support is present, likely for DRAM/SPD clients. | Medium | SMBus SPD |
| `SMBus_Engine.dll` | Mutex strings | `1015ef38`, `1015eff0` | `Global\Access_SMBUS.HTP.Method` and `Global\Access_SMBUS.HTP.Renesas.Method` | Synchronization names show a generic SMBus method and a Renesas-labeled method. | High | SMBus/Renesas synchronization |
| `SMBus_Engine.dll` | Log string | `1015eeac` | `C:\MSI\SMBus_Engine.log` | Logging path for this DLL. | Medium | Diagnostics |
| `SMBus_Engine.dll` | RTTI strings | `101b00dc`, `101b00f8` | `.?AVIntelSMBus@@`, `.?AVATISMBus@@` | Confirms two backend classes identified by decompiler/vtable selection. | High | Controller backend classes |
| `SMBus_Engine.dll` | Board/header string searches | N/A | No `MS-7E75`, `7E75`, `JRGB`, `JRAINBOW`, or meaningful `B850` string | Board/profile dispatch does not appear to live in this DLL as plain text. | Medium | Unknown / MBAPI profile layer |

## Confirmed vs Unknown

Confirmed:

- The actual `SMBus_Engine.dll` was loaded and analyzed statically in Ghidra.
- The DLL exports the SMBus byte/block/check/SPD functions MBAPI referenced.
- The exported transaction functions are wrappers over a global request object and a backend dispatcher.
- `SMBusInitialization` receives and stores a Driver Engine-like object from its caller.
- SMBus controller selection uses PCI config reads through that incoming object, not board strings.
- Intel SMBus discovery looks for PCI class `0x0c05` and derives register addresses from a discovered base.
- The DLL contains Renesas-labeled mutex/log strings but no hard-coded Renesas LED address strings found in this pass.
- No MS-7E75 hardware access was enabled or run.

Unknown:

- Exact names and semantics of every backend virtual method.
- Whether `b_` exports correspond to a specific bus lock, bus selector, or Renesas-specific synchronization mode.
- Whether the `param_2` mode passed into `SMBusInitialization` maps to default versus `n_` transaction behavior.
- Exact SMBus register write/read sequence inside `IntelSMBus` and `ATISMBus` methods beyond the dispatcher and PCI discovery layer.
- Whether MS-7E75 uses the SMBus/Renesas path; this DLL contains generic primitives but no MS-7E75 board mapping.
- Whether MBAPI or another MSI profile layer chooses when to use `SMB_*`, `b_SMB_*`, or `n_SMB_*` for a given controller.

## Open Questions

- Which caller passes the `param_2` mode byte to `SMBusInitialization`, and what does it select?
- What are the full `IntelSMBus` and `ATISMBus` vtable method mappings?
- Do `b_` exports acquire the `Global\Access_SMBUS.HTP.Renesas.Method` mutex differently from default exports?
- Which MBAPI functions call `b_SMB_*` versus `SMB_*` versus `n_SMB_*`?
- Does the Renesas LED controller path live entirely in MBAPI, using this DLL only as a generic SMBus transaction engine?
- Does MS-7E75 choose this path, or does it use EC/USB/HID/another MSI service path?
- Can `Driver_Engine.dll` confirm the PCI config and I/O port helper offsets used by this DLL?

## Next Static-Analysis Tasks

- Decompile the `b_SMB_*` exports and compare their mode/mutex behavior with default and `n_` wrappers.
- Build a vtable map for `SMBus`, `IntelSMBus`, and `ATISMBus`.
- Decompile Intel/ATI backend byte/block read/write methods to identify exact SMBus status/control/data register sequences.
- Cross-reference MBAPI callers of `SMB_*`, `b_SMB_*`, and `n_SMB_*` to identify which path is used for Renesas LED traffic.
- Load `Driver_Engine.dll` directly and confirm offsets `0x38` and `0x3c` as `PCIConfigRead` / `PCIConfigWrite`.
- Search additional MSI Center profile/config data for MS-7E75 / B850 / JRGB / JRAINBOW routing.

## Explicit Hardware-Access Note

No hardware access was enabled or run during this investigation. MSI Center and Mystic Light were not launched. `cargo run -- doctor` was not run. No `/dev/port` command, raw SMBus access, raw Super I/O access, chip-detection command, register-read command, write command, apply command, or board-control hardware support code was run or added.
