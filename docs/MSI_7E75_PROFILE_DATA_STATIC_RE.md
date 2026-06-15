# MSI MS-7E75 Profile Data Static Reverse Engineering Notes

Status: static analysis only, hardware access disabled.

## Scope

This document records static-only decoding and inspection of MSI Mystic Light profile/config data related to MSI MS-7E75 / B850 GAMING PLUS WIFI PZ.

Primary targets:

- `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Mystic Light Online Data.dat`
- `C:\Program Files (x86)\MSI\MSI Center\Data\Mystic Light Online Data.dat`
- `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Profile\*.tmp`
- `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Profile\loader.tmp`
- `C:\MSI\MSI Center\Mystic Light\Profile\ProfileInfo.cfg`
- `C:\MSI\MSI Center\Mystic Light\Profile\Profile_v2.txt`
- `C:\MSI\MSI Center\Mystic Light\Profile\Profile_1.cfg`
- `C:\MSI\MSI Center\Mystic Light\Profile\Profile_2.cfg`
- `C:\MSI\MSI Center\Mystic Light\Profile\ML3.cfg`

The goal was to identify the static source for `MS-7E75_1`, zone names, and any backend/register hints without executing MSI software or touching hardware.

## Safety Constraints

- Documentation only.
- Static file inspection only.
- MSI Center was not run.
- Mystic Light was not run.
- `LEDKeeper2.exe` was not run.
- No MSI binary was executed.
- `cargo run -- doctor` was not run.
- `detect-chip`, `read-reg`, `write`, and `apply` were not run.
- `/dev/port` was not touched.
- No raw SMBus access was performed.
- No raw Super I/O access was performed.
- MS-7E75 hardware access was not enabled.
- No MS-7E75 transport or register map is claimed unless evidence is specific.

## Files Inspected And Hashes

| File | Size | SHA-256 | Notes |
| --- | ---: | --- | --- |
| `Mystic Light\Mystic Light Online Data.dat` | 452,807 | `EA957F45C4F69AB4125195A00928842666A84C90980ED244510CC0511299BA00` | `!!MSI!!` AES/base64 data. Decoded successfully. |
| `Data\Mystic Light Online Data.dat` | 450,143 | `256AD24C5CB2733F23154CE766455249F62A604725C0E2CA5920CEB4FC59C4D6` | `!!MSI!!` AES/base64 data. Decoded successfully. |
| `Profile\loader.tmp` | 1,840 | `DCB88583FD15732401B104ACC083CEB33B603A9FA4B766F5E9159657B68EDDD7` | High-entropy binary. No readable target hits. |
| `C:\MSI...\ProfileInfo.cfg` | 71 | `F1243552E48B70A1AA3088B4E6C99703D9AF9AC4169105270D3CFCF59D30078A` | Current profile index INI. |
| `C:\MSI...\Profile_v2.txt` | 27,410 | `434C24474F248B4B5973B5972278776ABB9F8966735CD7827F62D8D139053D53` | JSON profile state with MS-7E75 device names. |
| `C:\MSI...\Profile_1.cfg` | 1,917 | `7C1D7978C9587B0CB3A6D5A5005171763CD3A88AD7CECD18EB382BB63311344A` | INI profile with MS-7E75 sections. |
| `C:\MSI...\Profile_2.cfg` | 1,909 | `D706498D8A4E56A5419018FFD2E94F073FFDCBFF247592E7BF13EF3CAA995164` | INI profile with MS-7E75 sections. |
| `C:\MSI...\ML3.cfg` | 724 | `683F5A84A3F0178C20274946B887A29509D1E4CACB39E98B60AE0D37E6B217AF` | Sync-style list containing `MS-7E75_1`. |

Packaged `Profile\*.tmp` files under `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Profile`:

