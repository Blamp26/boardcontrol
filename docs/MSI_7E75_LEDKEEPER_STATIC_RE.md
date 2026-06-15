# MSI MS-7E75 LEDKeeper2 Static Reverse Engineering Notes

Status: static analysis only, hardware access disabled.

## Scope

This document records static-only analysis of:

```text
C:\Program Files (x86)\MSI\MSI Center\Mystic Light\LEDKeeper2.exe
```

The goal was to inspect profile/zone dispatch logic for MSI MS-7E75 / B850 GAMING PLUS WIFI PZ without executing MSI software or touching hardware. The pass focused on `MSI_LED.MB`, `RGBControlClass`, `Class_Fun_MB`, `Class_ParseCfg`, `MSI_7B10Led`, `Class_MB_800`, and nearby profile/control classes required to explain profile loading and `ResetItem` logs.

## Safety Constraints

- Documentation only.
- Static analysis only.
- MSI Center was not run.
- Mystic Light was not run.
- `LEDKeeper2.exe` was not run.
- `cargo run -- doctor` was not run.
- `detect-chip`, `read-reg`, `write`, and `apply` were not run.
- `/dev/port` was not touched.
- No raw SMBus access was performed.
- No raw Super I/O access was performed.
- MS-7E75 hardware access was not enabled.
- No MS-7E75 transport or register map is claimed unless evidence is specific.
- No MS-7E75 behavior is inferred from the existing 7A45 profile.

## Tooling And Method

| Tool/method | Use |
| --- | --- |
| `Get-FileHash` | SHA-256 and file metadata for `LEDKeeper2.exe`. |
| Reflection-only .NET metadata | Assembly identity, type names, methods, fields, resources, and P/Invoke metadata without executing the target assembly. |
| `ilspycmd` 10.1.0.8386 | Static C# decompilation of selected .NET types. Installed into local `.tools/` and not committed. |
| Raw ASCII/UTF-16 string searches | Negative/positive confirmation for board IDs, profile strings, and path strings. |

No MSI binary was launched. ILSpy read the assembly as data.

## Analyzed Binary And Hash

| Field | Value |
| --- | --- |
| Path | `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\LEDKeeper2.exe` |
| Size | 3,541,064 bytes |
| SHA-256 | `990C65F31038AA6DCA39ABBE33735E42424B37696FB56D5B58D6EEA05FBB8159` |
| Last write time | `2026-04-30 20:54:06` local |
| PE machine | `0x14c` / x86 |
| .NET assembly | `LEDKeeper2, Version=3.0.32.2, Culture=neutral, PublicKeyToken=null` |
| Target framework string | `.NETFramework,Version=v4.8` |
| Native import table | `mscoree.dll!_CorExeMain` only |
| CLR header | Present |
| Manifest resources | `LEDKeeper2.g.resources`, `MSI_LED.Properties.Resources.resources` |

## Class/Method Evidence Table

