# Architecture

The parts that decide performance are DiskDome's own and independent of the UI toolkit: the scanners, the
data model, the treemap and the virtual tree table. Slint only draws the frame around them (panels, menus,
fields, text).

## Crates

| Folder | Package | Role |
| --- | --- | --- |
| `crates/core` | `diskdome-core` | data model, folder totals, sorting, per-extension statistics; no OS or UI dependency |
| `crates/scan-ntfs` | `diskdome-scan-ntfs` | Windows: reads the `$MFT` directly with `windows-rs`, parses it in parallel with `rayon`; empty on other platforms |
| `crates/scan-fs` | `diskdome-scan-fs` | other drives, single folders, network paths, the no-admin mode and macOS: parallel directory walk (`getattrlistbulk` on macOS) |
| `crates/treemap` | `diskdome-treemap` | squarified layout and its rendering into a pixel buffer |
| `crates/ui` | `diskdome-ui` (binary `diskdome`) | the Slint app: design system, screens, a lazy `Model` that serves only the visible rows |
| `crates/cli` | `diskdome-cli` | scan benchmarks and JSON export; every engine change is measured here |

Packages are prefixed `diskdome-` because `core` is the name of Rust's own core library.

Data flows one way: the scanners write into `core`; `ui`, `treemap` and `cli` read from `core` without
copying; the treemap hands `ui` nothing but a pixel buffer.

## Data model

One array per field (struct of arrays), `u32` indices, no full path stored (a path is rebuilt by walking up
the parents when needed).

```
names:       Vec<u8>   every name, UTF-8, in one buffer
name_off:    Vec<u32>  + name_len: Vec<u16>
parent:      Vec<u32>
first_child: Vec<u32>  + child_count: Vec<u32>  (children contiguous, CSR layout)
size, alloc: Vec<u64>
mtime:       Vec<u32>  Unix seconds, enough until 2106
ext:         Vec<u16>  interned extensions
flags:       Vec<u8>
```

About 41 fixed bytes plus ~25 bytes of name: ~65 bytes per entry, ~0.9 GB for the 13.8 million entries of the
reference drive (WizTree: 6.2 GB).

## NTFS scan pipeline

1. Get the `$MFT`'s extents with `FSCTL_GET_RETRIEVAL_POINTERS` (a 13.7 GB MFT is fragmented).
2. Read in 8–32 MB blocks with several requests in flight, to approach the drive's 4–5 GB/s.
3. Parse the blocks already read on every core while the next ones are being read.
4. Find each parent by array access: the MFT record number is already a dense index, so no hash map
   (`ntfs-reader`'s benchmark: 3.8 s with a `Vec`, 5.3 s with a `HashMap`).
5. Sum sizes in one bottom-up pass, in parallel per subtree.

NTFS cases to handle: attribute lists, non-resident attributes and data runs, hard links (counted once),
sparse and compressed files, alternate data streams, logical vs allocated size, reparse points. Check: folder
totals compared with WizTree's on the same drive.

Crates worth reading before writing our own parser: `ntfs-reader` (MFT in memory + USN journal), `ntfs`
(Colin Finck), `mft`.

## Keeping the model current

Every action of the app (delete, move, rename) updates the model and the totals of every parent folder at
once. Changes made outside the app arrive through the USN change journal, with no rescan.

## Administrator rights

- The interface always runs with the user's normal rights: deletions use the user's rights, and drag and drop
  with Explorer works (Windows blocks it between processes at different integrity levels).
- Only a small elevated component reads the `$MFT` and hands the data to the interface through shared memory
  (the array layout of the model suits it).
- The user picks the scan mode in a dialog at first launch and can change it in the settings:
  1. fast, with a UAC prompt at each scan (recommended);
  2. fast, without prompts: a service installed once (as Everything does);
  3. without administrator rights: directory walk, slower.

## Treemap

Redrawn only on resize or zoom: the CPU draws into a buffer (1920×1080, tens of thousands of shaded
rectangles, split across cores, a few milliseconds), handed to Slint as a `SharedPixelBuffer` (stable API).
Hover is a rectangle drawn on top. Rectangles under 1–2 px are merged. wgpu (Slint's integration sits behind an
`unstable-wgpu-*` feature) only where the profiler asks for it.

## Interface (Slint)

- `.slint` files are compiled into Rust at build time (`slint_build::compile` in `build.rs`,
  `slint::include_modules!()`).
- Lists and tables implement Slint's `Model` trait (row count + row N): Slint asks only for the visible rows.
- Live preview while developing: the VS Code extension, and inside the running app with
  `SLINT_LIVE_PREVIEW=1` and the `slint/live-preview` feature.
- Text through `@tr()`.
- Renderer: Slint's default for now; Skia is to be evaluated for text quality.

## CI and releases

- GitHub Actions: `windows-latest` and `macos-latest` run format, clippy and tests; `cargo-deny` checks
  licences, advisories and sources.
- To add: a benchmark of `diskdome-cli` on an NTFS test image, so a time or memory regression shows at once.
- Releases from a tag. Code signing: decided before the first release.

## Technical proof (step 3): the go / no-go

On the reference drive: scan ≤ 8 s, memory ≤ 1.5 GB, folder totals equal to WizTree's. Alongside: a Slint
table with 14 million fake rows that scrolls smoothly. If the engine misses the target, the project stops
before the design work is spent.