| File | Size | SHA-256 | Header |
| --- | ---: | --- | --- |
| `95324862-6f3f-47ab-a988-a97cae000001.tmp` | 1,496 | `D9EF6F1F499D31F4BE183951ECC7F4EE26140C64B53333AD24A8805ED90B686B` | `DF BD 50 36 8D 48 6C B5 16 32 16 3B 07 EB C1 EA` |
| `95324862-6f3f-47ab-a988-a97cae000007.tmp` | 1,288 | `102D39FB430B820980A30E62EB250B770E0A893F10BA445FB4E4620F2852CDFD` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000009.tmp` | 1,488 | `8741A2FC603180005FF3CAD9B12FAC55B0BFD67C5ED6469E0CB83B33BD7501FB` | same as above |
| `95324862-6f3f-47ab-a988-a97cae00000b.tmp` | 1,584 | `CAA469B94BC3AEB07B76986620B496A90DE30EDB7E91943C253B406DE55F8045` | same as above |
| `95324862-6f3f-47ab-a988-a97cae00000c.tmp` | 1,536 | `ADEDD4472EFBE65C7166FE17E6A3889E996ACFD1F08EBC8C0DA5A76553A1A193` | same as above |
| `95324862-6f3f-47ab-a988-a97cae00000d.tmp` | 1,296 | `E9BB383125616B859655FB2B596DF5B47D04397686C24CDE8B07791BD5CADCE2` | same as above |
| `95324862-6f3f-47ab-a988-a97cae00000e.tmp` | 1,200 | `02AFF792E8A235F3CCFD1AACFC1BE1E7D3B44231098D6CE697BA39DAAD28BD22` | same as above |
| `95324862-6f3f-47ab-a988-a97cae00000f.tmp` | 1,248 | `323152E439D0ECD0F9A36838A83E87E358330D6754C03AEE654171B095F15817` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000010.tmp` | 1,448 | `C5BAC1457F2BF741CFE48E60689E77D7A33DCE73039CC4F81927B9EE7EF8CBFC` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000011.tmp` | 1,720 | `EB68A55DEC5EC7721745509FC2FFED190A962A52EF17852AC5D890179588C119` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000012.tmp` | 1,144 | `BB3FDECDB062D17F60E11FE8E2F17181003BD0D1E391A458EF5C9CD4982A1578` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000013.tmp` | 1,544 | `E7B2157225442A68F285B97478571705D7DAB3AD9B630AC03DB07CBFF6048126` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000014.tmp` | 1,520 | `11F77214A50C090132DDA19D0546A1E1F8CC31F5F96B2DBC857E13450AC308CE` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000015.tmp` | 1,184 | `C9D2BC02B5AB2345076304574A2F72F75476B844C89C72ABA4D75F27E9F06961` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000016.tmp` | 1,872 | `AB9541E4927A97033202D936E21438CA2256068D844128C7164C925A1FC06A05` | same as above |
| `95324862-6f3f-47ab-a988-a97cae000018.tmp` | 1,352 | `8E2B9400511A34890D7B53452B13CF3681AAB54B0685B72C3F5262EF275F7D39` | same as above |
| `95324862-6f3f-47ab-a988-a97cae78415d.tmp` | 19,968 | `ADF521CFD206DF3C7B37ACD0882D1090F7CF5D8E48EEC39F013D151F40FE4CF6` | same as above |
| `95324862-6f3f-47ab-a988-a97cae78415e.tmp` | 28,696 | `AEAF8D06289D38AADEFA5C5C0567486260A67F05E041F080238F9CBA3F492144` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784160.tmp` | 35,368 | `A8186139200E4D13C80AE182E6930C457AB1D982D69E7DF8527817CEFE76D53F` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784161.tmp` | 26,688 | `D94AB63DFC98F0AF5D383E76063F43FD116949845B275082D787A317BEC25EC8` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784162.tmp` | 6,768 | `923547F69227A93B9727053702A463BCBC5A92BD880554E5F7807AB72E4E9764` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784163.tmp` | 6,944 | `9529012BA8626F454A10FC6093376A4EC35E10FC843C26E0453A80A013AFDF8B` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784164.tmp` | 7,912 | `79E713B35B0581AAC02A027BA350E1EC47168EB518C069097116AEA643A158D8` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784165.tmp` | 13,720 | `A9D33C8DBC63AEE30AC53F6CA6D2B703CE2CF0946006D4002F4518A4DEA3A30F` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784166.tmp` | 5,656 | `D9D5B2306F88495CC2ABA2E2D2473D7EF5AB0F7CB8AF8D67D75535C68D85F01E` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784167.tmp` | 6,784 | `489D79D18290F479D31E30BCA05B622419FA59AA91515F7D390A299BDDDF82AC` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784168.tmp` | 23,328 | `9BB332E0583AD6F631CC2C6BD6A4547425C2989ED7B33236B58E0AB612691C64` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784169.tmp` | 4,656 | `839A0E1A00747D76D4E8D8FF1531DC6473A533884CD4ABF928179A0844228BED` | same as above |
| `95324862-6f3f-47ab-a988-a97cae78416a.tmp` | 6,320 | `ECE3F32DDEA5E09322D9BA89963A688A60F9434F2A9BCE9CD00C2C4233F47367` | same as above |
| `95324862-6f3f-47ab-a988-a97cae78416b.tmp` | 5,600 | `5604FC6737C6ACBB98F0C732B51487B138220242A87C619E5DF0441B37755C9C` | same as above |
| `95324862-6f3f-47ab-a988-a97cae78416c.tmp` | 4,936 | `359A0650627FFD8C55C56591069AE9B6AF40EFA1553CE10F2954E44F0526B446` | same as above |
| `95324862-6f3f-47ab-a988-a97cae78416d.tmp` | 15,488 | `3FD4F3D2D4F7D76386B46485377029957CC6B4C3465538D96B9D53BAF92A2E66` | same as above |
| `95324862-6f3f-47ab-a988-a97cae78416e.tmp` | 11,312 | `BABA8AF8355D1B416D6721346DBAE0F12998EBB7032E3AB9C174224F2B7EA304` | same as above |
| `95324862-6f3f-47ab-a988-a97cae78416f.tmp` | 20,568 | `DFBDFF3757DC24E48C0F78228C605329DE4227FA3F73E15640778BF82C73B6A5` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784170.tmp` | 19,296 | `BAF31E41A95CB68B1CF9D6C8FACE944D93D3ED8BF995A4D9BF39A45C9BCF6A18` | same as above |
| `95324862-6f3f-47ab-a988-a97cae784171.tmp` | 27,184 | `90CF892A2307213F03C81C87327B277C91B1FEB81F5B90D6C8883513D1FDC54D` | same as above |

## Decode Method

The decode path was recovered from static `LEDKeeper2.exe` decompilation of `MSI_LED.C_Encrypt` and `Class_ParseCfg.ParseCfgFile`.

Reimplemented logic:

1. Read the file as UTF-8 text.
2. Require `!!MSI!!` prefix.
3. Remove the first seven characters.
4. Base64-decode the remainder.
5. Compute key text as `232345599.ToString("X")`, which is `DD94FFF`.
6. Use AES-CBC with PKCS#7 padding.
7. Use `SHA256(UTF8("DD94FFF"))` as AES-256 key.
8. Use `MD5(UTF8("DD94FFF"))` as the 16-byte IV.
9. Decode plaintext as UTF-8.

No MSI decryption code was executed. The algorithm was reimplemented with PowerShell/.NET cryptography primitives against data files only.

Decoded online-data results:

| Source | Cipher bytes | Plain bytes | Plain SHA-256 | Lines | `[SyncData]` entries | `MS-7E75` hits |
| --- | ---: | ---: | --- | ---: | ---: | ---: |
| `Mystic Light\Mystic Light Online Data.dat` | 339,600 | 339,584 | `E85E30C7195C05E447D4C55556A42CB4053FA9645CFBF5D6D24B3B9E88EBB7DF` | 2,050 | 1,038 | 4 |
| `Data\Mystic Light Online Data.dat` | 337,600 | 337,599 | `7CACC4A29C8AFE07BD0335A3D20377E2B835A84636F67A9667DB4E26E074BA0A` | 2,015 | 1,030 | 4 |

The two decoded online-data copies are similar but not byte-identical.

## Evidence Table

| Source | Evidence | MS-7E75 relevance | Limitations |
| --- | --- | --- | --- |
| `C_Encrypt.DecryptBase64` decompile | Uses `!!MSI!!` + base64 + AES-CBC/PKCS#7 with SHA-256 key and MD5 IV derived from `DD94FFF`. | Provides a static, reproducible decode method for Mystic Light online data. | Applies to `!!MSI!!` online data; not proven for `.tmp` files. |
| `Mystic Light\Mystic Light Online Data.dat` decoded `[SyncData]` | Contains `MS-7E75_1` and `MS-7E75_2` records. Each record includes zones `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, `EZ Conn`, and `SELECT ALL`. | Static profile-source proof for the observed MS-7E75 profile token and zone labels. | Opaque numeric fields are not decoded to transport/register semantics. |
| `Data\Mystic Light Online Data.dat` decoded `[SyncData]` | Also contains `MS-7E75_1` and `MS-7E75_2` with the same visible zone set. | Confirms the downloaded/data copy has the same MS-7E75 support/profile records. | Same limitation: no backend/register mapping. |
| Decoded online data term counts | `JRGB`, `JRAINBOW`, and `JARGB` terms are broadly present; `Renesas`, `SMBus`, and `MBAPI` are absent in both decoded copies. | Suggests the online data is profile/zone/support data rather than a named transport table. | Absence of names does not prove absence of backend flags in opaque numeric fields. |
| `MS-7E75_1` decoded record | Zone field pattern includes `JRGB1,09,69,...`, `JARGB_V2_1,00,69,...`, `JARGB_V2_2,01,69,...`, `JARGB_V2_3,02,69,...`, `EZ Conn,03,69,...`, `SELECT ALL,11,69,...`. A follow-up decompile pass resolves `69` as `EnumChipest.NUC126_MB800`. | Proves concrete zone labels and ties the decoded records to the MB800 managed device path. | `10`, `3+2`, `5+5`, and hex style masks are profile/effect fields, not decoded registers. The MB800 tie still does not prove a raw backend/register map. |
| `Profile_v2.txt` | JSON `DeviceData` includes `MS-7E75_1_JRGB1`, `MS-7E75_1_JARGB_V2_1`, `_2`, `_3`, `MS-7E75_1_EZ Conn`, and `MS-7E75_1_SELECT ALL`; each motherboard zone has 11 style entries and selected index 10, except `SELECT ALL` selected index 2. | Confirms local profile state was built from the same MS-7E75 zone model. | Runtime/user profile state, not original packaged support source and not backend proof. |
| `Profile_1.cfg` / `Profile_2.cfg` | INI profiles contain `MS-7E75_1_0` through `MS-7E75_1_5`, plus `CurrentSyncList=Style_Button_MB|MS-7E75_1,...`. | Confirms six local motherboard profile slots matching six decoded zones. | Numeric slot-to-zone mapping is inferred from decoded zone order, not explicitly named in these INI sections. |
| `ML3.cfg` | Contains `ListString=MS-7E75_1,10DE2F0453221462|0`. | Confirms local Mystic Light profile grouping includes the MS-7E75 board token. | No zone/backend details. |
| Packaged `Profile\*.tmp` files | High entropy, stable common header for most files, no ASCII/UTF-16 hits for target board/zone/backend terms. | No direct MS-7E75 evidence found in `.tmp` resources. | File format remains unknown. Some binary false-positive `EC` byte/UTF artifacts were ignored as non-evidence. |
| `loader.tmp` | High entropy, different header from other `.tmp` files, no target hits. | No MS-7E75 evidence found. | File format remains unknown. |

