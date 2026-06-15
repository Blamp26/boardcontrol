# MSI MS-7E75 LEDKeeper2 Static Reverse Engineering Notes

Status: static analysis only, hardware access disabled.

## Scope

This document records a static-only pass over:

```text
C:\Program Files (x86)\MSI\MSI Center\Mystic Light\LEDKeeper2.exe
```

The goal was to look for MS-7E75 / B850 GAMING PLUS WIFI PZ board/profile/zone dispatch evidence, including `7E75`, `MS-7E75`, `MS-7E75_1`, JRGB/JRAINBOW/JARGB/JARGB_V2 zone strings, `RGBControlClass`, profile/config filenames, online data references, and MBAPI/SMBus/Driver/EC/SIO/Renesas route clues.

Evidence came from PE metadata, .NET reflection-only metadata loading, manifest/resource names, P/Invoke metadata, and raw ASCII/UTF-16 string searches. This pass did not execute MSI software.

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

## Evidence Table

| Source | Evidence | MS-7E75 relevance | Limitations |
| --- | --- | --- | --- |
| PE/.NET metadata | `LEDKeeper2.exe` is an x86 .NET Framework 4.8 assembly with only `_CorExeMain` in the native import table. | Analysis should treat most logic as managed metadata/IL rather than native imports. | This pass did not recover full IL control flow. |
| Assembly references | References include `MLModule`, `MysticLight_AllDevice`, `SyncData`, `System.Management`, `websocket-sharp`, device APIs, and several Mystic Light companion assemblies. | Confirms LEDKeeper is a high-level Mystic Light orchestration executable. | References do not identify an MS-7E75 hardware route by themselves. |
| Manifest resources | Embedded resources are `LEDKeeper2.g.resources` and `MSI_LED.Properties.Resources.resources`. | Confirms WPF/application resource presence and localized Mystic Light strings. | No decoded MS-7E75 profile table was found in resource names or simple strings. |
| `MSI_LED.MB` metadata | Managed P/Invoke wrapper points to `Lib\MBAPI_x86.dll` for `SupportLED`, `LEDControl`, `LEDMysticControl`, many `SetMystic*` calls, `RenesasLEDControlV3`, `KeepRenesasLED`, `SetSIOGPIO`, `SetECSpace`, `SetECRAM_*`, `SMBus_Initial`, and other helpers. | Confirms LEDKeeper delegates motherboard LED/control work through MBAPI and exposes several possible route families. | P/Invoke presence is generic. It does not tie MS-7E75 to SMBus, Renesas, EC, SIO, or any register map. |
| `MSI_LED.Class_Fun_MB` metadata | Wrapper methods mirror MBAPI motherboard operations: `Compare_Support_MB`, `Init_MB`, `LEDMysticControl*`, `SetMystic*V2`, `RenesasLEDControlV3`, `ControlFANLED`, and related calls. | Candidate high-level motherboard dispatch wrapper. | No `7E75`-specific branch or map was recovered from this metadata pass. |
| `MSI_LED.RGBControlClass` metadata | Methods include `updateSupportedDevice`, `Init_MB_Adv_v1`, `Init_MB_Adv_v2`, private `MB_SetRGB`, device-specific `*_SetRGB`, `Init`, `RGBControl`, and server methods. Fields include `bIsMSI800sLED`, `bIsMSI7B10LED`, `bIsMSI7D26LED`, and support booleans. | Strong candidate for support/profile/device dispatch; matches log prefix `[RGBControlClass]`. | Static strings include `[RGBControlClass] mbID ` but no decoded `7E75` value. |
| `MSI_LED.MSI_7B10Led` metadata | Despite the class name, support enums include many board IDs through `MS_7E74`; methods include `CheckSupportMethod`, `IsSupportMixEffect`, `IsSupportJARGB_V2`, `Set_AllBoard`, `Get_AllBoard`, `JARGB_V2_Detect`, `JARGB_Apply`, and Gen1/per-LED helpers. | Candidate board/profile/zone dispatch class for many modern MSI boards and JARGB V2 support. | Its visible support lists do not include `MS_7E75`. It cannot be claimed as the MS-7E75 path from this evidence. |
| `MSI_LED.MSI_7B10Led+SupportList` | Contains IDs including `MS_7E01`, `MS_7E03`, `MS_7E06`, `MS_7E07`, `MS_7E09`, `MS_7E10`, `MS_7E12`, and older `7Dxx` boards. | Shows LEDKeeper embeds board-ID dispatch lists adjacent to LED support logic. | `MS_7E75` is absent. |
| `MSI_LED.MSI_7B10Led+SupportList_CommonID` | Contains `MS_7E00`, `MS_7E11`, `MS_7E12`, `MS_7E13`, `MS_7E14`, `MS_7E16`, `MS_7E18`, `MS_7E23`, `MS_7E24`, `MS_7E25`, `MS_7E26`, `MS_7E27`, `MS_7E28`, `MS_7E29`, `MS_7E30`, `MS_7E31`, `MS_7E36`, `MS_7E37`, `MS_7E46`, and `MS_7E74`. | Shows a separate common-ID dispatch table for modern boards. | `MS_7E75` is absent; `MS_7E74` proximity is not proof for `MS_7E75`. |
| `MSI_LED.MSI_7B10Led+SupportList_JARGB_V2` | Contains many `7Dxx` and `7Exx` entries through `MS_7E74`; paired metadata exposes `IsSupportJARGB_V2`, `JARGB_V2_Detect`, and `JARGB_Apply`. | Strong static evidence that LEDKeeper has JARGB V2 support gating for some motherboard IDs. | `MS_7E75` is absent. No MS-7E75 JARGB V2 dispatch is proven. |
| `MSI_LED.MSI_7D26Led` metadata | Support list includes `MS_7D26`, `MS_7D68`, `MS_7D85`, plus JARGB V2 and board-setting methods similar to `MSI_7B10Led`. | Another board-family dispatch implementation pattern. | No MS-7E75 evidence. |
| `MSI_LED.MSI_800sLed` / `MSI_LED.MSI_B921Led` metadata | Methods expose Gen1/Gen2 board and strip operations such as `Gen1_ApplyBoard`, `Gen2_Detect`, `Gen2_ApplyPort`, and target-port enums with `JARGB1`, `JARGB2`, `JARGB3`. | Shows LEDKeeper contains USB/common-device style Gen1/Gen2 ARGB logic. | No board-specific tie to MS-7E75. |
| Raw strings | `JARGB_V2_1`, `JARGB_V2_2`, and `JARGB_V2_3` appear as UTF-16 strings at offsets `0x2F0FCF`, `0x2F0FE5`, and `0x2F0FFB`. Related strings include `SetJARGB_V2_1`, `SetJARGB_V2_2`, `SetJARGB_V2_3`, and `MB_JARGB_V2_Info1/2/3`. | Static source for the same style of zone names seen in existing logs, except logs pair them with MS-7E75. | These strings are generic MB/JARGB V2 names and do not prove MS-7E75 routing. |
| Raw strings | `JRGB1`, `JRGB2`, `JRAINBOW1`, `JRAINBOW2`, and cycle-number strings are present. | Static source for generic motherboard header/zone vocabulary. | The log-specific `JRGB1` hit is consistent, but no MS-7E75 pairing was found. |
| Raw strings | `[RGBControlClass] Constructor`, `[RGBControlClass] mbID `, and many `[RGBControlClass] ...` log strings are embedded. | Confirms that existing log prefix `[RGBControlClass] mbID 7E75` can plausibly originate from LEDKeeper code. | The literal `7E75` value is not embedded next to this string in LEDKeeper. |
| Raw strings | `Support list : ` appears as UTF-16 at offsets `0x2FED29` and `0x2FF51A`. `ResetItem : ` appears at `0x306733`; `Finish ResetItem : ` appears at `0x306863`. | Confirms LEDKeeper embeds the log message templates seen in existing Mystic Light logs. | The actual runtime list value `MS-7E75_1` is not embedded in this executable. |
| Raw strings | `Data\Mystic Light Online Data.dat`, `Mystic Light\Mystic Light Online Data.dat`, `using main online data`, `using backup online data`, and `not found online data` are present. | Confirms LEDKeeper directly knows the online-data blob filenames and selection fallback messages. | The blob format remains undecoded; no MS-7E75 record was extracted here. |
| Raw strings | `\Profile\`, `MSI\MSI Center\Mystic Light\Profile\`, `ProfileInfo.cfg`, `Profile_v2.txt`, and `CurrentProfileIndex` are present. | Confirms LEDKeeper uses profile/config files separate from the executable. | No `Profile\*.tmp` or `loader.tmp` literal was found in `LEDKeeper2.exe`. |
| Raw strings | `Global\Access_SMBUS.HTP.Method` appears; method names include `SMBus_Initial` and `SetSMBusControl`. | Confirms an SMBus-related synchronization/name string and MBAPI-style SMBus calls are visible in LEDKeeper. | No `SMBus_Engine.dll` literal was found in LEDKeeper and no MS-7E75 SMBus route is proven. |
| Raw strings | `RenesasLEDControlV3`, `KeepRenesasLED`, `Device_Renesas_Fan`, and `RenesasMusicValumnTrans` are present. | Confirms Renesas-related call names are visible in the managed layer. | Generic; no MS-7E75 Renesas controller or address proof. |
| Raw strings | `SetSIOGPIO`, `SetSIO5567SLEDColor`, `CheckECRAM`, `SetECRAM_Mode`, and `SetECRAM_Color` are present. | Confirms EC/SIO paths are also exposed to LEDKeeper. | Generic; no MS-7E75 EC/SIO register proof. |
| Negative string search | No literal `7E75`, `MS-7E75`, `MS-7E75_1`, or `B850` was found in `LEDKeeper2.exe`. | Important boundary for claims: LEDKeeper has generic dispatch machinery and log templates, but not the known board ID in cleartext. | The value could be computed, loaded from MBAPI, online data, profile blobs, WMI/DMI, registry, or another module. |

## Board/Profile/Zone Hits

Confirmed in `LEDKeeper2.exe`:

- Generic motherboard wrapper classes: `MSI_LED.MB`, `MSI_LED.Class_Fun_MB`, `MSI_LED.RGBControlClass`.
- MBAPI P/Invoke target: `Lib\MBAPI_x86.dll`.
- Generic board support enums, especially `MSI_LED.MSI_7B10Led+SupportList`, `SupportList_CommonID`, `SupportList_MixEffect`, and `SupportList_JARGB_V2`.
- Visible board IDs up to nearby `MS_7E74` in `MSI_LED.MSI_7B10Led` support lists.
- Header/zone strings: `JRGB1`, `JRGB2`, `JRAINBOW1`, `JRAINBOW2`.
- JARGB V2 strings: `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, `SetJARGB_V2_1`, `SetJARGB_V2_2`, `SetJARGB_V2_3`, `MB_JARGB_V2_Info1`, `MB_JARGB_V2_Info2`, `MB_JARGB_V2_Info3`.
- Log templates: `Support list : `, `ResetItem : `, `Finish ResetItem : `, `[RGBControlClass] mbID `.
- Online/config filenames: `Data\Mystic Light Online Data.dat`, `Mystic Light\Mystic Light Online Data.dat`, `ProfileInfo.cfg`, `Profile_v2.txt`, and `MSI\MSI Center\Mystic Light\Profile\`.

Not found in `LEDKeeper2.exe` cleartext:

- `7E75`
- `MS-7E75`
- `MS-7E75_1`
- `B850`
- `GAMING PLUS` for the target board. `MPG B550 GAMING PLUS` and `MPG Z490 GAMING PLUS` are present, but they are unrelated older-board strings.
- `SMBus_Engine.dll`
- `Driver_Engine.dll`
- `NTIOLib`
- `Profile\*.tmp`
- `loader.tmp`

## Candidate Dispatch Functions/Tables

| Candidate | Why it matters | Current status |
| --- | --- | --- |
| `MSI_LED.RGBControlClass.updateSupportedDevice` | Likely builds supported-device state; class contains support booleans and log templates matching existing runtime logs. | Candidate only; no MS-7E75 branch recovered. |
| `MSI_LED.RGBControlClass.Init_MB_Adv_v1` / `Init_MB_Adv_v2` | Likely selects older vs advanced motherboard LED behavior. | Candidate only. |
| `MSI_LED.RGBControlClass.MB_SetRGB` | Private motherboard SignalRGB/set path, with nested compiler-generated methods. | Candidate only. |
| `MSI_LED.Class_Fun_MB.Compare_Support_MB` / `Init_MB` | Managed wrapper around motherboard support and MBAPI setup. | Candidate only. |
| `MSI_LED.MB.SupportLED` and related P/Invoke methods | Direct MBAPI boundary for support detection and Mystic Light operations. | Confirmed generic MBAPI boundary; not board-specific. |
| `MSI_LED.MSI_7B10Led.CheckSupportMethod` | Static support-list selector for many modern board IDs, including nearby `7E**` IDs. | Strong candidate table consumer; `MS_7E75` absent. |
| `MSI_LED.MSI_7B10Led.IsSupportJARGB_V2` | Likely consumes `SupportList_JARGB_V2`. | Strong candidate for JARGB V2 feature gating; `MS_7E75` absent. |
| `MSI_LED.MSI_7B10Led.Set_AllBoard` / `Get_AllBoard` | Candidate board state serialization methods for LED settings. | Candidate only. |
| `MSI_LED.MSI_7B10Led.JARGB_V2_Detect` / `JARGB_Apply` / `JARGB_SwitchToGen1` | Candidate JARGB V2 zone construction and apply path. | Candidate only. |
| `MSI_LED.Class_MB_800.SetStyle` / `UpdateJARGB_V2_Basic` | Higher-level MB 800/JARGB V2 UI-style application path. | Candidate only. |
| `MSI_LED.CControl.ResetItem` compiler-generated methods | Log string call site for `ResetItem`. | Outer type was not fully loaded by this reflection-only pass; raw metadata/string evidence confirms method-template presence. |
| `MSI_LED.Class_ParseCfg.ParseCfgFile` | Candidate parser for online/support/config data. | Candidate only. |

## Confirmed Vs Unknown

Confirmed:

- `LEDKeeper2.exe` is a managed x86 .NET Framework 4.8 Mystic Light orchestration executable.
- It statically references `Lib\MBAPI_x86.dll` through P/Invoke for motherboard LED/support, Renesas, EC, SIO, SMBus initialization, and other hardware-adjacent helpers.
- It contains `RGBControlClass` log templates matching existing log prefix style.
- It contains `Support list : ` and `ResetItem : ` log templates matching existing Mystic Light logs.
- It contains generic JRGB/JRAINBOW/JARGB V2 zone strings, including `JARGB_V2_1`, `JARGB_V2_2`, and `JARGB_V2_3`.
- It contains profile and online-data filenames, including `Mystic Light Online Data.dat`, `ProfileInfo.cfg`, and `Profile_v2.txt`.
- It contains embedded board support enums for many older and modern board IDs, including nearby `MS_7E74`.

Unknown:

- Which static source creates `MS-7E75_1`.
- Which static source pairs MS-7E75 with `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, and `JARGB_V2_3`.
- Whether MS-7E75 is intentionally absent from LEDKeeper support enums because it is supplied by online data, MBAPI, `MLModule.dll`, `MysticLight_AllDevice.dll`, encoded profile blobs, registry, or another runtime source.
- Which function consumes the real MBAPI `7E75` board-ID list hit.
- Whether MS-7E75 motherboard headers use SMBus/Renesas, EC, SIO, USB/HID/common-device, ACPI/WMI, or another transport.
- Any MS-7E75 SMBus address, EC offset, SIO register, USB report, IOCTL sequence, command payload, or register map.