| Class/method | Decompile evidence | Relevance | Limitations |
| --- | --- | --- | --- |
| `MSI_LED.MB` | Internal static P/Invoke wrapper with `MB_DLL_FileName = "Lib\\MBAPI_x86.dll"` and 94 imported methods. | Confirms LEDKeeper routes motherboard LED/support, Renesas, SMBus, EC, SIO, DRAM, Realtek SSD, and Sonix DDR calls through MBAPI. | Generic MBAPI boundary only; no MS-7E75-specific transport/register map. |
| `Class_Fun_MB.GetWMI` | Parses MSI board strings from WMI/system product data. If product contains `(MS-...)`, sets `MB_Info.Market` to text before `(MS-...)` and `MB_Info.Product` to the `MS-...` token. | Static source for `MB_Info.Product = MS-7E75` and market name when WMI/DMI string is `B850 GAMING PLUS WIFI PZ (MS-7E75)`. | Uses runtime OS/WMI data; no hardware register proof. |
| `Class_Fun_MB.Compare_Support_MB` | Checks `App.ParseCfg.List_SyncData` for either `WMI_MB_Info.Product + "_" + WMI_MB_Info.Version.Substring(0, 1)` or `MB_Info.Market`. | Explains how online/config `SyncData` can gate board support before profile token construction. | Requires decoded online data at runtime; this pass did not decode an MS-7E75 record. |
| `Class_Fun_MB.Init_MB` | Calls `MB.CheckMBVersion(MB_Info.Product, MB_Info.Version, MB_Info.Market, "T")` and stores `Init_MB_Status`. | Confirms MBAPI receives product/version/market for motherboard initialization/support. | Does not show MBAPI's internal 7E75 dispatch. |
| `Class_Fun_MB` Renesas wrappers | Methods such as `LEDMysticControlV2`, `SetMysticBreathingModeV2`, `SetMysticLEDColorV2`, and `SetMysticRainbowModeV2` call `RenesasLEDControlV3` with fixed command-like numeric values. | Confirms a generic Renesas-style MBAPI wrapper family exists in LEDKeeper. | Not tied to MS-7E75. Numeric arguments are not a board register map. |
| `App` startup support flow | Calls `ParseCfg.ParseCfgFile()`, `Compare_Support_MB()`, `Init_MB()`, builds `TestData`, logs `Support list : ...`, initializes profile state, and starts registry watchers. | Confirms support-list logging and profile setup are LEDKeeper code paths. | Runtime inputs decide the actual list. |
| `App` profile-token construction | Generic code adds either `MB_Info.Product + "_" + MB_Info.Version.Substring(0, 1) + "_" + MB_Info.Market` for special matches or `MB_Info.Product + "_" + MB_Info.Version.Substring(0, 1)` otherwise. | Explains how `MS-7E75_1` can be generated indirectly from `MB_Info.Product = MS-7E75` and version beginning with `1`, without a cleartext `MS-7E75_1` literal. | Token construction is profile/support evidence, not backend/register evidence. |
| `App.mbID` | Static `ushort mbID`; startup sets it with `Convert.ToUInt16(MB_Info.Product.Substring(MB_Info.Product.IndexOf("MS-") + 3), 16)`. | Explains how `7E75` can be generated indirectly from `MS-7E75`. | Does not prove that `7E75` passes later LEDKeeper enum/device dispatch. |
| `RGBControlClass.updateSupportedDevice` | Searches incoming supported-device data for `MS-7`, logs `[RGBControlClass] mbID ` plus four hex digits, converts those four digits to `ushort`, then dispatches through `MSI_7B10Led.SupportList_CommonID`, `MSI_7B10Led.SupportList`, `MSI_7D26Led.SupportList`, or `MSI_800sLed.CheckConnectedDevice`. | Static source for `[RGBControlClass] mbID 7E75` log style. | `MS_7E75` is absent from visible LEDKeeper support enums, so this does not prove a 7E75 LEDKeeper device path. |
| `RGBControlClass.Init_MB_Adv_v1` | Initializes `App.Device_7B10` or `App.Device_7D26`, checks `IsSupportJARGB_V2`, optionally runs `JARGB_V2_OnlyDetect` and `JARGB_SwitchToGen1` for ports `0..2`, reads current LED settings, and sets default style/sync state. | Candidate modern motherboard MCU/JARGB V2 init path. | Not reached for MS-7E75 unless a static/runtime source maps it into the supported enum/device path. |
| `RGBControlClass.Init_MB_Adv_v2` | Uses `MSI_800sLed` Gen2 target ports, `Gen2_Detect`, `Gen2_SetEnableGen2(false)`, and Gen1 board settings. | Candidate MB800/common-device Gen2 ARGB path. | No MS-7E75 tie found. |
| `MSI_7B10Led` support enums | `SupportList_CommonID`, `SupportList`, `SupportList_MixEffect`, and `SupportList_JARGB_V2` include many `7Dxx` and `7Exx` IDs, including nearby `MS_7E74 = 32372`. | Confirms static dispatch tables exist and include modern board IDs. | No `MS_7E75 = 32373` entry was found. |
| `MSI_7B10Led.IsDeviceConnect` | For `SupportList_CommonID`, opens common HID VID/PID `3504,118` and compares the HID serial-number prefix to the requested board ID. For `SupportList`, opens MSI VID `5218` with PID equal to the board ID. | Explains why some board IDs are common-device dispatch rather than direct PID only. | HID/open behavior was only read from static code; it was not run. No MS-7E75 entry. |
| `MSI_7B10Led.CheckSupportMethod` | Similar common-ID/direct-ID static logic; reads HID feature data buffers. | Candidate table consumer for board support method selection. | Not an MS-7E75 proof. |
| `MSI_7B10Led.IsSupportJARGB_V2` | Checks whether `PID` is in `SupportList_JARGB_V2`, with a special `7D36`/`Z790` fallback. | Static JARGB V2 support gate. | `MS_7E75` absent. |
| `Class_ParseCfg.ParseCfgFile` | Reads MSI Center `Component\SDK\WorkDir`, selects `Data\Mystic Light Online Data.dat` or `Mystic Light\Mystic Light Online Data.dat`, strips the first 7 characters, decrypts with `C_Encrypt.DecryptBase64(..., 232345599.ToString("X"))`, and extracts `[Motherboard]`, `[Graphics]`, `[GraphicsNumber]`, and `[SyncData]` sections. | Confirms online data is a real static input for support/profile selection and identifies the decode function/key expression. | The decrypted installed blob was not decoded in this pass; no MS-7E75 record extracted here. |
| `ProfileFunction` | `SavePath` is Windows-drive root plus `MSI\MSI Center\Mystic Light\Profile\`; `SaveName = "Profile_v2.txt"`. `LoadProfile` creates/reads JSON profile data. `CheckProfile` and `GetDeviceSetting` read `ProfileInfo.cfg` section/key `CurrentProfileIndex/Index`. | Confirms `Profile_v2.txt` and `ProfileInfo.cfg` profile flow. | No `Profile\*.tmp` or `loader.tmp` reference found in decompiled target classes. |
| `CControl.ResetItem` | Logs `ResetItem : <index> (<ShowName>) <StyleSelectIndex>` and proofing messages, then applies per-chipset reset logic. | Static source for runtime log lines such as `ResetItem : 1 (JARGB_V2_1) 10`. | The actual `ShowName` list comes from parsed profile/support data, not a hard-coded MS-7E75 table in this method. |
| `CControl.StartWatcher` | Creates a WMI `RegistryValueChangeEvent` watcher for `...\LED` value `MB_JARGB_V2`. | Confirms registry event flow for JARGB V2 setting changes. | Registry watching was not run. |
| `Class_MB_800.GetCycleNumber` | Maps device IDs containing `JARGB_V2_1`, `_2`, `_3`, or `EZ Conn` to ports 1..4 and falls back to `JRAINBOW*_CycleNumber` registry keys when Gen2 data is unavailable. | Static zone-name-to-port evidence. | MB800 path is not proven for MS-7E75. |
| `Class_MB_800.UpdateJARGB_V2_Basic` | Uses `App.listFixIDJARGBGen2[port]`, `App.listLEDNumJARGBGen2[port]`, registry `MB_JARGB_V2_Info{port+1}`, and `MSI_800sLed.Gen2_SetStrip` / `Gen2_ApplyPort`. | Static evidence for Gen2 strip/port construction. | No MS-7E75 tie. |
| `Class_MB_800.SetStyle` | For `JARGB_V2_1/2/3`, writes registry value `MB_JARGB_V2` to `SetJARGB_V2_1/2/3`, reads `MB_JARGB_V2_Info1/2/3`, and calls `UpdateJARGB_V2_Basic`. | Confirms the `JARGB_V2_1/2/3` profile strings drive JARGB V2 port handling in this path. | Applies to MB800/common-device path only unless tied to MS-7E75 later. |

## P/Invoke Table For `MSI_LED.MB`

All entries import from `Lib\MBAPI_x86.dll`.

| Group | Methods |
| --- | --- |
| Board/platform | `CheckMBVersion(string _csMB, string _csMBVer, string _csMBMarket, string _csMBSIOInit)`, `GetDRAMInfo3(...)`, `InitialDDRTIMING(bool bFirstRun = true)`, `Check_IsDDR5()`, `GetCPUTemp()`, `GetCPU_GameBoostSec(ref int sec)`, `GetCPU_MaxRatio(ref int ratio)`, `GetSIO_DefaultWhite(ref bool defWhite)` |
| Basic motherboard LED | `SupportLED()`, `LEDControl(int ledmode)`, `LEDBOTControl(int ledmode)`, `LEDMysticControl(int ledmode)`, `LEDAudioControl(int ledmode)`, `ResetLED()`, `CloseLEDControl(bool bBackToDefault)`, `SetLEDModelName(int model)`, `SetExtendSequence(int mode)` |
| Mystic/basic effects | `SetBreathingMode()`, `SetMysticBreathingMode()`, `SetAudioBreathingMode()`, `SetFlashingMode()`, `SetMysticFlashingMode()`, `SetAudioFlashingMode()`, `SetDualBlinkingMode()`, `SetRainbowMode()`, `SetRainbowBreathingMode()`, `SetRainbowFlashingMode()`, `SetMysticDualBlinkingMode()`, `SetAudioDualBlinkingMode()`, `SetMysticMarqueeMode()`, `SetMysticRainbowMode()`, `SetMysticMeteorMode()`, `SetMysticLightningMode()`, `SetMysticSequenceMode(int mode)`, `SetColorMode(int R, int G, int B)`, `SetColorMode3(int R, int G, int B)`, `SetMysticLEDColor(int R, int G, int B)`, `SetMusicLED(bool mystic, bool on, int mode)`, `SetMusicVolumeV2(int left, int right)` |
| Renesas-style helpers | `RenesasLEDControlV3(int index70, int index71, int index80, int index81, int index82, int index83, int cmd, int data, int r, int g, int b, int e0, int e1, int e2, int e3, int e5)`, `KeepRenesasLED()` |
| LAN/fan/BT LED | `CheckLANLED()`, `ControlLANLED(int value)`, `ControlFANLED(int value)`, `ControlBTLED(int value)`, `CheckBTLED()` |
| DRAM LED vendors | `ControlKingStonDRAMLED(int r, int g, int b, int speed, int style)`, `ControlKingStonDRAMLED_X299(int offset, int data)`, `ControlCorsairDRAMLED(...)`, `SetCorsairDRAMLED(int mode)`, `ControlCorsairProDRAMLED(...)`, `CorsairProDRAMSync()`, `ControlGALAXDRAMLED(int style, int r, int g, int b)`, `ControlGALAXDRAMLED_Byte(int data0, int data1, int data2, int data3)`, `ControlMICRONDRAMLED(int style, int r, int g, int b)` |
| DRAM helper APIs | `GSKDDR_CheckMAVERIK()`, `GSKDDR_Initial()`, `Micron_Initial(out byte Out_Micron_DDR_Type)`, `KINGSTON_Initial()`, `GSKDDR_RainbowStop()`, `GSKDDR_MeteorStop()`, `GSKDDR_MarqueeStop()`, `GSKDDR_LoopStop()`, `ITEDDR_LoopStop()`, `GSKDDR_ONOFF(int ledmode)`, `GSKDDR_Change(...)`, `GSKDDR_Change_PerLed(...)`, `GSKDDR_MSI_Style(...)`, `GetAURAInfo(...)`, `ITEDDR_Change(...)`, `GetITEInfo(...)`, `KingstonDDR5_Change(...)`, `Micron_DDR4_Change(...)`, `IT8295QFN_OP(int mode, int r, int g, int b, int addr)` |
| EC/SIO/SMBus | `SMBusControl(int addr, int offset, int data)`, `SetSIOGPIO(int Grop, int Sel, int Data)`, `SetECSpace(int Page, int Index, int Data)`, `GetECSpace(int Page, int Index, out int data)`, `SetECRAM_Mode(int mode)`, `SetECRAM_Color(byte r, byte g, byte b)`, `CheckECRAM(out bool data)`, `SetSIO5567SLEDColor(int R, int G, int B)`, `SMBus_Initial()`, `ReleaseDll()` |
| Storage/other LED | `RealtekSSD_Initial()`, `RealtekSSD_Release()`, `RealtekSSD_SetStyle(...)`, `RealtekSSD_AllSync(...)`, `SonixDDR_Initial()`, `SonixDDR_SetColor(...)`, `SonixDDR_SetVolume(...)`, `SonixDDR_SetLed1Led2Color(...)`, `SonixDDR_GetModuleInfo(...)`, `SonixDDR_LoopStop()` |

This table is generic MBAPI surface evidence. It does not prove MS-7E75 uses any specific imported method.

## Profile And Config Loading Evidence

- `Class_ParseCfg.ParseCfgFile` reads MSI Center `Component\SDK\WorkDir` from registry, then selects the newer or available copy of `Data\Mystic Light Online Data.dat` and `Mystic Light\Mystic Light Online Data.dat`.
- The online data parser strips the first seven characters, matching the known `!!MSI!!` header length, then calls `C_Encrypt.DecryptBase64` with key expression `232345599.ToString("X")`.
- `ParseCfgFile` extracts `[Motherboard]`, `[Graphics]`, `[GraphicsNumber]`, and `[SyncData]` sections. `Class_Fun_MB.Compare_Support_MB` later consumes `List_SyncData`.
- `ProfileFunction` stores profile JSON in `MSI\MSI Center\Mystic Light\Profile\Profile_v2.txt` under the Windows drive root and reads current profile index from `ProfileInfo.cfg` key `CurrentProfileIndex/Index`.
- No decompiled target class referenced `Profile\*.tmp` or `loader.tmp` by literal name.

## Zone Construction Evidence

- `CControl.ResetItem` logs the `ShowName` of `CLEDParser.List_PartItem[In_ItemIndex]`. This explains log lines with `JRGB1` and `JARGB_V2_1/2/3` once those show names are present in parsed device/profile data.
- `Class_MB_800.GetCycleNumber` maps `JARGB_V2_1`, `_2`, `_3`, and `EZ Conn` to logical ports 1..4.
- `App` reads registry keys `MB_JARGB_V2_Support1` through `MB_JARGB_V2_Support4` to set `bSupportJARGBGen2Port1..4`, `currentJARGB_Gen2`, fixed IDs, and LED counts.
- `Class_MB_800.UpdateJARGB_V2_Basic` reads `MB_JARGB_V2_Info{n}` entries as comma-separated 21-field strip records, builds `MSI_800sLed.Struct_Gen2StripSetting`, calls `Gen2_SetStrip`, then calls `Gen2_ApplyPort`.
- `Class_MB_800.SetStyle` writes `MB_JARGB_V2 = SetJARGB_V2_1/2/3/4` for advanced port changes and reads the matching `MB_JARGB_V2_Info{n}` data.
- `RGBControlClass.Init_MB_Adv_v1` detects JARGB V2 via `Device_7B10.IsSupportJARGB_V2()` or `Device_7D26.IsSupportJARGB_V2()` and runs per-port `JARGB_V2_OnlyDetect` / `JARGB_SwitchToGen1`.

## `7E75` / `MS-7E75_1` Findings

Confirmed:

- `LEDKeeper2.exe` still has no cleartext `7E75`, `MS-7E75`, `MS-7E75_1`, or `B850` after decompilation and recursive text search of decompiled output.
- `Class_Fun_MB.GetWMI` can derive `MB_Info.Product = MS-7E75` from a runtime system product string containing `(MS-7E75)`.
- `App` can derive `App.mbID = 0x7E75` from `MB_Info.Product` using substring extraction and hex conversion.
- `App` can construct `MS-7E75_1` generically as `MB_Info.Product + "_" + MB_Info.Version.Substring(0, 1)`.
- `RGBControlClass.updateSupportedDevice` can parse and log `[RGBControlClass] mbID 7E75` from incoming supported-device text containing `MS-7E75`.

Not confirmed:

- No static LEDKeeper support enum contains `MS_7E75`.
- No static LEDKeeper switch/case specifically names `7E75`.
- No decompiled LEDKeeper method maps MS-7E75 to SMBus, Renesas, EC, SIO, HID, USB, or MB800.
- No MS-7E75 SMBus address, EC offset, SIO register, HID report, IOCTL sequence, command payload, or register map was found.

## Confirmed Vs Unknown

Confirmed:

- LEDKeeper has a decompiled MBAPI P/Invoke boundary through `MSI_LED.MB`.
- LEDKeeper can generate `MS-7E75_1` and `7E75` indirectly from runtime board identity strings.
- LEDKeeper consumes encrypted/base64-like Mystic Light online data through `Class_ParseCfg`.
- LEDKeeper stores/loads user profile data through `Profile_v2.txt` and `ProfileInfo.cfg`.
- LEDKeeper contains generic JRGB/JRAINBOW/JARGB V2 zone handling and JARGB V2 registry/port logic.
- LEDKeeper contains static modern-board dispatch lists near `MS_7E75`, including `MS_7E74`, but not `MS_7E75`.

Unknown:

- Whether the installed `Mystic Light Online Data.dat` contains an MS-7E75 `[SyncData]` or profile record.
- Whether MBAPI's static `7E75` board-list hit is the source that approves MS-7E75 support.
- Which module creates the final `CLEDParser.List_PartItem` records for `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, and `JARGB_V2_3` on MS-7E75.
- Whether MS-7E75 lighting uses SMBus/Renesas, EC, SIO, USB/HID/common-device, ACPI/WMI, or another transport.

