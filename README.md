# DiskDome

A fast, lightweight disk space analyzer for Windows, written in Rust.

DiskDome reads the NTFS Master File Table directly to show in seconds what fills your drives, then lets you
act on it: delete, move, rename, copy paths, find the folders you can safely rebuild. It aims to be faster
than the tools it replaces while using a fraction of their memory.

> **Status: early development.** Nothing to download yet. The plan and the decisions behind it are in
> [`docs/`](docs/); the work in progress is on the [Trello board](https://trello.com/b/BrdLKkTt/diskdome).

## Goals

Measured on a real drive (13.8 million files and folders, NVMe PCIe Gen4 SSD), against WizTree 4.31:

| | WizTree 4.31 | DiskDome target |
| --- | --- | --- |
| Full scan | 24 s | ≤ 8 s |
| Memory after the scan | 6.2 GB | ≤ 1.5 GB |
| Memory per entry | ~473 bytes | 65–100 bytes |

Beyond speed:

- **Safer by design.** The interface never runs as administrator; only a small component that reads the
  `$MFT` does. Deletions run with your own rights.
- **More than a viewer.** File-manager actions on what you find, kept in sync live through the NTFS change
  journal.
- **Compact, careful interface.** Dense, keyboard-friendly, built with [Slint](https://slint.dev/).

## Building

Requirements: Rust (stable, installed through [rustup](https://rustup.rs/)); on Windows the MSVC build tools.

```
cargo run -p diskdome-ui          # the desktop app
cargo run -p diskdome-cli         # the command line
cargo run -p diskdome-ui --features kit -- --kit   # the design system's Kit window (development only)
./scripts/check.ps1               # every check the CI runs
./scripts/proof.ps1               # measure a scan of C: and compare it with WizTree (asks for admin)
```

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — crates, data model, scan pipeline, privileges
- [`docs/features.md`](docs/features.md) — what is in v1 and what comes later
- [`docs/design-system.md`](docs/design-system.md) — the design system: principles, tokens, components, patterns
- [`docs/research.md`](docs/research.md) — measurements, alternatives considered, sources
- [`docs/decisions.md`](docs/decisions.md) — every decision and its reason

## Licence

DiskDome is free software under the [GNU General Public License v3.0 or later](LICENSE).
