# MSI MS-7E75 Profile Selection Static Notes

Status: static search only, hardware access disabled.

## Scope

This document records a static-only search for MSI Center / Mystic Light board-profile and route-selection evidence related to MSI MS-7E75 / B850 GAMING PLUS WIFI PZ.

The search focused on installed MSI Center and Mystic Light files, looking for:

- Board and platform terms: `MS-7E75`, `7E75`, `B850`, `GAMING PLUS`
- Lighting zone terms: `JRGB`, `JRAINBOW`, `JARGB`, `ARGB`
- Candidate profile/config formats: JSON, XML, INI, CFG, DAT, TMP, resource strings, logs
- Transport/module references: `MBAPI`, `SMBus_Engine`, `Driver_Engine`, `NTIOLib`
- Renesas, SMBus address, LED zone, board-ID, and backend-dispatch evidence

This pass does not replace the direct static module notes. It tries to identify where MSI Center stores or computes board selection, backend selection, LED zones, SMBus addresses, JRGB/JRAINBOW mapping, and MS-7E75-specific behavior.

## Safety Constraints

- Documentation only.
- Static file inspection only.
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
- No path is claimed as the MS-7E75 hardware path unless evidence is specific.

## Searched Locations And Files

Primary installed locations:

- `C:\Program Files (x86)\MSI\MSI Center\Mystic Light`
- `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib`
- `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Profile`
- `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Log`
- `C:\Program Files (x86)\MSI\MSI Center\Data`
- `C:\Program Files (x86)\MSI\MSI Center\Log`

Notable files inspected:

- `Mystic Light\Lib\MBAPI_x86.dll`
- `Mystic Light\LEDKeeper2.exe`
- `Mystic Light\API_Mystic Light.dll`
- `Mystic Light\Support.cfg`
- `Mystic Light\Mystic Light Online Data.dat`
- `Data\Mystic Light Online Data.dat`
- `Mystic Light\Lib\CCD_MB.xml`
- `Mystic Light\Profile\*.tmp`
- Existing log artifacts under `Mystic Light\Log`, `MSI Center\Log\CC_Engine`, and `MSI Center\Log\MysticLight_Test`

The log files are treated separately from static config/code evidence. They are useful installed artifacts that show what MSI software previously recorded on this machine, but they are not the static source of the board/profile mapping.

## Search Terms

The main terms were:

```text
MS-7E75
7E75
B850
GAMING PLUS
JRGB
JRAINBOW
JARGB
ARGB
MS-7E75_1
JRGB1
JARGB_V2_1
JARGB_V2_2
JARGB_V2_3
Renesas
SMBus
0x52
MBAPI
SMBus_Engine
Driver_Engine
NTIOLib
RGBControlClass
Support list
```

## Evidence Table

