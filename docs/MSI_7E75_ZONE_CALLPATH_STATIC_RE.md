# MSI MS-7E75 Zone Call-Path Static Reverse Engineering Notes

## Scope

This document records a static-only follow-up pass from decoded MS-7E75 Mystic Light zones to the managed LEDKeeper device-control path.

Target question:

- Which decompiled LEDKeeper classes consume `MS-7E75_1_JRGB1`, `MS-7E75_1_JARGB_V2_1`, `MS-7E75_1_JARGB_V2_2`, `MS-7E75_1_JARGB_V2_3`, `MS-7E75_1_EZ Conn`, and `MS-7E75_1_SELECT ALL`?
- Do those zones map to `MSI_LED.MB` / `MBAPI_x86.dll` P/Invokes, or to another static helper path?
- Are any concrete backend parameters, reports, addresses, or register maps visible?

The pass used existing ILSpy decompilation output for `LEDKeeper2.exe` and static decompilation of related managed assemblies including `MLModule.dll`, `SyncData.dll`, and `MysticLight_AllDevice.dll`. No MSI binary was executed.

## Safety Constraints

- Static analysis and documentation only.
- Do not run MSI Center, Mystic Light, `LEDKeeper2.exe`, or MSI binaries.
- Do not run doctor.
- Do not run detect-chip, read-reg, write, or apply.
- Do not touch `/dev/port`.
- Do not perform raw SMBus or Super I/O access.
- Do not enable MS-7E75 hardware access.
- Do not claim an MS-7E75 transport or register map unless the evidence is specific.

## Inputs And Evidence Used

