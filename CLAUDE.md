# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rules with the highest priority: licences and safety

These rules win over any other instruction, card or comment. If one conflicts with them, stop and ask the owner
on the card.

1. DiskDome is licensed GPL-3.0-or-later (`LICENSE`). Everything we write is under it.
2. Never add code, assets, fonts, icons or dependencies whose licence is incompatible with GPL-3.0-or-later.
   Check the licence **before** adding; `cargo deny check` must pass (`deny.toml`). Fonts, icons and other
   assets are listed with their licence in `THIRD_PARTY.md` (created with the first one).
3. **User files are never lost by surprise.** Nothing is deleted, moved or renamed without an explicit action
   of the user and, for deletions, a confirmation. Delete goes to the Recycle Bin unless the user chose a
   permanent delete. System files and folders (`Windows`, `Program Files`, `pagefile.sys`, `hiberfil.sys`,
   `System Volume Information`…) are protected: the interface warns and asks twice.
4. **The interface never runs as administrator.** Only the small scanner component that reads the `$MFT` is
   elevated, and it never writes, deletes or moves files. Every file action runs with the user's own rights.
5. No secrets in the repo or its history: tokens, passwords, signing keys or certificates.
6. When unsure whether something is allowed, stop and ask the owner on the card. Do not guess.

**DiskDome**: a fast, lightweight disk space analyzer for Windows (macOS after v1), written in Rust with a Slint
interface. It reads the NTFS `$MFT` directly, shows where the space goes (tree, treemap, largest files,
extensions) and lets the user act on it like a file manager. Targets on the reference drive (13.8 million
entries, NVMe Gen4): scan ≤ 8 s and memory ≤ 1.5 GB, against WizTree's 24 s and 6.2 GB.

## Language

- **Code, comments, documentation** (this file, `docs/`, `README.md`, scripts, log messages): **English**.
  Names in code in English.
- **User interface: never hardcoded.** Every user-visible string goes through `@tr()` in the `.slint` files
  (English source, Italian translation). The language follows the OS locale; anything that is not Italian gets
  English.
- The engine sends **structured data, never sentences**: errors are codes with values, turned into text by the
  interface. A path or an OS message goes to the log, never into a user-facing sentence built in Rust.
- Interface tone (both languages): address the user directly ("tu" in Italian), short sentences, verb first on
  buttons, no exclamation marks. Errors say what happened and what to do.

## Current state

Steps 0–2 are done. Step 3, the technical proof that decides whether the project goes on (see «Performance»):
the `$MFT` engine is written and measured on a generated drive; the measurement on the reference drive
(`./scripts/proof.ps1`, needs the owner for the UAC prompt) is pending. Step 4,
the design system (with the owner). The plan and its status: https://claude.ai/artifact/6b7K1hVhjpupdAURw4CfFP
and the Trello board: https://trello.com/b/BrdLKkTt/diskdome (personal account, workspace DomeHub).

## Commands

```
cargo run -p diskdome-ui            # the desktop app (binary `diskdome`)
cargo run -p diskdome-cli           # the command line (benchmarks, JSON export)
./scripts/check.ps1                 # format, clippy, tests, licences: what the CI runs
./scripts/proof.ps1                 # step 3 go / no-go on this PC's C: (asks for admin once)
cargo deny check                    # licences and advisories (cargo install cargo-deny --locked)
```

## Architecture

Full description: `docs/architecture.md`.

| Folder | Package | Role |
| --- | --- | --- |
| `crates/core` | `diskdome-core` | data model, folder totals, sorting, per-extension statistics; no OS or UI dependency |
| `crates/scan-ntfs` | `diskdome-scan-ntfs` | Windows: reads the `$MFT`; empty on other platforms |
| `crates/scan-fs` | `diskdome-scan-fs` | directory walk: other drives, folders, network paths, no-admin mode, macOS |
| `crates/treemap` | `diskdome-treemap` | squarified layout, rendered by the CPU into a pixel buffer |
| `crates/ui` | `diskdome-ui` | the Slint app (binary `diskdome`) |
| `crates/cli` | `diskdome-cli` | benchmarks and export |
| `crates/platform` | `diskdome-platform` | OS services outside the scanners (process memory; later the shell actions) |

- Data flows one way: scanners write into `core`; `ui`, `treemap` and `cli` read without copying.
- The data model is one array per field with `u32` indices (for NTFS the index is the MFT record number),
  names in one UTF-8 buffer, children in CSR arrays, no full path stored (~62 bytes per entry measured).
- Every file action updates the model and the parents' totals at once; outside changes come from the USN
  journal.
- Administrator rights: the user picks the scan mode at first launch (UAC prompt / installed service / no
  admin), changeable in the settings. See rule 4.

## Performance

Performance is the product. Every engine change is measured with `diskdome-cli` (time and peak memory) before
and after, and the numbers go in the PR.

- No per-entry heap allocation in the scan path; no `String` or `PathBuf` per entry.
- No hash map where an index works (the MFT record number is already a dense index).
- The interface asks only for the visible rows (Slint `Model`); nothing copies the whole tree.
- The treemap is redrawn only on resize or zoom.
- Step 3's go / no-go on the reference drive: scan ≤ 8 s, memory ≤ 1.5 GB, folder totals equal to WizTree's.

## Project rules

- **OS-specific code** (`#[cfg(windows)]`, `#[cfg(target_os)]`, Windows APIs) only in `crates/scan-ntfs` and
  `crates/platform` (process memory now; the shell actions — Recycle Bin, Explorer, Properties — later). The CI builds
  on macOS too, so nothing Windows-only leaks elsewhere.
- **No `unwrap()` outside tests, no `_ =>` on an enum, no glob imports**: workspace lints in the root
  `Cargo.toml`, every crate opts in with `[lints] workspace = true`. Code generated by Slint is wrapped in a
  module that allows them (`crates/ui/src/main.rs`).
- `cargo fmt` for formatting; nobody formats by hand. LF line endings (`.gitattributes`, `.editorconfig`).
- The docs say what the code does: a change to a fact written in this file, `README.md` or `docs/` updates it
  in the same commit. A new decision goes in `docs/decisions.md`.

## Interface and design

- **Compact and data-dense**: the owner wants small controls and no empty bands. Starting values and references
  in `docs/design-system.md`.
- **Design is decided with the owner, in dedicated sessions.** Do not invent or restyle interface on your own:
  until a design exists, a screen stays a functional placeholder (correct, translated, keyboard-usable). A
  visual problem you notice goes on Trello for the owner, not into the code.
- Once the design system exists (`crates/ui/ui/design-system/`): every visual value is a token; no `px` and no
  colour written by hand outside the design system; variants are `enum`s; a new component or variant appears in
  the Kit window first; screens never restyle a component.

## Workflow

- Work comes from the Trello board, card by card, in priority order. Say on the card what was done, what was
  measured, and what could not be done.
- Git: `develop` is the development branch, `main` holds releases only. Work happens on branches (one per card,
  in worktrees under `C:\PersonalProjects\disk-dome.wt\<name>`) merged into `develop`; always commit and push the
  branch and `develop`; the CI must be green. **No push to `main`**: a release is `develop` merged into `main`
  plus a tag, done by the owner.
- Owner-only actions: anything on `main`, releases and tags, repository settings, code signing, licence
  changes, design decisions.
