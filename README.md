# boardcontrol

![Status](https://img.shields.io/badge/status-read--only%20hardware%20detection-yellow)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Hardware writes](https://img.shields.io/badge/hardware%20writes-not%20implemented-red)

Open-source experimental alternative to MSI Center / Mystic Light for low-level LED initialization control on the MSI motherboard family `7A45`.

The current codebase is a safety-first research MVP. It models known register-level behavior but does not perform real hardware writes yet.

## Project Status

Current MVP status:

- Rust CLI
- trace backend for safe sequence simulation
- experimental Linux read-only Super I/O chip detection
- no real LED hardware writes yet
- supports only MSI board profile `7A45`
- models the Nuvoton NCT6779D LED init/reset sequence
- includes safe RMW allowlist logic
- passes `cargo check`, `cargo test`, and `cargo clippy -- -D warnings`

## Supported Hardware Status

| Board | Super I/O | Renesas SMBus | Status |
| --- | --- | --- | --- |
| `7A45` | `Nuvoton NCT6779D` | `0x52` | `Trace simulation + experimental Linux read-only chip detection` |

## Architecture

MSI 7A45 LED control paths:

- NCT6779D Super I/O through ports `0x4E / 0x4F`
- Renesas LED controller through Intel SMBus address `0x52`

MVP structure:

- `TraceBackend`
- board profile
- NCT allowlist
- RMW executor
- CLI commands

## Safety Model

- no blind writes
- all NCT writes are modeled as read-modify-write
- every changed bit must be allowed by `(LDN, REG, allowed_change_mask)`
- unknown boards are unsupported
- real hardware writes are intentionally not implemented yet

```text
new_value = (current & and_mask) | or_mask
changed = current ^ new_value

if changed & !allowed_change_mask != 0:
    block
else:
    write
```

## Current CLI

```bash
cargo run -- detect --board 7A45
cargo run -- nct init-7a45 --dry-run
cargo run -- nct reset-led --dry-run
```

Commands without `--dry-run` are intentionally not implemented yet.

## Test Commands

```bash
cargo fmt
cargo check
cargo test
cargo clippy -- -D warnings
```

## Roadmap

- [x] Trace-only Rust CLI MVP
- [x] 7A45 NCT init/reset sequence model
- [x] Safe RMW allowlist tests
- [x] Linux read-only NCT6779D chip detection
- [ ] Linux `/dev/port` backend for controlled NCT RMW writes
- [x] `/proc/ioports` conflict checks
- [ ] Renesas SMBus raw write backend
- [ ] Renesas RGB/mode mapping
- [ ] Windows backend

## Experimental Read-Only Hardware Detection

```bash
cargo run -- nct detect-chip --backend dev-port --confirm-read
```

This command only performs Super I/O config-mode register reads for chip identification. It does not execute LED init/reset writes.

Linux only. Requires permission to access `/dev/port`. The command refuses to run without `--confirm-read`.

## Legal / Project Note

This project does not include MSI binaries, MSI drivers, MSI logos, or decompiled MSI source code. It is an independent clean-room implementation based on observed hardware behavior and register-level research.