| Source | Evidence | MS-7E75 relevance | Limitations |
| --- | --- | --- | --- |
| `Mystic Light\Lib\MBAPI_x86.dll` | Contains a broad ASCII board-ID list with `7E75` between nearby `7E7x` IDs. The hit was at file offset `0x21D434` in this installed binary. | Confirms the MBAPI-like layer knows the `7E75` board ID statically. | The surrounding bytes look like a broad board/support list, not a decoded LED-zone map. It does not identify backend, SMBus address, register map, or JRGB/JRAINBOW mapping. |
| `Mystic Light\Lib\MBAPI_x86.dll` | Contains module/transport strings including `\SMBus_Engine.dll`, `\Driver_Engine.dll`, `NTIOLib_MysticLight`, `Global\Access_SMBUS.Renesas.HTP.Method`, `Global\Access_SMBUS.HTP.Method`, `Global\Access_ISABUS.HTP.Method`, and `Global\Access_EC`. | Supports prior generic-path evidence and shows possible route families available to MBAPI. | The `7E75` board-list hit was not tied to these route strings by this pass. |
| `Mystic Light\LEDKeeper2.exe` | Contains many Mystic Light orchestration, device, profile, all-sync, JARGB, MB, and route-like strings, plus board/profile strings for older boards such as B460/B550/Z490/Z590/Z690 families. | Strong candidate for profile and UI/device dispatch logic. It contains generic JRGB/JRAINBOW zone strings and motherboard profile code paths. | No confirmed `MS-7E75`, `7E75`, `B850`, `MS-7E75_1`, `JRGB1`, or `JARGB_V2_1` static hit was isolated in this pass. |
| `Mystic Light\LEDKeeper2.exe` direct metadata pass | SHA-256 `990C65F31038AA6DCA39ABBE33735E42424B37696FB56D5B58D6EEA05FBB8159`. It is a managed x86 .NET Framework 4.8 assembly. `MSI_LED.MB` P/Invokes `Lib\MBAPI_x86.dll`; `RGBControlClass`, `Class_Fun_MB`, and `MSI_7B10Led` are candidate dispatch classes. Static strings include `Support list : `, `ResetItem : `, `[RGBControlClass] mbID `, `JRGB1`, `JRGB2`, `JRAINBOW1`, `JRAINBOW2`, `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, `Mystic Light Online Data.dat`, `ProfileInfo.cfg`, and `Profile_v2.txt`. | Confirms LEDKeeper is a strong static source for generic profile/zone/log-template and MBAPI dispatch evidence, and likely produced several log template strings seen in installed logs. | Cleartext `7E75`, `MS-7E75`, `MS-7E75_1`, and `B850` were not found in this executable. Visible `MSI_7B10Led` support lists include nearby `MS_7E74` but not `MS_7E75`, so no MS-7E75 backend/register map is proven. See [MSI_7E75_LEDKEEPER_STATIC_RE.md](MSI_7E75_LEDKEEPER_STATIC_RE.md). |
| `Mystic Light\API_Mystic Light.dll` | Contains generic UI/resource strings such as `StringStyleADDRESSABLE`, `StringStyleJRAINBOW_Marquee`, `StringStyleJRAINBOW_Meteor`, `StringStyleJRAINBOW_Stack`, `StringStyleJRAINBOW_Rainbow`, and `StringLabelLEDAREA`. | Confirms generic addressable/JRAINBOW UI vocabulary. | Generic UI text only; no MS-7E75 board dispatch, zone map, transport, address, or register proof. |
| `Mystic Light\Support.cfg` | Plain text config. It starts with `[Motherboard]`, but that section is empty; `[Graphics]` and `[GraphicsNumber]` contain GPU support records. | Candidate static config file, but it does not provide MS-7E75 motherboard lighting data. | The only `GAMING Plus` hit is a GPU record: `RX 580 GAMING Plus 8G`; it is not B850/MS-7E75 motherboard evidence. |
| `Mystic Light\Lib\CCD_MB.xml` | Plain XML table of motherboard IDs for CPU/clock/game-boost-like feature groups. Includes older IDs such as `7A45`. | Shows MSI uses XML MBID tables in installed components. | No `7E75` hit and no Mystic Light LED/profile mapping identified. |
| `Data\Mystic Light Online Data.dat` | Candidate online Mystic Light data file. SHA-256 `256AD24C5CB2733F23154CE766455249F62A604725C0E2CA5920CEB4FC59C4D6`, length 450143 bytes, header `!!MSI!!`. Base64 decoding yields 337600 bytes beginning `2C 38 38 32 A1 E2 30 E1 ...`. | Strong candidate for downloaded profile/config data. The hash matches the installed log line reporting a successful download of `Mystic Light Online Data.dat`. | No plaintext board/profile terms were found after simple base64 decode; likely encrypted, compressed, or custom encoded. No MS-7E75 mapping decoded. |
| `Mystic Light\Mystic Light Online Data.dat` | Candidate bundled or cached Mystic Light data file. SHA-256 `EA957F45C4F69AB4125195A00928842666A84C90980ED244510CC0511299BA00`, length 452807 bytes, header `!!MSI!!`. Base64 decoding yields 339600 bytes with the same decoded header prefix as the `Data` copy. | Another strong candidate profile/config blob. | No plaintext board/profile terms were found after simple base64 decode; decoding format remains unknown. |
| `Mystic Light\Profile\*.tmp` | 37 profile-like `.tmp` files plus `loader.tmp`. Most `.tmp` files share binary header `DF BD 50 36 8D 48 6C B5 16 32 16 3B 07 EB C1 EA`; `loader.tmp` begins `71 9C D1 F9 BA 29 D2 06 BF FF F1 21 DB CF 1F 75`. | Candidate packaged/encrypted profile resources. | No readable `MS-7E75`, `7E75`, `B850`, `JRGB`, `JRAINBOW`, `JARGB`, `Renesas`, or `SMBus` evidence found in simple static string searches. |
| `Mystic Light\Log\MLModule.txt` | Existing runtime log artifact repeatedly records `BaseBoard Product : B850 GAMING PLUS WIFI PZ (MS-7E75)`. | Confirms MSI software previously identified this host board in logs. | Runtime log artifact only; not the static profile source and not hardware path proof from this pass. |
| `MSI Center\Log\CC_Engine\CC_Engine_2026_05_16.txt` | Existing runtime log artifact records `Type1 ProductName : MS-7E75`, `Type2 ProductName : B850 GAMING PLUS WIFI PZ (MS-7E75)`, `MB Procuct Name : 7E75`, `NBChip : B850`, and `Processline : 8 - SMBus`. | Confirms MSI Center previously classified board ID/chipset and reached an SMBus process-line step. | Runtime log artifact only. `Processline : 8 - SMBus` is not proof that MS-7E75 Mystic Light LED headers use SMBus. |
| `MSI Center\Log\MysticLight_Test\MysticLight.txt` | Existing runtime log artifact records `Support list : 1,MS-7E75_1,...`, `ResetItem : 0 (JRGB1) 10`, `ResetItem : 1 (JARGB_V2_1) 10`, `ResetItem : 2 (JARGB_V2_2) 10`, `ResetItem : 3 (JARGB_V2_3) 10`, and `[RGBControlClass] mbID 7E75`. | Strongest current evidence for the board/profile name and LED zone labels used by Mystic Light on this host. | Runtime log artifact only. It does not identify the static source of the profile, transport backend, SMBus address, command bytes, or register map. |

## MS-7E75 / B850 / JRGB / JRAINBOW Hits

Confirmed static code/config hits:

- `MBAPI_x86.dll` contains `7E75` in a broad board-ID list.
- `MBAPI_x86.dll` contains generic SMBus, Renesas, EC, ISA, Driver Engine, SMBus Engine, and NTIOLib strings.
- `API_Mystic Light.dll` and `LEDKeeper2.exe` contain generic `JRAINBOW`/addressable lighting UI strings.
- `LEDKeeper2.exe` contains generic motherboard, JARGB, JRGB/JRAINBOW, profile, and route-like code/resource strings, but no isolated `7E75` or `B850` static string was confirmed in this pass.
- A later direct metadata pass on `LEDKeeper2.exe` confirmed generic `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, and `JARGB_V2_3` strings, `Support list : ` / `ResetItem : ` / `[RGBControlClass] mbID ` log templates, `Lib\MBAPI_x86.dll` P/Invoke metadata, and candidate dispatch classes, but still found no cleartext `7E75`, `MS-7E75`, `MS-7E75_1`, or `B850` in that executable.