## Next Static-Only Targets

- Decompile `LEDKeeper2.exe` IL around `RGBControlClass.updateSupportedDevice`, `RGBControlClass.Init_MB_Adv_v1`, `RGBControlClass.Init_MB_Adv_v2`, `RGBControlClass.MB_SetRGB`, `Class_Fun_MB.Compare_Support_MB`, and `Class_Fun_MB.Init_MB`.
- Decompile `MSI_LED.MSI_7B10Led.CheckSupportMethod`, `IsSupportJARGB_V2`, `Set_AllBoard`, `Get_AllBoard`, `JARGB_V2_Detect`, `ParserJargbV2InfoToLed_Settings`, and `JARGB_Apply`.
- Cross-reference the `Support list : `, `ResetItem : `, and `[RGBControlClass] mbID ` log templates back to IL call sites and data sources.
- Decompile `Class_ParseCfg.ParseCfgFile` and online-data selection logic around `Mystic Light Online Data.dat`.
- Reverse the `!!MSI!!` encoded `Mystic Light Online Data.dat` format and determine whether it contains `MS-7E75_1`.
- Reverse or identify the `Mystic Light\Profile\*.tmp` and `loader.tmp` profile blob format.
- Continue static work on `MBAPI_x86.dll` around the confirmed `7E75` board-ID list and map its table consumer.
- Inspect `MLModule.dll`, `MysticLight_AllDevice.dll`, and `SyncData.dll` statically for `MS-7E75_1`, JARGB V2 support records, and LEDKeeper call targets.

## Explicit Hardware-Access Note

No MSI binaries were executed during this pass. MSI Center, Mystic Light, and `LEDKeeper2.exe` were not launched. No hardware access was enabled or run: no doctor command, no chip detection, no register read, no write/apply command, no `/dev/port`, no raw SMBus, no raw Super I/O, and no MS-7E75 hardware support code changes.
