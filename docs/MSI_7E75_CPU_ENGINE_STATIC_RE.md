# MSI MS-7E75 CPU Engine Static Reverse Engineering Notes

Status: research-only, static analysis only.

## Scope

This note records a static-only Ghidra/headless pass over MSI Center Mystic Light's `CPU_Engine.dll` companion module. The goal was to determine whether this module explains any MS-7E75 Mystic Light control path, or whether it is a CPU telemetry/tuning helper unrelated to board RGB.

These notes are evidence for future research only. They do not add or enable MS-7E75 hardware support.

## Safety Constraints

- No MSI Center process was started.
- No `doctor`, `detect-chip`, `read-reg`, write, or apply command was run.
- No `/dev/port` access was attempted.
- No raw SMBus or Super I/O access was attempted.
- No MS-7E75 hardware access was enabled.
- The target DLL was imported into Ghidra/headless for static analysis only. A byte-identical temporary copy was used for headless import to avoid Windows batch quoting problems with `Program Files (x86)`.

## Analyzed Binary

| Field | Value |
| --- | --- |
| Original path | `C:\Program Files (x86)\MSI\MSI Center\Mystic Light\Lib\CPU_Engine.dll` |
| Size | `2096304` bytes |
| Last modified | `2019-12-23 16:17:12` |
| SHA-256 | `3F5243C2A105467FE2D1672E939868671B025D61E28538E833EF63B224C6F1EC` |
| Ghidra language | `x86:LE:32:default` |
| Compiler spec | `windows` |
| Image base | `0x10000000` |
| Ghidra function count | `12354` |
| Ghidra string count | `2915` |
| Ghidra import count | `601` |

## Evidence Table

| Area | Evidence | Assessment |
| --- | --- | --- |
| Module purpose | Exports are CPU-centric: `CPUInitialization`, `CPURelease`, `GetCPUInfo`, `GetCPUCoreTemp`, `GetCPUCoreAndThread`, `GetCoreClock`, `GetTDP`, `GetCPUMutilper`, `GetCPUTurboRatio`, `SetCPUMutilper`, `SetCPUTurboBoost`, `SetPowerLimit`, `SetCurrentLimit`, `AMD_READ_SMU`, and `_AMD_WRITE_SMU@12`. | Strong evidence this module is for CPU telemetry and CPU tuning/overclocking, not Mystic Light LED control. |
| Telemetry | Exports include core/thread, core temperature, Tjmax, core clock, core count, TDP, multiplier, ring multiplier, and APU frequency getters. | Strong evidence of CPU telemetry/readback support. |
| Control/tuning | Exports include CPU multiplier, turbo boost, turbo ratio, current limit, power limit, ring multiplier, PB0/PB1/PB2 ratio, and mailbox setters. | Strong evidence it is not telemetry-only; it exposes CPU tuning/control entry points. |
| AMD evidence | Exports include `AMD_READ_SMU` and `_AMD_WRITE_SMU@12`; strings include AMD feature/family text such as `AMD K15`, `Threadripper`, `Socket AM4 (1331)`, `Socket SP3r2 (4094)`, `Socket SP3r3 (4094)`, `AMD-V`, and AMD instruction feature summaries. | Strong evidence of AMD CPU/SMU support. |
| Intel evidence | Strings include `Intel Haswell`, `Intel Broadwell-E/EP`, `Intel Haswell-E/EP`, `Intel Ivy Bridge`, `Intel Sandy Bridge`, `Intel Comet Lake`, `Intel SkyLake`, `Intel Coffee Lake`, `Intel KabyLake`, `Intel SkyLake-X`, `Intel Cascade Lake-X`, `LGA1150`, `LGA1151`, `LGA1200`, `LGA2011`, `LGA2066`, and `Global\Access_Intel_OC_Mailbox`. | Strong evidence of Intel CPU and overclocking-mailbox support. |
| CPUID/MSR strings | Targeted plain-string search found no `CPUID`, `cpuid`, `MSR`, `rdmsr`, or `wrmsr` strings. | The binary clearly has CPU-identification behavior by exports/CPU-family tables, but this pass did not name a CPUID/MSR string path. Inline instructions or abstracted object methods remain possible. |
| Shared command wrapper | Exported wrappers check initialized global state, call an internal parameter-staging function `FUN_1000ec00(...)`, then call execution/result helpers such as `FUN_1000e6a0()` and `FUN_1000dd90(...)`. | Strong evidence of a common internal CPU command dispatch layer. Exact backend implementation remains unresolved in this pass. |
| Driver Engine references | No `Driver_Engine`, `NTIOLib`, or `NTIOLib_MysticLight` strings were found. | No static string evidence tying this DLL directly to Driver Engine or NTIOLib Mystic Light. |
| EC references | No `GetECSpace`, `SetECSpace`, or `ECSpace` strings were found. The broad `EC` substring count is from unrelated words and was not evidence of EC-space APIs. | No evidence that this module exposes MSI EC get/set-space paths. |
| Fan/PWM/control strings | Targeted string counts found `fan=0`, `Fan=0`, `PWM=0`, `pwm=0`, `temperature=0`, `Temperature=0`, `voltage=0`, `Voltage=0`, and `Vcore=0`; only generic `Temp`/`Core` hits appeared. | No static evidence of fan, PWM, or motherboard voltage-control strings. CPU temperature/core strings are expected from CPU telemetry. |
| Mystic Light/RGB strings | No `Mystic`, `ARGB`, `JRGB`, or `JRAINBOW` strings were found. `RGB` hits came from MFC/UI helper strings such as `RGB(%d, %d, %d)` and `commdlg_SetRGBColor`, not lighting-control code. | No evidence that this DLL implements Mystic Light RGB header control. |
| Board strings | No `MS-7E75`, `7E75`, or `B850` strings were found. | No board-specific evidence for MS-7E75. |
| Device/service APIs | Imports include broad Windows/MFC APIs and registry APIs. Targeted search found no `DeviceIoControl`, `CreateService`, `OpenService`, or `StartService` imports/strings. `CreateFileW` exists, but references appear in MFC/CRT file helpers and UI/framework code. | No direct service/IOCTL transport path was visible from this pass. |
| MBAPI relationship | Earlier MBAPI static notes listed `\CPU_Engine.dll` as a companion module string. This DLL itself has no `MBAPI` string. | MBAPI may load/use this module dynamically, probably for CPU information/tuning, but this pass found no reverse dependency or Mystic Light-specific purpose. |