Confirmed runtime-log artifact hits:

- `B850 GAMING PLUS WIFI PZ (MS-7E75)`
- `MS-7E75`
- `7E75`
- `B850`
- `MS-7E75_1`
- `JRGB1`
- `JARGB_V2_1`
- `JARGB_V2_2`
- `JARGB_V2_3`
- `[RGBControlClass] mbID 7E75`

Not found as decoded static profile/config evidence:

- No decoded `MS-7E75_1` profile record.
- No decoded MS-7E75 backend selector.
- No decoded MS-7E75 SMBus address.
- No decoded MS-7E75 Renesas controller profile.
- No decoded MS-7E75 JRGB/JRAINBOW register or payload map.

## Candidate Profile / Config Files

`Mystic Light Online Data.dat` is the strongest candidate for externally supplied Mystic Light profile/config data. Both installed copies begin with `!!MSI!!`, and simple base64 decoding produces binary data. The current `Data` copy has SHA-256 `256AD24C5CB2733F23154CE766455249F62A604725C0E2CA5920CEB4FC59C4D6`, matching the hash recorded in an existing MSI Center log line for a successful download.

`Mystic Light\Profile\*.tmp` is also a strong candidate family. The stable binary headers and profile-like directory placement suggest packed, encrypted, or otherwise encoded data. No readable MS-7E75 or lighting-zone strings were found by this pass.

`LEDKeeper2.exe` remains a likely static code target for profile selection and board dispatch. It has extensive Mystic Light orchestration strings and embedded board/profile data for older motherboard families. It should be decompiled statically around `Check support`, `Support list`, `ResetItem`, `RGBControlClass`, `MB800`, `TimerControlMB`, `JARGB_V2`, `Get_AllBoard`, `Set_AllBoard`, `LoadProfile`, and `ParseCfgFile`.

