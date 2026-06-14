# Tested Environments

This document records known test environments and safety outcomes.

## Dell OptiPlex 5000 / Ubuntu Server

Status: non-target host, correctly blocked.

Observed DMI:

```text
board_vendor = Dell Inc.
board_name = 03V7GF
board_version = A01
product_name = OptiPlex 5000
looks_like_msi_7a45 = false
```

Observed preflight result:

```text
Hardware read preflight: BLOCKED
Reason: host DMI does not look like MSI 7A45: vendor=Dell Inc. board=03V7GF product=OptiPlex 5000
```

Safe regression result:

* `cargo test` passed: 38 tests
* `cargo clippy -- -D warnings` passed
* `cargo run -- nct plan-init-7a45` passed
* `cargo run -- nct init-7a45 --dry-run` passed
* `cargo run -- doctor` blocked hardware-read preflight as expected

No hardware-read commands were run on this host.

Commands intentionally not run:

```bash
cargo run -- nct detect-chip --backend dev-port --confirm-read
cargo run -- nct read-reg ...
```

## MSI GP66 Leopard 11UG / Nobara GNOME

Status: MSI-branded host, but non-target board, correctly blocked.

Observed DMI:

```text
board_vendor = Micro-Star International Co., Ltd.
board_name = MS-1543
board_version = REV:1.0
product_name = GP66 Leopard 11UG
looks_like_msi_7a45 = false
```

Observed preflight result:

```text
Hardware read preflight: BLOCKED
Reason: host DMI does not look like MSI 7A45: vendor=Micro-Star International Co., Ltd. board=MS-1543 product=GP66 Leopard 11UG
```

Safe regression result:

* `cargo test` passed: 38 tests
* `cargo clippy -- -D warnings` passed
* `cargo run -- nct plan-init-7a45` passed
* `cargo run -- nct init-7a45 --dry-run` passed
* `cargo run -- doctor` blocked hardware-read preflight as expected

No hardware-read commands were run on this host.

Commands intentionally not run:

```bash
cargo run -- nct detect-chip --backend dev-port --confirm-read
cargo run -- nct read-reg ...
```

## MSI 7A45 Target

Status: not tested yet.

Before any hardware-read on a real MSI `7A45` host:

1. run `cargo run -- doctor`
2. verify DMI passes
3. verify `/proc/ioports` does not show `004e-004f` busy
4. only then consider `detect-chip --confirm-read`
5. do not run write/apply commands because they are not implemented