| Input | Static evidence used | Relevance |
| --- | --- | --- |
| Decoded `Mystic Light Online Data.dat` `[SyncData]` | `MS-7E75_1` and `MS-7E75_2` records with `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, `EZ Conn`, and `SELECT ALL`. | Source of the MS-7E75 zone list and tuple fields. |
| `MLModule.CLEDParser` | `VerifySupportDevice` parses motherboard records into `PartItem` fields: `ShowName`, `DeviceName`, `Chipest`, `Style`, `EntryAddress`, `StyleSelectIndex`, colors, speed, brightness, and filters. | Explains how decoded tuples become runtime zone items. |
| `MLModule.CTransferFun` | Converts tuple fields from hex/text into `EnumDeviceName`, `EnumChipest`, `EnumStyle`, entry-address lists, speed, brightness, and filter styles. | Decodes `09/00/01/02/03/11` device codes and `69` chipset code. |
| `SyncData.EnumDeviceName` | Contains MB800 aliases such as `MB_800_JARGB1`, `MB_800_JARGB2`, `MB_800_JARGB3`, `MB_800_JAF`, `MB_800_JRGB1`, and `MB_800_SelectAll`. | Maps MS-7E75 zone tuple device IDs to MB800 LED areas in MB800 context. |
| `SyncData.EnumChipest` | `0x69` / decimal `105` resolves to `NUC126_MB800`. | Static tie from MS-7E75 decoded profile records to `Class_MB_800`. |
| `MSI_LED.Class_MB_800` | `SetStyle` filters `CLEDParser.List_PartItem` entries where `Chipest[0] == EnumChipest.NUC126_MB800`, builds `DeviceID = MainDevice + "_" + ShowName`, and calls `MSI_800sLed` methods. | Main MS-7E75 zone call-path candidate. |
| `MysticLight_AllDevice.Device.MB_800.MSI_800sLed` | Defines `Enum_LedArea`, Gen1/Gen2 setting structs, `Gen1_SetArea`, `Gen1_ApplyBoard`, `Gen2_SetStrip`, and `Gen2_ApplyPort`; apply methods call `HID_Basic.SetFeature`. | Concrete helper layer below `Class_MB_800`. |
| `MSI_LED.MB` | Managed P/Invoke wrapper for `Lib\MBAPI_x86.dll`, including LED, Renesas, SMBus, EC, and SIO methods. | Generic MBAPI surface exists, but no static MS-7E75 zone path was found from these decoded MB800 zones into `MSI_LED.MB`. |

## Zone List

Decoded MS-7E75 `[SyncData]` records define this motherboard zone set:

| Profile device name | Tuple evidence | Decoded static meaning |
| --- | --- | --- |
| `MS-7E75_1_JRGB1` | `JRGB1,09,69,1342D02C23469345A74401,,10,FF0000,3+2,5+5,+1301` | `ShowName=JRGB1`, `DeviceName=0x09`, `Chipest=0x69/NUC126_MB800`; MB800 `Enum_LedArea.JRGB1`. |
| `MS-7E75_1_JARGB_V2_1` | `JARGB_V2_1,00,69,...` | MB800 Gen2 ARGB port 0 / `Enum_LedArea.JARGB1` candidate. |
| `MS-7E75_1_JARGB_V2_2` | `JARGB_V2_2,01,69,...` | MB800 Gen2 ARGB port 1 / `Enum_LedArea.JARGB2` candidate. |
| `MS-7E75_1_JARGB_V2_3` | `JARGB_V2_3,02,69,...` | MB800 Gen2 ARGB port 2 / `Enum_LedArea.JARGB3` candidate. |
| `MS-7E75_1_EZ Conn` | `EZ Conn,03,69,...` | MB800 Gen2 ARGB port 3 / `Enum_LedArea.JAF` candidate. |
| `MS-7E75_1_SELECT ALL` | `SELECT ALL,11,69,...` | MB800 `Enum_LedArea.SelectAll` in MB800 context. |

The common style mask `1342D02C23469345A74401` parses as `EnumStyle` bytes `0x13`, `0x42`, `0xD0`, `0x2C`, `0x23`, `0x46`, `0x93`, `0x45`, `0xA7`, `0x44`, `0x01`. The tuple default style index is `10`, default color is `FF0000`, speed is `3+2`, brightness is `5+5`, and filter suffix is `+1301`. These are LEDKeeper profile/effect fields, not hardware registers.

## Method / Call-Path Evidence Table

| Method / class | Static call-path evidence | MS-7E75 zone relevance | Backend/register conclusion |
| --- | --- | --- | --- |
| `CLEDParser.VerifySupportDevice` | For motherboard records, splits each zone tuple and assigns `ShowName`, `DeviceName`, `Chipest`, style list, default color, speed, brightness, and filters into `PartItem`. | Converts decoded `MS-7E75_1` zone records into `CLEDParser.List_PartItem`. | Parser only; no hardware backend. |
| `CTransferFun.DeviceName` | Parses tuple field 2 as hex bytes and casts values to `EnumDeviceName`. | `09/00/01/02/03/11` become MB800 area identifiers in an MB800 chipset context. | Device enum values are logical IDs, not registers. |
| `CTransferFun.Chipest` | Parses tuple field 3 as hex bytes and casts values to `EnumChipest`. | `69` resolves to decimal `105`, `EnumChipest.NUC126_MB800`. | This is the strongest static selector tying MS-7E75 profile records to `Class_MB_800`. |
| `Class_MB_800.SetStyle` | Iterates `CLEDParser.List_PartItem`, filters `Chipest[0] == EnumChipest.NUC126_MB800`, builds `DeviceID = MainDevice + "_" + ShowName`, finds matching profile `DeviceData`, and constructs `MSI_800sLed.Struct_Gen1AreaSetting`. | Consumes `MS-7E75_1_JRGB1`, `..._JARGB_V2_*`, `..._EZ Conn`, and `..._SELECT ALL` as profile device IDs. | Static path points to MB800 helper calls, not direct `MSI_LED.MB` calls. |
| `Class_MB_800.SetStyle` -> `MSI_800sLed.Gen1_SetArea` | Calls `Gen1_SetArea((Enum_LedArea)PartItem.DeviceName[0], AreaSetting)` for matching MB800 zones, then later applies the board if settings changed. | `JRGB1` maps to `Enum_LedArea.JRGB1`; `SELECT ALL` can drive all MB800 areas through select-all handling. | Stores Gen1 area settings in managed state before apply. |
| `Class_MB_800.SetStyle` -> `MSI_800sLed.Gen1_ApplyBoard(save)` | After Gen1 settings change, obtains mutex `Global\Access_SMBUS.HTP.Method` and calls `Gen1_ApplyBoard(save)`. | Applies Gen1 board data for `JRGB1` and other Gen1 MB800 areas. | The mutex name is not proof of SMBus transport; the lower helper calls `HID_Basic.SetFeature`. |
| `Class_MB_800.GetCycleNumber` | Checks `DeviceID.Contains("JARGB_V2_1")`, `_2`, `_3`, and `EZ Conn`, maps them to ports 1..4, and returns Gen2 LED counts or registry cycle-number fallbacks. | Static name-to-port evidence for MS-7E75 JARGB V2 zones and `EZ Conn`. | Logical port mapping only. |
| `Class_MB_800.UpdateJARGB_V2_Basic` | For Gen2 support, reads `App.listFixIDJARGBGen2[port]` and `App.listLEDNumJARGBGen2[port]`, or parses `MB_JARGB_V2_Info{port+1}` records into `Struct_Gen2StripSetting`, then calls `Gen2_SetStrip` and `Gen2_ApplyPort`. | Handles `JARGB_V2_1` port 0, `_2` port 1, `_3` port 2, and `EZ Conn` port 3. | Constructs strip payload fields, still not a register map. |
| `Class_MB_800.SetStyle` advanced Gen2 branch | Writes registry `MB_JARGB_V2 = SetJARGB_V2_1/2/3/4` for advanced changes, reads `MB_JARGB_V2_Info1/2/3/4`, and calls `UpdateJARGB_V2_Basic` when needed. | `SetJARGB_V2_4` is logged as the `JAF` / `EZ Conn` advanced path. | Registry/profile path only; not hardware proof by itself. |
| `MSI_800sLed.Gen1_SetArea` | Stores a `Struct_Gen1AreaSetting` in `LedSettings_Gen1[(int)Area]`. | Area enum includes `JARGB1`, `JARGB2`, `JARGB3`, `JAF`, `JRGB1`, and `SelectAll`. | No I/O until apply. |
| `MSI_800sLed.Gen1_ApplyBoard` | Builds a 290-byte feature buffer with `array[0] = 80`, 18 area records of lighting mode, four RGB colors, option bytes, cycle number, and final store byte; calls `HID_Basic.SetFeature(_Device, array, array.Length)`. | Concrete helper payload shape for Gen1 MB800 area apply. | Static HID feature-buffer evidence; no MS-7E75-specific register/address map. |
| `MSI_800sLed.Gen2_SetStrip` | Stores `Struct_Gen2StripSetting` for a target port and fixed strip ID. | Used for JARGB V2 strips from `UpdateJARGB_V2_Basic`. | No I/O until apply. |
| `MSI_800sLed.Gen2_ApplyPort` | Builds a 302-byte feature buffer initialized to `0xFF`, sets `array[0] = 144 + Port`, writes fixed ID, lighting mode, four colors, option bytes, LED count per strip, final store byte, and calls `HID_Basic.SetFeature`. | Concrete helper payload shape for JARGB V2 ports 0..3. | Static HID feature-buffer evidence; report bytes are not sufficient to claim a register map. |
| `MSI_LED.MB` | P/Invokes `Lib\MBAPI_x86.dll` methods such as `LEDControl`, `LEDMysticControl`, `RenesasLEDControlV3`, `SMBusControl`, `SetECSpace`, and `SetSIOGPIO`. | Generic motherboard LED API surface remains present in LEDKeeper. | No static path found from MS-7E75 MB800 zones to these methods in this pass. |

Follow-up HID details are documented in [MSI_7E75_HID_MB800_STATIC_RE.md](MSI_7E75_HID_MB800_STATIC_RE.md). That pass confirms the primary LEDKeeper MB800 path is `MSI_LED.MSI_800sLed -> MsiHid.HID_Basic -> Lib\MsiHid.dll`, with native static imports including `HidD_SetFeature`, `HidD_GetFeature`, SetupAPI enumeration helpers, `CreateFileW`, `ReadFile`, and `WriteFile`.

## Candidate Zone-To-Call-Target Mapping

This is the current static mapping from decoded MS-7E75 profile zones to decompiled managed call targets. It is a call-target map, not a register map.

| MS-7E75 profile zone | Decoded fields | `Class_MB_800` path | Lower helper target | Candidate payload selector |
| --- | --- | --- | --- | --- |
| `MS-7E75_1_JRGB1` | Device `0x09`, chipset `0x69/NUC126_MB800` | `SetStyle` -> `Gen1_SetArea(Enum_LedArea.JRGB1, AreaSetting)` -> `Gen1_ApplyBoard(save)` | `MSI_800sLed.Gen1_ApplyBoard` -> `HID_Basic.SetFeature` | 290-byte Gen1 buffer, `array[0] = 0x50`; area index `9`. |
| `MS-7E75_1_JARGB_V2_1` | Device `0x00`, chipset `0x69/NUC126_MB800` | `SetStyle` -> `UpdateJARGB_V2_Basic(0, ...)` -> `Gen2_SetStrip/Gen2_ApplyPort` | `MSI_800sLed.Gen2_ApplyPort(Enum_TargetPort.JARGB1, save)` -> `HID_Basic.SetFeature` | 302-byte Gen2 buffer, `array[0] = 0x90`; port index `0`. |
| `MS-7E75_1_JARGB_V2_2` | Device `0x01`, chipset `0x69/NUC126_MB800` | `SetStyle` -> `UpdateJARGB_V2_Basic(1, ...)` -> `Gen2_SetStrip/Gen2_ApplyPort` | `MSI_800sLed.Gen2_ApplyPort(Enum_TargetPort.JARGB2, save)` -> `HID_Basic.SetFeature` | 302-byte Gen2 buffer, `array[0] = 0x91`; port index `1`. |
| `MS-7E75_1_JARGB_V2_3` | Device `0x02`, chipset `0x69/NUC126_MB800` | `SetStyle` -> `UpdateJARGB_V2_Basic(2, ...)` -> `Gen2_SetStrip/Gen2_ApplyPort` | `MSI_800sLed.Gen2_ApplyPort(Enum_TargetPort.JARGB3, save)` -> `HID_Basic.SetFeature` | 302-byte Gen2 buffer, `array[0] = 0x92`; port index `2`. |
| `MS-7E75_1_EZ Conn` | Device `0x03`, chipset `0x69/NUC126_MB800` | `SetStyle` -> `UpdateJARGB_V2_Basic(3, ...)`; advanced branch logs `Set JAF Advanced` and writes `SetJARGB_V2_4` | `MSI_800sLed.Gen2_ApplyPort(Enum_TargetPort.JAF, save)` -> `HID_Basic.SetFeature` | 302-byte Gen2 buffer, `array[0] = 0x93`; port index `3`. |
| `MS-7E75_1_SELECT ALL` | Device `0x11`, chipset `0x69/NUC126_MB800` | `SetStyle` select-all handling touches MB800 areas and may set default/GodLike/MLG/default generated settings. | `MSI_800sLed.Gen1_SetArea` plus `Gen1_ApplyBoard`; JARGB V2 select-all can feed `UpdateJARGB_V2_Basic` per port. | Logical aggregate selector; no separate concrete backend/register map found. |

## Confirmed Vs Unknown

Confirmed:

- Static profile data defines the MS-7E75 zones listed above.
- `CLEDParser` and `CTransferFun` parse those zone tuples into `PartItem` objects.
- Tuple chipset byte `69` resolves to `EnumChipest.NUC126_MB800`.
- In MB800 context, tuple device bytes map to MB800 logical LED areas/ports.
- `Class_MB_800.SetStyle` consumes `NUC126_MB800` `PartItem` entries and dispatches to `MSI_800sLed` helper methods.
- `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, and `EZ Conn` map to Gen2 ports 0, 1, 2, and 3 respectively in `Class_MB_800`.
- `MSI_800sLed.Gen1_ApplyBoard` and `Gen2_ApplyPort` construct HID feature buffers and call `HID_Basic.SetFeature`.
- No direct static call path was found from decoded MS-7E75 MB800 zones to `MSI_LED.MB` P/Invokes such as `RenesasLEDControlV3`, `LEDControl`, `LEDMysticControl`, `SMBusControl`, `SetECSpace`, or `SetSIOGPIO`.