The direct LEDKeeper2 metadata pass refines this target list to `RGBControlClass.updateSupportedDevice`, `RGBControlClass.Init_MB_Adv_v1`, `RGBControlClass.Init_MB_Adv_v2`, `RGBControlClass.MB_SetRGB`, `Class_Fun_MB.Compare_Support_MB`, `MSI_7B10Led.CheckSupportMethod`, `MSI_7B10Led.IsSupportJARGB_V2`, `MSI_7B10Led.Set_AllBoard`, `MSI_7B10Led.Get_AllBoard`, `MSI_7B10Led.JARGB_V2_Detect`, `MSI_7B10Led.JARGB_Apply`, `Class_MB_800.SetStyle`, `Class_MB_800.UpdateJARGB_V2_Basic`, and `Class_ParseCfg.ParseCfgFile`.

`MBAPI_x86.dll` remains relevant because it contains the confirmed `7E75` board-ID list and the generic transport strings. The open question is how the board-ID list participates in dispatch and whether it gates SMBus, EC, ISA/SIO, or other paths.

## Confirmed Vs Unknown

Confirmed:

- Static MBAPI evidence includes the `7E75` board ID.
- Static MBAPI evidence includes generic route families: SMBus/Renesas, EC, ISA/SIO, Driver Engine, SMBus Engine, and NTIOLib.
- Static LEDKeeper2 evidence includes the MBAPI P/Invoke boundary, generic profile/online-data filenames, generic JARGB V2 zone strings, and log templates matching `Support list`, `ResetItem`, and `[RGBControlClass] mbID`.
- Installed runtime logs show MSI software previously recognized this machine as `B850 GAMING PLUS WIFI PZ (MS-7E75)`.
- Installed Mystic Light logs show a board/profile-like token `MS-7E75_1`.
- Installed Mystic Light logs show LED zone labels `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, and `JARGB_V2_3`.
- Candidate encoded profile/config blobs exist, especially `Mystic Light Online Data.dat` and `Mystic Light\Profile\*.tmp`.

Unknown:

- Which static file or code path creates `MS-7E75_1`.
- Which static file or code path creates the `JRGB1` / `JARGB_V2_*` reset-item list.
- Whether MS-7E75 lighting uses SMBus/Renesas, EC, Super I/O GPIO, ACPI/WMI, USB/HID, RTK bridge, or another MSI route.
- Whether `Processline : 8 - SMBus` in CC Engine logs is relevant to Mystic Light LED control or only generic platform initialization.
- Whether the encoded `Mystic Light Online Data.dat` files contain the missing board/profile table.
- Whether the `Profile\*.tmp` files contain the missing zone or profile resources.
- Any MS-7E75 SMBus address, command byte, payload layout, IOCTL sequence, EC offset, Super I/O register, or register map.

## Next Static-Only Targets

- Statically decompile `LEDKeeper2.exe` around support detection and board/profile construction, especially `RGBControlClass.updateSupportedDevice`, `RGBControlClass.MB_SetRGB`, `Class_Fun_MB.Compare_Support_MB`, `MSI_7B10Led.CheckSupportMethod`, `MSI_7B10Led.IsSupportJARGB_V2`, `MSI_7B10Led.JARGB_V2_Detect`, `Class_MB_800.UpdateJARGB_V2_Basic`, and log-string call sites for `Support list`, `ResetItem`, and `[RGBControlClass] mbID`.
- Statically decompile `MBAPI_x86.dll` around the `7E75` board-ID list to identify table shape, consumers, and dispatch effects.
- Reverse the `!!MSI!!` plus base64 plus binary format used by `Mystic Light Online Data.dat`.
- Reverse or identify the binary format used by `Mystic Light\Profile\*.tmp`.
- Cross-reference runtime log strings back to static call sites in `LEDKeeper2.exe` and `MBAPI_x86.dll`.
- Continue searching installed MSI Center modules for ACPI/WMI/HID/USB board selectors without executing MSI binaries.

## Explicit Hardware-Access Note

No hardware access was enabled or run during this search. MSI Center and Mystic Light were not launched. No hardware monitor command, `/dev/port` command, raw SMBus access, raw Super I/O access, chip-detection command, register-read command, write command, apply command, or MS-7E75 hardware support code was run or added.
