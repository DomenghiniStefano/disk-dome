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
The Slint tokens (`crates/ui/ui/design-system/tokens/`) and the source checks of `docs/design-system.md` § 10 are in
place (`crates/ui/tests/design_conventions.rs`, `design_tokens.rs`). The first components (Label, Icon with the Lucide
set, Button, SegmentedControl, ToggleButton, SearchField, ScrollBar) are in `crates/ui/ui/design-system/components/`; the Kit window shows every token,
every component and the review board (cards 4.2, 4.3), and checks text and digits (card 4.T7). Slint draws with
its software renderer only (decision 9); Segoe UI at 400 and 700 only (decision 8). Menus and tooltips are ours (decision 11,
2026-11-26, after test 8). The menu is built (`Menu`, `MenuList`,
`MenuFrame`, `ContextMenu`, `MenuButton`, § 5.16) with `DesignWindow`, which every window inherits, and
`design_system::install`, which every window calls. Next: the Tooltip (§ 5.17). The plan and its status: https://claude.ai/artifact/6b7K1hVhjpupdAURw4CfFP
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
| `crates/ui` | `diskdome-ui` | the Slint app (binary `diskdome`) |
| `crates/cli` | `diskdome-cli` | benchmarks and export |
| `crates/platform` | `diskdome-platform` | OS services outside the scanners (process memory, the own title bar; later the shell actions) |

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
- **Before «done»**: `./scripts/check.ps1` (format, clippy and tests with every feature, licences) passes.

### Slint

- **Layers**: `tokens/` → `primitives/` → `components/` → screens. Each layer imports only the ones below it;
  screens and `app.slint` import only `design-system/index.slint`.
- **Components**: one per file in `design-system/components/`, exported from `index.slint`, with its variants
  and sizes as enums in `enums.slint`. The look for each variant comes from one pure-function global in the
  file (as `ButtonLook`), not from ternaries scattered over the elements.
- **Properties**: `in` by default; `out` only for what the component computes; `in-out` and two-way bindings
  (`<=>`) only when both sides really write, with a comment saying why. Everything that is not the component's
  interface is a private `property` (no direction).
- **Screens talk to Rust through globals**: one `export global` per area (`ScanApi`, `TreeApi`…) with
  properties and callbacks, re-exported by the build entry; one Rust adapter module per area wires it. Screens
  never get data through properties of the `Window`, so the Kit can show them with fake data.
- **Models are lazy**: a list or table implements `slint::Model` over `core` and builds only the rows asked
  for; no `VecModel` copy of the tree. Small fixed lists (tabs, menu items) may be a `VecModel`.
- **From another thread to the interface** only through `Weak::upgrade_in_event_loop` or
  `slint::invoke_from_event_loop`; their error (the event loop has ended) is logged, never dropped with `let _`.
- **Behaviour is never rewritten**: clicks, hover, press, Enter / Space and the focus ring come from
  `primitives/` (`Pressable`, `FocusTarget`, `FocusRing`); the ring shows only when `Nav.keyboard` is set.
  Disabled is one `opacity: Size.disabled` on the component's root.
- **Text only through `Label`, icons only through `Icon`** with `Icons.*`; colours through `InkTone`. A new
  icon: the SVG from the pinned Lucide version, its property in `icons.slint`, its entry in the Kit
  (`tests/design_icons.rs` checks all three).
- **A value the spec does not give** becomes a token under `// Pending design`, listed on the card for the
  owner. Never a literal, never a guess presented as decided.
- **Accessibility on every component**: `accessible-role`, `accessible-label`, `accessible-enabled`, and
  `accessible-checked` where it applies; an icon-only button always has its `accessible-label`.
- **The Kit is the proof**: every component, variant and state appears in `kit/kit.slint` (test
  `kit-coverage`); states are shown with `kit-state`, which screens never set. A visual change is looked at
  in the off-screen shots before it is called done:
  `DISKDOME_KIT_SHOTS=<folder> cargo test -p diskdome-ui --features kit --test kit_headless`. The shots are
  a local look only: never committed, never compared by a test.
- **Slint 1.19 traps**:
  - an element inside a component without a layout is centred at its preferred size, so an overlay sets
    `width: 100%; height: 100%`;
  - `&&` next to `||` needs parentheses;
  - an enum value whose name matches a property is written in full (`AccessibleRole.text`);
  - `Flickable` takes `content-height` and `content-y`, not the deprecated `viewport-*`;
  - `visible: false` keeps the element's place in a layout: to take no room, bind its min / preferred / max height
    to 0 (as `KitSection`) or use `if`;
  - a property named like a built-in one cannot be declared (`row` in a layout child): pick another name;
  - strings have no `contains` or regex: answer it in Rust with a `pure callback`;
  - off-screen shots do not advance time: animations and `Timer`s stay where they started, so a colour caught
    mid-animation in a shot is not a bug;
  - Segoe UI Semibold (600) has proportional digits; 400 and 700 are tabular, so the design uses only those (§ 3.3).

## Interface and design

- **Compact and data-dense**: the owner wants small controls and no empty bands. Starting values and references
  in `docs/design-system.md`.
- **Design is decided with the owner, in dedicated sessions.** Do not invent or restyle interface on your own:
  until a design exists, a screen stays a functional placeholder (correct, translated, keyboard-usable). A
  visual problem you notice goes on Trello for the owner, not into the code.
- Every visual value is a token; no `px` and no colour written by hand outside `design-system/tokens/`; a new
  component or variant appears in the Kit window first; screens never restyle a component (how: «Implementation
  rules › Slint» above).

## Workflow

- Work comes from the Trello board, card by card, in priority order. Say on the card what was done, what was
  measured, and what could not be done.
- Git: `develop` is the development branch, `main` holds releases only. Work happens on branches (one per card,
  in worktrees under `C:\PersonalProjects\disk-dome.wt\<name>`) merged into `develop`; always commit and push the
  branch and `develop`; the CI must be green. **No push to `main`**: a release is `develop` merged into `main`
  plus a tag, done by the owner.
- Owner-only actions: anything on `main`, releases and tags, repository settings, code signing, licence
  changes, design decisions.