## Exports, Imports, Functions, and Strings

Notable exports:

- Initialization/lifetime: `CPUInitialization`, `CPURelease`.
- CPU information/telemetry: `GetCPUInfo`, `GetCPUCoreAndThread`, `GetCPUCoreTemp`, `GetCPUCoreTempTjmax`, `GetCoreClock`, `GetCoreNum`, `GetAPUFrequency`, `GetTDP`.
- Ratio/frequency controls: `GetCPUMutilper`, `GetCPUsetMutilper`, `GetCPUPerCoreRatio`, `GetCPUMaxMinRatio`, `GetCPUTurboRatio`, `SetCPUMutilper`, `SetCPUTurboBoost`, `SetCPUTurboRatio`, `GetRingMutilper`, `GetRingsetMutilper`, `SetRingMutilper`, `SetPB0Ratio`, `SetPB1Ratio`, `SetPB2Ratio`.
- Power/current/mailbox controls: `SetPowerLimit`, `SetCurrentLimit`, `GetMailBoxData`, `GetMailBoxOffsetData`, `SetMailBoxData`, `SetMailBoxOffsetData`.
- AMD SMU helpers: `AMD_READ_SMU`, `_AMD_WRITE_SMU@12`.

Notable imports/categories:

- `KERNEL32.DLL`: thread, file, heap, module, and runtime helpers including `CreateFileW`, `LoadLibraryA/W`, `LoadLibraryExW`, `GetProcAddress`, and `CloseHandle`.
- `ADVAPI32.DLL`: registry helpers such as `RegOpenKeyExW`, `RegQueryValueExW`, `RegSetValueExW`, and related registry enumeration/delete APIs.
- UI/framework libraries: `USER32.DLL`, `GDI32.DLL`, `GDIPLUS.DLL`, `MSIMG32.DLL`, `SHELL32.DLL`, `SHLWAPI.DLL`, `UXTHEME.DLL`, `OLE32.DLL`, `OLEAUT32.DLL`, `OLEACC.DLL`, `WINMM.DLL`, and `WINSPOOL.DRV`.

Notable strings:

- CPU families/platforms: `AMD K15`, `Threadripper`, `Socket AM4 (1331)`, `Socket SP3r2 (4094)`, `Socket SP3r3 (4094)`, `Intel Haswell`, `Intel Broadwell-E/EP`, `Intel Haswell-E/EP`, `Intel Ivy Bridge`, `Intel Sandy Bridge`, `Intel Comet Lake`, `Intel SkyLake`, `Intel Coffee Lake`, `Intel KabyLake`, `Intel SkyLake-X`, `Intel Cascade Lake-X`.
- CPU/socket strings: `LGA1150`, `LGA1151`, `LGA1200`, `LGA2011`, `LGA2066`, `Radeon`.
- Overclocking/mailbox string: `Global\Access_Intel_OC_Mailbox`.
- Export/name strings: `CPU_Engine.dll`, `AMD_READ_SMU`, `_AMD_WRITE_SMU@12`, `GetCPUCoreAndThread`, `GetCPUCoreTemp`, `GetCPUCoreTempTjmax`, `GetCPUPerCoreRatio`, `GetCoreClock`, `GetCoreNum`.

Strings not found in this pass:

- `MS-7E75`
- `7E75`
- `B850`
- `JRGB`
- `JRAINBOW`
- `ARGB`
- `Mystic`
- `Driver_Engine`
- `NTIOLib`
- `NTIOLib_MysticLight`
- `GetECSpace`
- `SetECSpace`
- `ECSpace`
- `fan`, `Fan`
- `PWM`, `pwm`
- `DeviceIoControl`
- `CreateService`
- `OpenService`
- `StartService`
- `MBAPI`
- `SMBus_Engine`

