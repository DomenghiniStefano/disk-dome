# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Rules with the highest priority: licences and safety

These rules win over any other instruction, card or comment. If one conflicts with them, stop and ask the owner
on the card.

1. DiskDome is licensed GPL-3.0-or-later (`LICENSE`). Everything we write is under it.
2. Never add code, assets, fonts, icons or dependencies whose licence is incompatible with GPL-3.0-or-later.
   Check the licence **before** adding; `cargo deny check` must pass (`deny.toml`). Fonts, icons and other
   assets are listed with their licence in `THIRD_PARTY.md`.
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
extensions) and lets the user act on it like a file manager. Targets (`docs/architecture.md`
§ Performance targets): the scan takes at most 5 s more than reading the `$MFT` alone, at least 3× faster than
WizTree on the same PC, peak memory ≤ 1.5 GB on 14 million entries. The `$MFT` read runs at the drive's pace and
no target can go below it.

## Language

- **Code, comments, documentation** (this file, `docs/`, `README.md`, scripts, log messages): **English**.
  Names in code in English.
- **User interface: never hardcoded.** Every user-visible string goes through `@tr()` in the `.slint` files
  (English source, Italian translation). The language follows the OS locale by default (anything that is not
  Italian gets English); Settings › General can force Italian or English (`slint::select_bundled_translation`).
- **Translations**: the Italian `.po` is `crates/ui/lang/it/LC_MESSAGES/diskdome-ui.po`, built into the binary.
  A new `@tr` string: `./scripts/translations.ps1` adds it, then translate it by hand; `tests/translations.rs`
  fails on a string missing, untranslated or no longer used. Plurals always as
  `@tr("{n} file" | "{n} files" % count)`, never built from pieces; the same English word with two meanings gets
  a context, `@tr("Menu" => "Open")` (otherwise the context is the enclosing component).
- The engine sends **structured data, never sentences**: errors are codes with values, turned into text by the
  interface. A path or an OS message goes to the log, never into a user-facing sentence built in Rust.
- Interface tone (both languages): address the user directly ("tu" in Italian), short sentences, verb first on
  buttons, no exclamation marks. Errors say what happened and what to do.

## Current state