## Next Static-Only Targets

- Use the recovered `Class_ParseCfg` decode path to statically decode both installed `Mystic Light Online Data.dat` files and search decrypted `[SyncData]` / `[Motherboard]` records for MS-7E75.
- Decompile `CLEDParser` support parsing, especially `CheckSupportDevice`, `VerifySupportDevice`, `List_PartItem` construction, `ShowName`, `MainDevice`, `DeviceName`, and `Chipest` assignment.
- Decompile `C_Encrypt.DecryptBase64` to document the exact `!!MSI!!` data transform.
- Continue static MBAPI work around the confirmed `7E75` board-ID list and `CheckMBVersion` / `SupportLED` consumers.
- Inspect `MLModule.dll`, `MysticLight_AllDevice.dll`, and `SyncData.dll` statically for MS-7E75 records and `CLEDParser` data types.
- Inspect registry/profile schema references for `MB_JARGB_V2_Support*`, `MB_JARGB_V2_Info*`, and generated zone show names without reading live hardware.

## Explicit Hardware-Access Note

No MSI binaries were executed during this pass. MSI Center, Mystic Light, and `LEDKeeper2.exe` were not launched. No hardware access was enabled or run: no doctor command, no chip detection, no register read, no write/apply command, no `/dev/port`, no raw SMBus, no raw Super I/O, and no MS-7E75 hardware support code changes.