## MS-7E75 / Zone / Backend Hits

Confirmed MS-7E75 static profile-data hits:

- `MS-7E75_1`
- `MS-7E75_2`
- `JRGB1`
- `JARGB_V2_1`
- `JARGB_V2_2`
- `JARGB_V2_3`
- `EZ Conn`
- `SELECT ALL`

Confirmed local profile-state hits:

- `MS-7E75_1_JRGB1`
- `MS-7E75_1_JARGB_V2_1`
- `MS-7E75_1_JARGB_V2_2`
- `MS-7E75_1_JARGB_V2_3`
- `MS-7E75_1_EZ Conn`
- `MS-7E75_1_SELECT ALL`

Not found as meaningful MS-7E75 backend evidence:

- No decoded `Renesas` hit.
- No decoded `SMBus` hit.
- No decoded `MBAPI` hit.
- No decoded MS-7E75 EC offset, SIO register, SMBus address, HID report, IOCTL sequence, command payload, or register map.
- `B850` appears in the newer `Mystic Light` copy only as a GPU-like identifier line, not as the MS-7E75 motherboard marketing name.
- `GAMING PLUS` appears in other board/image/GPU records, not as a decoded B850 GAMING PLUS WIFI PZ motherboard record.

## Confirmed Vs Unknown

