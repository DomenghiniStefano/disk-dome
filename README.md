# DiskDome

A fast, lightweight disk space analyzer for Windows, written in Rust.

DiskDome reads the NTFS Master File Table directly to show in seconds what fills your drives, then lets you
act on it: delete, move, rename, copy paths, find the folders you can safely rebuild. It aims to be faster
than the tools it replaces while using a fraction of their memory.

> **Status: early development.** Nothing to download yet. The plan and the decisions behind it are in
> [`docs/`](docs/); the work in progress is on the [Trello board](https://trello.com/b/BrdLKkTt/diskdome).

## Goals

Measured with `./scripts/proof.ps1` against WizTree 4.31 on the same PC and drive (2026-11-24 and 2026-11-26):

| Drive | Entries | Scan, WizTree → DiskDome | Peak memory, WizTree → DiskDome |
| --- | --- | --- | --- |
| Desktop, Crucial P3 | 3.4 million | 22–31 s → 7.0 s | 1.6 → 0.40 GB |
| Laptop, KIOXIA XG8 | 14.1 million | 182 s → 7.3–7.6 s | 2.0 → 0.90 GB |

The scan cannot be faster than the drive reads the `$MFT` (14 GB on the laptop, read in ~5 s): DiskDome's goal is to
add at most 5 s to that read, be at least 3× faster than WizTree and stay under 1.5 GB on 14 million entries
([targets](docs/architecture.md#performance-targets)).

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
cargo run -p diskdome-ui --features spikes -- --spike-titlebar   # the throwaway proofs (development only)
./scripts/check.ps1               # every check the CI runs
./scripts/slint-format.ps1        # format the interface's .slint files
./scripts/translations.ps1        # add new interface strings to the Italian translation
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