Unknown:

- Whether the installed/live MS-7E75 path always reaches `Class_MB_800`; this static pass proves the decoded profile records select `NUC126_MB800`, not that runtime initialization succeeds.
- The native `MsiHid.dll` filtering logic for MI/COL, usage page, usage, collection, and device path selection.
- Whether MBAPI's separate static `7E75` board-list entry participates before, beside, or independently of the MB800 profile path.
- The exact controller-side semantics of the HID feature reports below `HID_Basic.SetFeature`.
- Any MS-7E75 SMBus address, EC offset, SIO register, raw IOCTL sequence, or register map.

## Next Static-Only Targets

- Continue native static analysis of `Lib\MsiHid.dll` around `openMyDevice_Read`, `SetFeature`, and `GetAllDevicesID`.
- Cross-reference `MSI_800sLed.CheckConnectedDevice`, `Init`, and support lists with the known `7E75` board ID without opening devices.
- Continue MBAPI static analysis around the `7E75` board-list hit to determine whether it is a separate support gate or unrelated to the MB800 LED path.
- Inspect `Class_MB_800.Initial`, `RGBControlClass.updateSupportedDevice`, and startup support-list construction for a complete static initialization chain into `Class_MB_800`.
- Keep `SMBus_Engine.dll`, `Driver_Engine.dll`, EC, and SIO paths marked unproven for MS-7E75 unless a future static cross-reference ties them to the decoded `NUC126_MB800` zones.

## Explicit Hardware-Access Note

No MSI binaries were executed during this pass. MSI Center, Mystic Light, and `LEDKeeper2.exe` were not launched. No hardware access was enabled or run: no doctor command, no chip detection, no register read, no write/apply command, no `/dev/port`, no raw SMBus, no raw Super I/O, and no MS-7E75 hardware support code changes.