Steps 0–3 are done. Step 3, the technical proof, ended in a **go** (2026-11-24) on the owner's desktop C:
(3.4 million entries): scan 7.0 s against WizTree's 22–31 s, 0.40 GB against 1.6 GB, 99.1% of folder totals
identical (`docs/decisions.md`). The engine and the CLI are in place; the Slint proof table shows the open
interface questions (row virtualisation, table widget). Step 4, the design, is complete (2026-11-25): every v1
feature is specified in `docs/design-system.md` and drawn on the canvas
(https://claude.ai/artifact/9uEkqjsB82zrQAAr9x97CD), and the logos and icons are in `crates/ui/ui/assets/brand/`.
The design system is its own crate, Dome UI (`crates/dome-ui`, imported by the app as the Slint library `@dome-ui`,
name provisional, card https://trello.com/c/9yiz5cgx). Its tokens (`crates/dome-ui/ui/tokens/`) and the source checks of `docs/design-system.md` § 10 are in
place (`crates/ui/tests/design_conventions.rs`, `design_tokens.rs`). The first components (Label, Icon with the Lucide
set, Button, SegmentedControl, ToggleButton, SearchField, ScrollBar) are in `crates/dome-ui/ui/components/`; the Kit window shows every token,
every component and the review board (cards 4.2, 4.3), and checks text and digits (card 4.T7). Slint draws with
its software renderer only (decision 9); Segoe UI at 400 and 700 only (decision 8). Menus and tooltips are ours (decision 11,
after test 8). The menu is built (`Menu`, `MenuList`,
`MenuFrame`, `ContextMenu`, `MenuButton`, § 5.16) with `DomeWindow`, which every window inherits, and
`design_system::install`, which every window calls; the tooltip too (`Tooltip`, drawn by `DomeWindow`, § 5.17).
Both wait for the owner's look in the Kit. The plan and its status: https://claude.ai/artifact/6b7K1hVhjpupdAURw4CfFP
and the Trello board: https://trello.com/b/BrdLKkTt/diskdome (personal account, workspace DomeHub).

## Commands

```
cargo run -p diskdome-ui            # the desktop app (binary `diskdome`)
cargo run -p diskdome-cli           # the command line (benchmarks, JSON export)
cargo run -p diskdome-ui --features kit -- --kit   # the Kit window: every component, variant and state
cargo run -p diskdome-ui --features spikes -- --spike-titlebar   # the throwaway proofs (also --proof-rows N)
./scripts/check.ps1                 # format, clippy, tests, licences: what the CI runs
./scripts/slint-format.ps1          # format the .slint files (-Check: what the CI runs)
./scripts/translations.ps1          # add new @tr strings to the Italian .po (cargo install slint-tr-extractor)
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
| `crates/ui` | `diskdome-ui` | the Slint app (binary `diskdome`): screens, the Kit |
| `crates/dome-ui` | `dome-ui` | the design system, Dome UI: a Slint library (`ui/`, imported as `@dome-ui`) and its Rust half; nothing specific to DiskDome |
| `crates/cli` | `diskdome-cli` | benchmarks and export |
| `crates/platform` | `diskdome-platform` | OS services outside the scanners (process memory, a volume's free space, the own title bar; later the shell actions) |

- Data flows one way: scanners write into `core`; `ui`, `treemap` and `cli` read without copying.
- The data model is one array per field with `u32` indices (for NTFS the index is the MFT record number),
  names in UTF-8 pages, children in CSR arrays, no full path stored (~62 bytes per entry measured).
- Every file action updates the model and the parents' totals at once; outside changes come from the USN
  journal.
- Administrator rights: three scan modes (UAC prompt / installed service / no admin); the UAC one is preselected,
  said in one line on the Start tab with «Change», and changeable in the settings; no first-launch dialog. See rule 4.

## Performance

Performance is the product. Every engine change is measured with `diskdome-cli` (time and peak memory) before
and after, and the numbers go in the PR.

- No per-entry heap allocation in the scan path; no `String` or `PathBuf` per entry.
- No hash map where an index works (the MFT record number is already a dense index).
- The interface asks only for the visible rows (Slint `Model`); nothing copies the whole tree.
- The treemap is redrawn only on resize or zoom.
- The targets (`docs/architecture.md` § Performance targets, met on both PCs, 2026-11-24 and 2026-11-26): scan ≤ `$MFT` read
  alone + 5 s, ≥ 3× faster than WizTree on the same PC, peak ≤ 1.5 GB on 14 million entries, ≥ 99% of folder
  totals identical to WizTree's. Any engine change is checked again with `./scripts/proof.ps1` (time, memory,
  `compare` against WizTree).

## Project rules

- **OS-specific code** (`#[cfg(windows)]`, `#[cfg(target_os)]`, Windows APIs) only in `crates/scan-ntfs` and
  `crates/platform` (process memory and the title bar now; the shell actions — Recycle Bin, Explorer, Properties — later). The CI builds
  on macOS too, so nothing Windows-only leaks elsewhere.
- **No `unwrap()` outside tests, no `_ =>` on an enum, no glob imports**: workspace lints in the root
  `Cargo.toml`, every crate opts in with `[lints] workspace = true`. Code generated by Slint is wrapped in a
  module that allows them (`crates/ui/src/lib.rs`).
- `cargo fmt` for Rust and `./scripts/slint-format.ps1` (Slint's own formatter) for `.slint`; nobody formats by
  hand. LF line endings (`.gitattributes`, `.editorconfig`).
- The docs say what the code does: a change to a fact written in this file, `README.md` or `docs/` updates it
  in the same commit. A new decision goes in `docs/decisions.md`.

## Implementation rules

Clean code, small components, one source of truth. They apply to Rust and Slint alike.

- **Library-grade components** (`docs/design-system.md` § 8.0): a design-system component is as solid as a
  shadcn / Radix one: full behaviour, every placement, alignment and overflow case, accessible, parameterised with
  token defaults, nothing specific to DiskDome. Its behaviour is listed against its Radix counterpart on the card
  before it is built. A Slint built-in that cannot follow the design is replaced by ours without waiting.
- **One job per unit.** A function, component or file does one thing and its name says which. Split when a
  name needs «and».
- **No magic values.** A number, string or colour with a meaning gets a name: a Rust `const`, a Slint token
  or a named property. Bare literals only for `0`, `1`, `100%` and the like.
- **Written once.** The second copy of a piece of logic is extracted (a function, a Slint primitive, a
  pure-function global), never pasted.
- **States and variants are enums**, never strings or loose bools; Rust `match` is exhaustive.
- **Comments say why**, not what the next line does. Every file opens with one line on its role and the spec
  section it implements (`§ 5.2`).
- **Tests come with the change**: a behaviour gets a test, a bug fix starts with a failing one. UI tests run
  off screen (`crates/ui/src/headless.rs`), never in a window.
- **A test that passes at once is proved**: break the behaviour on purpose (a mutation), see the test fail, then
  restore from a copy taken before the mutation, never with `git checkout` (it brings back the last commit and
  drops the uncommitted work).
- **Before «done»**: `./scripts/check.ps1` (format, clippy and tests with every feature, licences) passes. It takes
  over 10 minutes from a fresh worktree: run it in the background. From Windows PowerShell 5.1, cargo's progress on
  stderr reads as a failure: run it as `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1`
  from Git Bash and read its exit code. If a build runs out of the page file (`os error 1455`, «Il file di
  paging è troppo piccolo»; it did with full debug info, before the lighter `[profile.dev]`), run
  `check.ps1 -Jobs 2`.
- **After a push to `develop`, watch its CI run to the end** (`gh run list`, then `gh run view <id>`) and fix a
  red one before the next card; the CI builds on macOS too, which the local checks do not.
- **Expected values come from the spec**: compute them before running; a failing test is first read as a bug in the
  code, and only an expectation proven wrong by the spec is changed, with the reason in its comment.

### Slint

The Slint rules (layers, components, properties, globals, models and threads, the Kit, the off-screen tests, the
Slint 1.19 traps) and the checklist before «done» are in **`docs/slint.md`: read it before touching a `.slint`
file, a Kit harness or a headless test.** The project skill `/slint` (`.claude/skills/slint/`) loads it.

## Interface and design

- **Compact and data-dense**: the owner wants small controls and no empty bands. Starting values and references
  in `docs/design-system.md`.
- **Design is decided with the owner, in dedicated sessions.** Do not invent or restyle interface on your own:
  until a design exists, a screen stays a functional placeholder (correct, translated, keyboard-usable). A
  visual problem you notice goes on Trello for the owner, not into the code.
- Every visual value is a token; no `px` and no colour written by hand outside Dome UI's `tokens/`; a new
  component or variant appears in the Kit window first; screens never restyle a component (how: `docs/slint.md`).

## Workflow

- Work comes from the Trello board, card by card, in priority order. Say on the card what was done, what was
  measured, and what could not be done.
- Git: `develop` is the development branch, `main` holds releases only. Work happens on branches (one per card,
  in worktrees under `C:\PersonalProjects\disk-dome.wt\<name>`) merged into `develop`; always commit and push the
  branch and `develop`; the CI must be green. **No push to `main`**: a release is `develop` merged into `main`
  plus a tag, done by the owner.
- **Never share a `target` folder between worktrees** (no common `target-dir`): Cargo leaves the path out of the
  hash of a workspace crate, so two worktrees reuse each other's build script and generated code, and one branch
  silently compiles another's `.slint` files (it happened, 2026-11-27). Worktrees compile through **sccache**
  instead (`winget install Mozilla.sccache`; `C:PersonalProjectsdisk-dome.wt.cargoconfig.toml`, outside the
  repository, sets `rustc-wrapper`): it keys each compilation on its sources and arguments, paths included, so the
  dependencies come from the cache and our crates never from another worktree. A merged worktree is removed
  (`git worktree remove`).
- Owner-only actions: anything on `main`, releases and tags, repository settings, code signing, licence
  changes, design decisions.