Confirmed:

- The recovered `Class_ParseCfg` / `C_Encrypt` decode path works on installed `Mystic Light Online Data.dat` files.
- Both decoded online-data copies contain `[SyncData]` records for `MS-7E75_1` and `MS-7E75_2`.
- The decoded MS-7E75 records statically define the visible zone set `JRGB1`, `JARGB_V2_1`, `JARGB_V2_2`, `JARGB_V2_3`, `EZ Conn`, and `SELECT ALL`.
- A follow-up static call-path pass resolves the decoded chipset byte `69` to `EnumChipest.NUC126_MB800` and maps the zones into `Class_MB_800` / `MSI_800sLed` helper calls. See [MSI_7E75_ZONE_CALLPATH_STATIC_RE.md](MSI_7E75_ZONE_CALLPATH_STATIC_RE.md).
- Local profile files under `C:\MSI\MSI Center\Mystic Light\Profile` contain MS-7E75 profile state using the same zone names.
- The decoded data provides the missing static source for `MS-7E75_1` and the observed Mystic Light zone labels.

Unknown:

- The remaining meaning of profile/effect fields such as default style index `10`, speed `3+2`, brightness `5+5`, and suffix `+1301`.
- Whether the MB800 helper path is the complete live MS-7E75 runtime path, and how its lower HID helper opens the final physical device.
- Whether MBAPI, SMBus/Renesas, EC, SIO, ACPI/WMI, or another backend participates before or beside the MB800 helper path.
- Whether MBAPI's static `7E75` board-list hit is used together with these decoded profile records.
- The binary format and purpose of packaged `Profile\*.tmp` and `loader.tmp`.

## Next Static-Only Targets

- Continue static MB800 call-path work from [MSI_7E75_ZONE_CALLPATH_STATIC_RE.md](MSI_7E75_ZONE_CALLPATH_STATIC_RE.md), especially `HID_Basic`, `MSI_800sLed.CheckConnectedDevice`, and initialization support-list construction.
- Cross-reference the MS-7E75 style/effect mask `1342D02C23469345A74401`, default index `10`, and suffix `+1301` against MB800 style parsers.
- Continue static MBAPI work around the confirmed `7E75` board-ID list and `CheckMBVersion` / `SupportLED` consumers.
- Identify the `.tmp` binary format statically, but treat it as lower priority now that online data gives direct MS-7E75 profile records.

## Explicit Hardware-Access Note

No MSI binaries were executed during this pass. MSI Center, Mystic Light, and `LEDKeeper2.exe` were not launched. No hardware access was enabled or run: no doctor command, no chip detection, no register read, no write/apply command, no `/dev/port`, no raw SMBus, no raw Super I/O, and no MS-7E75 hardware support code changes.
