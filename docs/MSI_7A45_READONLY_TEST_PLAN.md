# MSI 7A45 Read-Only Test Plan

## Goal

The goal of this test is to validate read-only hardware access on a real MSI 7A45 board without performing any hardware writes.

## Rules

* do not run write/apply commands
* do not add temporary raw port commands
* do not bypass DMI
* do not bypass `/proc/ioports`
* do not read arbitrary registers
* only use allowlisted read-only commands
* stop immediately if `doctor` blocks

## Step 1: Update repository

```bash
cd ~/boardcontrol
git pull
git status -sb
git log --oneline -5
```

Expected top commit should be current `main`.

## Step 2: Safe software regression

```bash
cargo test
cargo clippy -- -D warnings
cargo run -- nct plan-init-7a45
cargo run -- nct init-7a45 --dry-run
```

Expected:

* tests pass
* clippy clean
* plan/dry-run output uses TraceBackend only

## Step 3: Doctor preflight

```bash
cargo run -- doctor
```

Expected PASS only if:

```text
looks_like_msi_7a45 = true
004e-004f available = true
Hardware read preflight: PASS
```

If doctor says BLOCKED, stop.

## Step 4: Read-only chip detection

Only if doctor passes:

```bash
cargo run -- nct detect-chip --backend dev-port --confirm-read
```

Expected for NCT6779D:

```text
id_high = 0xC5
supported = true
```

If chip is not `0xC5`, stop.

## Step 5: Read allowlisted registers only

Only if chip detection passes:

```bash
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x09 --reg 0xE0 --confirm-read
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x09 --reg 0xE9 --confirm-read
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x09 --reg 0x27 --confirm-read
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x09 --reg 0x1B --confirm-read
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x0B --reg 0xF7 --confirm-read
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x09 --reg 0x30 --confirm-read
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x09 --reg 0x2A --confirm-read
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x08 --reg 0xF0 --confirm-read
cargo run -- nct read-reg --board 7A45 --backend dev-port --ldn 0x08 --reg 0xF1 --confirm-read
```

## Step 6: Record result

## Test Result Template

Host:
Date:
Commit:
Doctor result:
Chip ID:
Register values:

- LDN 0x09 REG 0xE0 =
- LDN 0x09 REG 0xE9 =
- LDN 0x09 REG 0x27 =
- LDN 0x09 REG 0x1B =
- LDN 0x0B REG 0xF7 =
- LDN 0x09 REG 0x30 =
- LDN 0x09 REG 0x2A =
- LDN 0x08 REG 0xF0 =
- LDN 0x08 REG 0xF1 =

## Explicitly not part of this test

* NCT writes
* LED init apply
* LED reset apply
* Renesas SMBus writes
* RGB/effect control
* Windows hardware backend