## Candidate Driver, Device, and Control Paths

No direct Mystic Light driver/device/IOCTL path was confirmed in this pass.

Candidate CPU-control paths:

| Candidate | Evidence | Notes |
| --- | --- | --- |
| CPU command dispatch object | Exported wrappers require `DAT_101cf270 != 0`, initialized by `CPUInitialization`; wrappers stage command IDs and parameters through `FUN_1000ec00(...)`, then call execution/result helpers. | Likely internal CPU telemetry/tuning backend. Exact implementation needs deeper static analysis of initialization and virtual/object call targets. |
| AMD SMU access | `AMD_READ_SMU` stages command `0x18`; `_AMD_WRITE_SMU@12` stages command `0x19`; both use the same initialized dispatch path. | Strong evidence of AMD SMU read/write support, but not evidence of board RGB. |
| Intel OC mailbox | Export strings and CPU strings include `Global\Access_Intel_OC_Mailbox`; exports include mailbox get/set helpers. | Strong evidence of Intel overclocking mailbox support. |
| Power/current/ratio setters | `SetPowerLimit`, `SetCurrentLimit`, `SetCPUMutilper`, `SetCPUTurboBoost`, `SetCPUTurboRatio`, and PB ratio setters are exported. | This DLL is not CPU telemetry only; it includes CPU tuning/control entry points. |

Paths not confirmed:

- No direct `DeviceIoControl` import/string.
- No direct NTIOLib or Driver Engine string.
- No direct MSI EC `GetECSpace`/`SetECSpace` string.
- No service creation/open/start strings.
- No Mystic Light RGB/JRGB/JRAINBOW path.

## Confirmed vs Unknown

Confirmed:

- `CPU_Engine.dll` is a 32-bit Windows DLL with CPU telemetry and CPU tuning exports.
- It includes AMD SMU read/write exports.
- It includes Intel and AMD CPU family/socket/feature strings.
- It includes Intel overclocking mailbox evidence.
- It includes power, current, multiplier, turbo, PB ratio, and mailbox setter exports.
- It contains no MS-7E75/B850/JRGB/JRAINBOW/Mystic strings in this pass.
- It contains no direct `Driver_Engine`, `NTIOLib_MysticLight`, `GetECSpace`, `SetECSpace`, `DeviceIoControl`, or Windows service-control strings in this pass.
- No hardware access was enabled or run.

Unknown:

- Which exact MBAPI code path loads and calls `CPU_Engine.dll`.
- Whether the lower-level CPU backend uses a driver, model-specific registers, CPUID instructions, SMU mailbox, Intel OC mailbox, or another abstraction internally.
- Whether any CPU tuning path is ever used by Mystic Light installs, or whether this DLL is bundled because MSI Center shares components across features.
- Whether the missing CPUID/MSR strings reflect inline instructions, static library code, or an abstraction that does not name those APIs in strings.

## Relevance to MS-7E75 Mystic Light Path

This module looks likely unrelated to the MS-7E75 Mystic Light RGB path. It appears to be a CPU information and CPU tuning helper with AMD SMU and Intel OC mailbox support. It does not contain MS-7E75, B850, Mystic Light, JRGB, JRAINBOW, ARGB, Driver Engine, NTIOLib Mystic Light, or MSI EC-space strings.

The only current relationship to the MS-7E75 research graph is that earlier MBAPI static notes listed `\CPU_Engine.dll` as a companion module string. That is not enough evidence to claim MS-7E75 uses this module for lighting.

## Open Questions

- Which MBAPI functions load `CPU_Engine.dll`, and are they CPU-monitoring/tuning features rather than Mystic Light features?
- What does `CPUInitialization` construct at `DAT_101cf270`, and which virtual/object methods execute the staged commands?
- Does the CPU backend use an MSI/third-party driver that is not named by strings, or does it rely on user-mode CPU instruction paths?
- Are AMD SMU command IDs `0x18` and `0x19` generic read/write selectors in MSI's wrapper?
- Is `Global\Access_Intel_OC_Mailbox` only a synchronization object, or part of a broader Intel tuning transport?

## Next Static-Analysis Tasks

- In MBAPI, decompile dynamic load/call sites for `CPU_Engine.dll` and identify the feature gate around CPU exports.
- In `CPU_Engine.dll`, trace `CPUInitialization`, `FUN_1000c7c0`, `FUN_1000e6a0`, and the virtual/object method table behind `DAT_101cf270`.
- Search for inline `cpuid`, `rdmsr`, `wrmsr`, `in`, and `out` instructions in Ghidra rather than relying on strings.
- Cross-check whether MSI Center UI/config files reference `CPU_Engine.dll` from hardware-monitoring or overclocking pages rather than Mystic Light.
- Continue keeping CPU Engine evidence separate from MS-7E75 lighting claims until a board-specific or Mystic Light-specific selector is found.

## Explicit Safety Note

No hardware access was enabled or run. This pass used static Ghidra/headless analysis of `CPU_Engine.dll` only.
