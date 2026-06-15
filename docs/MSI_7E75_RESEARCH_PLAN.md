# MSI MS-7E75 Research Plan

Status: planning only, hardware access disabled.

Board:

- MSI B850 GAMING PLUS WIFI PZ
- MS-7E75
- Known DMI validated from Fedora Live doctor output
- Recognized by code as `looks_like_msi_7e75 = true`
- Hardware access remains blocked

Purpose:

Define what must be learned before this board can ever receive a real hardware profile.

Hard rules:

- MS-7E75 must not reuse the 7A45 NCT register map.
- MS-7E75 must not pass hardware-read preflight until a separate profile exists.
- detect-chip/read-reg/write/apply must remain blocked.
- Any future hardware probing must be explicitly documented, reviewed, and limited.

Research phases:

Phase 0 - Current state:

- DMI detection exists.
- Fedora Live doctor validation exists.
- Hardware access is blocked.
- No raw hardware reads were run.

Phase 1 - Static information only:

- Collect exact motherboard name and revision.
- Collect BIOS version.
- Collect public MSI support/manual/spec links.
- Record static MSI Center / Mystic Light evidence in [MSI_7E75_STATIC_RE.md](MSI_7E75_STATIC_RE.md).
- Record direct Driver Engine transport, service, device, and IOCTL evidence in [MSI_7E75_DRIVER_ENGINE_STATIC_RE.md](MSI_7E75_DRIVER_ENGINE_STATIC_RE.md).
- Record SMBus Engine transaction/controller-selection evidence in [MSI_7E75_SMBUS_ENGINE_STATIC_RE.md](MSI_7E75_SMBUS_ENGINE_STATIC_RE.md).
- Record RTK Bridge Realtek bridge/device-handle evidence in [MSI_7E75_RTK_BRIDGE_STATIC_RE.md](MSI_7E75_RTK_BRIDGE_STATIC_RE.md).
- Record CPU Engine CPU telemetry/tuning evidence in [MSI_7E75_CPU_ENGINE_STATIC_RE.md](MSI_7E75_CPU_ENGINE_STATIC_RE.md).
- Consolidate static reverse-engineering findings and module relevance in [MSI_7E75_STATIC_RE.md](MSI_7E75_STATIC_RE.md).
- Record MSI Center / Mystic Light profile-selection search evidence in [MSI_7E75_PROFILE_SELECTION_STATIC_RE.md](MSI_7E75_PROFILE_SELECTION_STATIC_RE.md).
- Identify likely Super I/O, EC, RGB, fan, and sensor controller families from public information only.
- Record uncertainty instead of guessing.

Next static-only targets:

- Decompile `LEDKeeper2.exe` support/profile logic around `Support list`, `ResetItem`, `RGBControlClass`, `JARGB_V2`, and board/profile construction.
- Decompile `MBAPI_x86.dll` around the `7E75` board-ID list to identify table consumers and dispatch effects.
- Reverse the `!!MSI!!` encoded `Mystic Light Online Data.dat` format and the `Mystic Light\Profile\*.tmp` binary profile blobs.
- Cross-reference MBAPI call sites that pass arguments into `DriverInitialization` and `SMBusInitialization`.
- Cross-reference MBAPI callers of `SMB_*`, `b_SMB_*`, and `n_SMB_*` to identify Renesas/Mystic Light call families.
- Build static vtable maps for `SMBus_Engine.dll` `IntelSMBus` and `ATISMBus` backends.
- Statically inspect `NTIOLib.sys` / `NTIOLib_X64.sys`, if available, for IOCTL dispatch and device names.

Phase 2 - OS-visible inventory only:

- Collect OS-visible PCI/USB/ACPI/SMBus device inventory.
- Prefer standard OS listings and logs.
- Do not use /dev/port.
- Do not use raw Super I/O register reads.
- Do not use raw SMBus pokes.

Phase 3 - Read-only sensor correlation:

- Compare values from safe existing tools such as BIOS hardware monitor, HWiNFO, lm-sensors, or LibreHardwareMonitor.
- Treat this as observation, not register-map proof.
- Do not infer write behavior from sensor labels.

Phase 4 - Minimal hardware-read proposal:

- Only after phases 1-3 are documented.
- Must define exactly what will be read, why, from which backend, and how it is bounded.
- Must include stop conditions.
- Must be reviewed before implementation.

Phase 5 - Board profile proposal:

- Only after chip identity and register behavior are independently established.
- Add an explicit MS-7E75 profile separate from 7A45.
- Keep read-only support separate from any future write/apply support.

Open questions:

- What Super I/O chip is actually present?
- Is fan control handled by Super I/O, EC, AMD chipset path, or MSI firmware layer?
- What controls RGB on this board?
- Are sensors exposed through standard Linux hwmon?
- Does OpenRGB identify any relevant controller?
- Does MSI Center use SMBus, ACPI/WMI, HID/USB, or another path for this board?

Exit criteria before any hardware-read code:

- Board identity documented.
- Controller identity documented or explicitly unknown.
- Static/public evidence collected.
- Safe OS-visible inventory collected.
- Proposed read scope reviewed.
- 7E75 profile remains separate from 7A45.
